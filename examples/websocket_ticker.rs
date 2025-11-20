//! WebSocket ticker subscription example
//!
//! This example demonstrates how to connect to OKX WebSocket API
//! and subscribe to real-time ticker updates.
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables (or use .env file)
//! export OKX_API_KEY="your-api-key"
//! export OKX_SECRET_KEY="your-secret-key"
//! export OKX_PASSPHRASE="your-passphrase"
//! export OKX_TESTNET="true"  # Optional, default false
//!
//! # Run the example
//! cargo run --example websocket_ticker
//! ```

use ea_okx_client::auth::Credentials;
use ea_okx_client::models::{Channel, SubscriptionRequest, WebSocketEvent};
use ea_okx_client::websocket::{OkxWebSocketClient, WebSocketConfig};
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("OKX WebSocket Ticker Example");

    // Load credentials from environment variables
    let api_key = env::var("OKX_API_KEY").unwrap_or_else(|_| "demo-api-key".to_string());
    let secret_key = env::var("OKX_SECRET_KEY").unwrap_or_else(|_| "demo-secret-key".to_string());
    let passphrase = env::var("OKX_PASSPHRASE").unwrap_or_else(|_| "demo-passphrase".to_string());
    let is_testnet = env::var("OKX_TESTNET").unwrap_or_else(|_| "true".to_string()) == "true";

    info!("Using testnet: {}", is_testnet);

    // Create credentials
    let credentials = Credentials::new(&api_key, &secret_key, &passphrase);

    // Create WebSocket client with custom config
    let config = WebSocketConfig {
        auto_reconnect: true,
        max_reconnect_attempts: 5,
        reconnect_delay_ms: 1000,
        max_reconnect_delay_ms: 30000,
        heartbeat_interval_secs: 20,
        pong_timeout_secs: 30,
    };

    let mut client = OkxWebSocketClient::with_config(credentials, is_testnet, config);

    // Connect to WebSocket
    info!("Connecting to OKX WebSocket...");
    match client.connect().await {
        Ok(_) => info!("Successfully connected to OKX WebSocket"),
        Err(e) => {
            error!("Failed to connect: {}", e);
            return Err(e.into());
        }
    }

    // Subscribe to multiple tickers
    let symbols = vec!["BTC-USDT", "ETH-USDT", "SOL-USDT"];
    let subscriptions: Vec<SubscriptionRequest> = symbols
        .iter()
        .map(|symbol| SubscriptionRequest::new(Channel::Tickers, *symbol))
        .collect();

    info!("Subscribing to tickers: {:?}", symbols);
    client.subscribe(subscriptions).await?;

    // Also subscribe to 1-minute candles for BTC
    let candle_sub = SubscriptionRequest::new(Channel::Candle1m, "BTC-USDT");
    info!("Subscribing to BTC-USDT 1m candles");
    client.subscribe(vec![candle_sub]).await?;

    // Receive and process messages
    info!("Starting message processing loop...");
    let mut message_count = 0;
    let max_messages = 50; // Limit messages for demo

    while let Some(event) = client.next_message().await? {
        message_count += 1;

        match event {
            WebSocketEvent::Subscribe(resp) => {
                info!("✓ Subscription confirmed: {:?}", resp.arg);
            }
            WebSocketEvent::Ticker(ticker) => {
                info!(
                    "📊 Ticker {} - Last: {}, Bid: {}, Ask: {}, 24h Vol: {}",
                    ticker.inst_id,
                    ticker.last,
                    ticker.bid_px,
                    ticker.ask_px,
                    ticker.vol_24h
                );
            }
            WebSocketEvent::Candle(candle) => {
                match candle.parse() {
                    Ok(parsed) => {
                        info!(
                            "🕯️ Candle - O: {}, H: {}, L: {}, C: {}, V: {}, Confirmed: {}",
                            parsed.open,
                            parsed.high,
                            parsed.low,
                            parsed.close,
                            parsed.volume,
                            parsed.is_confirmed
                        );
                    }
                    Err(e) => error!("Failed to parse candle: {}", e),
                }
            }
            WebSocketEvent::Error { code, msg } => {
                error!("❌ Error - Code: {}, Message: {}", code, msg);
            }
            WebSocketEvent::Login { code, msg } => {
                if code == "0" {
                    info!("🔐 Login successful");
                } else {
                    error!("🔒 Login failed - Code: {}, Message: {}", code, msg);
                }
            }
            _ => {
                info!("Received event: {:?}", event);
            }
        }

        // Exit after max messages for demo purposes
        if message_count >= max_messages {
            info!("Reached {} messages, disconnecting...", max_messages);
            break;
        }
    }

    // Disconnect
    info!("Disconnecting from WebSocket...");
    client.disconnect().await?;
    info!("Disconnected successfully");

    Ok(())
}
