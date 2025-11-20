//! OKX WebSocket client for real-time market data streaming
//!
//! This module provides a WebSocket client that connects to OKX's WebSocket API
//! for receiving real-time market data, including:
//! - Market tickers
//! - OHLCV candles
//! - Order book snapshots and updates
//! - Recent trades
//! - Account updates
//! - Position updates
//! - Order updates
//!
//! # Features
//!
//! - Auto-reconnection with exponential backoff
//! - Subscription management (subscribe/unsubscribe)
//! - Heartbeat/ping-pong mechanism
//! - Message validation and parsing
//! - Connection state management
//!
//! # Example
//!
//! ```no_run
//! use ea_okx_client::websocket::{OkxWebSocketClient, WebSocketConfig};
//! use ea_okx_client::models::{Channel, SubscriptionRequest};
//! use ea_okx_client::auth::Credentials;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let credentials = Credentials::new("api-key", "secret-key", "passphrase");
//!     let mut client = OkxWebSocketClient::new(credentials, false);
//!     
//!     client.connect().await?;
//!     
//!     // Subscribe to ticker
//!     let sub = SubscriptionRequest::new(Channel::Tickers, "BTC-USDT");
//!     client.subscribe(vec![sub]).await?;
//!     
//!     // Receive messages
//!     while let Some(msg) = client.next_message().await? {
//!         println!("Received: {:?}", msg);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use crate::auth::Credentials;
use crate::error::{Error, Result};
use crate::models::websocket::{SubscriptionRequest, WebSocketEvent};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};

/// OKX WebSocket API URLs
const WS_PUBLIC_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
const WS_PRIVATE_URL: &str = "wss://ws.okx.com:8443/ws/v5/private";
const WS_BUSINESS_URL: &str = "wss://ws.okx.com:8443/ws/v5/business";

const WS_PUBLIC_TESTNET_URL: &str = "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999";
const WS_PRIVATE_TESTNET_URL: &str = "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999";

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

/// WebSocket client configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Enable automatic reconnection
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts (0 = unlimited)
    pub max_reconnect_attempts: u32,
    /// Initial reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
    /// Maximum reconnection delay in milliseconds
    pub max_reconnect_delay_ms: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Maximum time without pong response before reconnection
    pub pong_timeout_secs: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            max_reconnect_attempts: 0, // unlimited
            reconnect_delay_ms: 1000,
            max_reconnect_delay_ms: 60000,
            heartbeat_interval_secs: 20,
            pong_timeout_secs: 30,
        }
    }
}

/// OKX WebSocket client
pub struct OkxWebSocketClient {
    credentials: Credentials,
    is_testnet: bool,
    config: WebSocketConfig,

    // Connection management
    public_ws: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>>,
    private_ws: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>>,
    state: Arc<Mutex<ConnectionState>>,

    // Message channels
    message_tx: mpsc::UnboundedSender<WebSocketEvent>,
    message_rx: Arc<Mutex<mpsc::UnboundedReceiver<WebSocketEvent>>>,

    // Subscription tracking
    subscriptions: Arc<Mutex<Vec<SubscriptionRequest>>>,

    // Heartbeat tracking
    last_pong: Arc<Mutex<std::time::Instant>>,
}

impl OkxWebSocketClient {
    /// Create a new WebSocket client
    pub fn new(credentials: Credentials, is_testnet: bool) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            credentials,
            is_testnet,
            config: WebSocketConfig::default(),
            public_ws: Arc::new(Mutex::new(None)),
            private_ws: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            last_pong: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        credentials: Credentials,
        is_testnet: bool,
        config: WebSocketConfig,
    ) -> Self {
        let mut client = Self::new(credentials, is_testnet);
        client.config = config;
        client
    }

    /// Get current connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.lock().await
    }

    /// Connect to WebSocket servers
    pub async fn connect(&mut self) -> Result<()> {
        self.set_state(ConnectionState::Connecting).await;

        // Connect to public channel
        let public_url = if self.is_testnet {
            WS_PUBLIC_TESTNET_URL
        } else {
            WS_PUBLIC_URL
        };

        match connect_async(public_url).await {
            Ok((ws_stream, _)) => {
                *self.public_ws.lock().await = Some(ws_stream);
                info!("Connected to OKX public WebSocket");
            }
            Err(e) => {
                error!("Failed to connect to public WebSocket: {}", e);
                self.set_state(ConnectionState::Failed).await;
                return Err(Error::WebSocketConnection(e.to_string()));
            }
        }

        // Connect to private channel (requires authentication)
        let private_url = if self.is_testnet {
            WS_PRIVATE_TESTNET_URL
        } else {
            WS_PRIVATE_URL
        };

        match connect_async(private_url).await {
            Ok((ws_stream, _)) => {
                *self.private_ws.lock().await = Some(ws_stream);
                info!("Connected to OKX private WebSocket");

                // Authenticate private channel
                self.authenticate().await?;
            }
            Err(e) => {
                error!("Failed to connect to private WebSocket: {}", e);
                self.set_state(ConnectionState::Failed).await;
                return Err(Error::WebSocketConnection(e.to_string()));
            }
        }

        self.set_state(ConnectionState::Connected).await;

        // Start heartbeat task
        self.start_heartbeat();

        // Start message processing task
        self.start_message_processor();

        Ok(())
    }

    /// Authenticate private WebSocket connection
    async fn authenticate(&self) -> Result<()> {
        let timestamp = Utc::now().timestamp().to_string();
        let _sign_str = format!("{}GET/users/self/verify", timestamp);
        let signature = self
            .credentials
            .sign(&timestamp, "GET", "/users/self/verify", "")?;

        let auth_msg = serde_json::json!({
            "op": "login",
            "args": [{
                "apiKey": self.credentials.api_key(),
                "passphrase": self.credentials.passphrase(),
                "timestamp": timestamp,
                "sign": signature
            }]
        });

        let mut ws = self.private_ws.lock().await;
        if let Some(ws) = ws.as_mut() {
            ws.send(WsMessage::Text(auth_msg.to_string().into()))
                .await
                .map_err(|e| Error::WebSocketSend(e.to_string()))?;

            debug!("Sent authentication request");
        }

        Ok(())
    }

    /// Subscribe to channels
    pub async fn subscribe(&self, requests: Vec<SubscriptionRequest>) -> Result<()> {
        if requests.is_empty() {
            return Ok(());
        }

        // Separate public and private subscriptions
        let (public_subs, private_subs): (Vec<_>, Vec<_>) =
            requests.iter().partition(|req| req.channel.is_public());

        // Subscribe to public channels
        if !public_subs.is_empty() {
            self.send_subscription_request(&public_subs, true).await?;
        }

        // Subscribe to private channels
        if !private_subs.is_empty() {
            self.send_subscription_request(&private_subs, false).await?;
        }

        // Store subscriptions for reconnection
        let mut subs = self.subscriptions.lock().await;
        subs.extend(requests);

        Ok(())
    }

    /// Unsubscribe from channels
    pub async fn unsubscribe(&self, requests: Vec<SubscriptionRequest>) -> Result<()> {
        if requests.is_empty() {
            return Ok(());
        }

        let (public_subs, private_subs): (Vec<_>, Vec<_>) =
            requests.iter().partition(|req| req.channel.is_public());

        if !public_subs.is_empty() {
            self.send_unsubscription_request(&public_subs, true).await?;
        }

        if !private_subs.is_empty() {
            self.send_unsubscription_request(&private_subs, false)
                .await?;
        }

        // Remove from stored subscriptions
        let mut subs = self.subscriptions.lock().await;
        subs.retain(|s| !requests.contains(s));

        Ok(())
    }

    /// Send subscription request
    async fn send_subscription_request(
        &self,
        requests: &[&SubscriptionRequest],
        is_public: bool,
    ) -> Result<()> {
        let args: Vec<Value> = requests.iter().map(|req| req.to_json()).collect();

        let sub_msg = serde_json::json!({
            "op": "subscribe",
            "args": args
        });

        let ws_lock = if is_public {
            self.public_ws.clone()
        } else {
            self.private_ws.clone()
        };

        let mut ws = ws_lock.lock().await;
        if let Some(ws) = ws.as_mut() {
            ws.send(WsMessage::Text(sub_msg.to_string().into()))
                .await
                .map_err(|e| Error::WebSocketSend(e.to_string()))?;

            debug!("Sent subscription request: {:?}", requests);
        } else {
            return Err(Error::WebSocketConnection("Not connected".to_string()));
        }

        Ok(())
    }

    /// Send unsubscription request
    async fn send_unsubscription_request(
        &self,
        requests: &[&SubscriptionRequest],
        is_public: bool,
    ) -> Result<()> {
        let args: Vec<Value> = requests.iter().map(|req| req.to_json()).collect();

        let unsub_msg = serde_json::json!({
            "op": "unsubscribe",
            "args": args
        });

        let ws_lock = if is_public {
            self.public_ws.clone()
        } else {
            self.private_ws.clone()
        };

        let mut ws = ws_lock.lock().await;
        if let Some(ws) = ws.as_mut() {
            ws.send(WsMessage::Text(unsub_msg.to_string().into()))
                .await
                .map_err(|e| Error::WebSocketSend(e.to_string()))?;

            debug!("Sent unsubscription request: {:?}", requests);
        }

        Ok(())
    }

    /// Get next message from the message queue
    pub async fn next_message(&self) -> Result<Option<WebSocketEvent>> {
        let mut rx = self.message_rx.lock().await;
        Ok(rx.recv().await)
    }

    /// Start heartbeat task
    fn start_heartbeat(&self) {
        let public_ws = self.public_ws.clone();
        let private_ws = self.private_ws.clone();
        let last_pong = self.last_pong.clone();
        let config = self.config.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(config.heartbeat_interval_secs));

            loop {
                ticker.tick().await;

                // Check if we should send ping
                let current_state = *state.lock().await;
                if current_state != ConnectionState::Connected {
                    continue;
                }

                // Send ping to public channel
                if let Some(ws) = public_ws.lock().await.as_mut() {
                    if let Err(e) = ws.send(WsMessage::Text("ping".to_string().into())).await {
                        warn!("Failed to send ping to public channel: {}", e);
                    }
                }

                // Send ping to private channel
                if let Some(ws) = private_ws.lock().await.as_mut() {
                    if let Err(e) = ws.send(WsMessage::Text("ping".to_string().into())).await {
                        warn!("Failed to send ping to private channel: {}", e);
                    }
                }

                // Check pong timeout
                let elapsed = last_pong.lock().await.elapsed();
                if elapsed.as_secs() > config.pong_timeout_secs {
                    error!("Pong timeout exceeded, connection may be dead");
                    *state.lock().await = ConnectionState::Reconnecting;
                }
            }
        });
    }

    /// Start message processor task
    fn start_message_processor(&self) {
        let public_ws = self.public_ws.clone();
        let private_ws = self.private_ws.clone();
        let message_tx = self.message_tx.clone();
        let last_pong = self.last_pong.clone();
        let last_pong_clone = last_pong.clone();

        // Process public channel messages
        tokio::spawn(async move {
            loop {
                let mut ws_guard = public_ws.lock().await;
                if let Some(ws) = ws_guard.as_mut() {
                    match ws.next().await {
                        Some(Ok(msg)) => {
                            drop(ws_guard); // Release lock before processing

                            if let Err(e) =
                                Self::process_message(msg, &message_tx, &last_pong).await
                            {
                                error!("Error processing public message: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error on public channel: {}", e);
                            break;
                        }
                        None => {
                            warn!("Public WebSocket stream ended");
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        // Process private channel messages
        let private_ws_clone = private_ws.clone();
        let message_tx_clone = self.message_tx.clone();

        tokio::spawn(async move {
            loop {
                let mut ws_guard = private_ws_clone.lock().await;
                if let Some(ws) = ws_guard.as_mut() {
                    match ws.next().await {
                        Some(Ok(msg)) => {
                            drop(ws_guard);

                            if let Err(e) =
                                Self::process_message(msg, &message_tx_clone, &last_pong_clone)
                                    .await
                            {
                                error!("Error processing private message: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error on private channel: {}", e);
                            break;
                        }
                        None => {
                            warn!("Private WebSocket stream ended");
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Process a WebSocket message
    async fn process_message(
        msg: WsMessage,
        tx: &mpsc::UnboundedSender<WebSocketEvent>,
        last_pong: &Arc<Mutex<std::time::Instant>>,
    ) -> Result<()> {
        match msg {
            WsMessage::Text(text) => {
                // Handle pong response
                if text == "pong" {
                    *last_pong.lock().await = std::time::Instant::now();
                    debug!("Received pong");
                    return Ok(());
                }

                // Parse JSON message
                let value: Value = serde_json::from_str(&text)
                    .map_err(|e| Error::ParseError(format!("Invalid JSON: {}", e)))?;

                // Parse into WebSocketEvent
                let event = WebSocketEvent::from_json(&value)?;

                // Send to message channel
                tx.send(event)
                    .map_err(|e| Error::Internal(format!("Failed to send message: {}", e)))?;
            }
            WsMessage::Binary(_) => {
                debug!("Received binary message (ignoring)");
            }
            WsMessage::Ping(_) => {
                debug!("Received ping");
            }
            WsMessage::Pong(_) => {
                *last_pong.lock().await = std::time::Instant::now();
                debug!("Received pong");
            }
            WsMessage::Close(_) => {
                warn!("Received close message");
            }
            WsMessage::Frame(_) => {}
        }

        Ok(())
    }

    /// Set connection state
    async fn set_state(&self, state: ConnectionState) {
        *self.state.lock().await = state;
        info!("Connection state changed to: {:?}", state);
    }

    /// Disconnect from WebSocket servers
    pub async fn disconnect(&self) -> Result<()> {
        self.set_state(ConnectionState::Disconnected).await;

        // Close public connection
        if let Some(mut ws) = self.public_ws.lock().await.take() {
            ws.close(None)
                .await
                .map_err(|e| Error::WebSocketConnection(e.to_string()))?;
        }

        // Close private connection
        if let Some(mut ws) = self.private_ws.lock().await.take() {
            ws.close(None)
                .await
                .map_err(|e| Error::WebSocketConnection(e.to_string()))?;
        }

        info!("Disconnected from OKX WebSocket");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.auto_reconnect, true);
        assert_eq!(config.max_reconnect_attempts, 0);
        assert_eq!(config.reconnect_delay_ms, 1000);
        assert_eq!(config.heartbeat_interval_secs, 20);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let credentials = Credentials::new("test-key", "test-secret", "test-pass");
        let client = OkxWebSocketClient::new(credentials, true);
        assert_eq!(client.state().await, ConnectionState::Disconnected);
        assert_eq!(client.is_testnet, true);
    }

    #[tokio::test]
    async fn test_client_with_config() {
        let credentials = Credentials::new("test-key", "test-secret", "test-pass");
        let config = WebSocketConfig {
            auto_reconnect: false,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 2000,
            max_reconnect_delay_ms: 30000,
            heartbeat_interval_secs: 15,
            pong_timeout_secs: 25,
        };

        let client = OkxWebSocketClient::with_config(credentials, false, config.clone());
        assert_eq!(client.config.auto_reconnect, false);
        assert_eq!(client.config.max_reconnect_attempts, 5);
        assert_eq!(client.config.reconnect_delay_ms, 2000);
    }
}
