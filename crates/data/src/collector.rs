//! Market data collector
//!
//! Collects real-time market data from OKX WebSocket streams,
//! applies quality control, and stores to database/cache.

use crate::error::{Error, Result};
use crate::quality::{QualityConfig, QualityControl};
use crate::storage::{Candle, RedisStorage, Tick, TimescaleStorage};
use chrono::Utc;
use ea_okx_client::models::{
    CandleData, Channel, SubscriptionRequest, TickerData, TradeData, WebSocketEvent,
};
use ea_okx_client::websocket::OkxWebSocketClient;
use ea_okx_client::Credentials;
use ea_okx_core::types::{Price, Quantity, Symbol};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Market data collector configuration
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Symbols to collect data for
    pub symbols: Vec<String>,

    /// Channels to subscribe to
    pub channels: Vec<Channel>,

    /// Quality control configuration
    pub quality_config: QualityConfig,

    /// Enable TimescaleDB storage
    pub enable_timescale: bool,

    /// Enable Redis caching
    pub enable_redis: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            symbols: vec!["BTC-USDT".to_string()],
            channels: vec![Channel::Tickers, Channel::Candle1m, Channel::Trades],
            quality_config: QualityConfig::default(),
            enable_timescale: false,
            enable_redis: false,
        }
    }
}

/// Market data collector
pub struct MarketDataCollector {
    config: CollectorConfig,
    ws_client: Option<OkxWebSocketClient>,
    quality_control: Arc<QualityControl>,
    timescale: Option<TimescaleStorage>,
    redis: Option<RedisStorage>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl MarketDataCollector {
    /// Create a new market data collector
    pub fn new(config: CollectorConfig) -> Self {
        let quality_control = Arc::new(QualityControl::new(config.quality_config.clone()));

        Self {
            config,
            ws_client: None,
            quality_control,
            timescale: None,
            redis: None,
            shutdown_tx: None,
        }
    }

    /// Initialize connections
    pub async fn initialize(
        &mut self,
        credentials: Credentials,
        is_testnet: bool,
        timescale_url: Option<&str>,
        redis_url: Option<&str>,
    ) -> Result<()> {
        // Initialize WebSocket client
        let mut ws_client = OkxWebSocketClient::new(credentials, is_testnet);
        ws_client
            .connect()
            .await
            .map_err(|e| Error::WebSocketError(e))?;

        // Subscribe to channels
        let mut subscriptions = Vec::new();
        for symbol in &self.config.symbols {
            for channel in &self.config.channels {
                subscriptions.push(SubscriptionRequest::new(channel.clone(), symbol));
            }
        }

        ws_client
            .subscribe(subscriptions)
            .await
            .map_err(|e| Error::WebSocketError(e))?;
        self.ws_client = Some(ws_client);

        // Initialize storage backends
        if self.config.enable_timescale {
            if let Some(url) = timescale_url {
                self.timescale = Some(TimescaleStorage::new(url).await?);
                info!("TimescaleDB storage initialized");
            }
        }

        if self.config.enable_redis {
            if let Some(url) = redis_url {
                self.redis = Some(RedisStorage::new(url)?);
                info!("Redis cache initialized");
            }
        }

        info!(
            "Market data collector initialized for {} symbols",
            self.config.symbols.len()
        );
        Ok(())
    }

    /// Start collecting data
    pub async fn start(&mut self) -> Result<()> {
        let ws_client = self
            .ws_client
            .as_ref()
            .ok_or_else(|| Error::ConfigError("WebSocket client not initialized".to_string()))?;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        info!("Starting market data collection...");

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping collector");
                    break;
                }

                // Process WebSocket messages
                event = ws_client.next_message() => {
                    match event {
                        Ok(Some(evt)) => {
                            if let Err(e) = self.process_event(evt).await {
                                error!("Error processing event: {}", e);
                            }
                        }
                        Ok(None) => {
                            warn!("WebSocket stream ended");
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        info!("Market data collector stopped");
        Ok(())
    }

    /// Process WebSocket event
    async fn process_event(&self, event: WebSocketEvent) -> Result<()> {
        match event {
            WebSocketEvent::Ticker(ticker) => {
                self.process_ticker(ticker).await?;
            }
            WebSocketEvent::Candle(candle) => {
                self.process_candle(candle).await?;
            }
            WebSocketEvent::Trade(trade) => {
                self.process_trade(trade).await?;
            }
            WebSocketEvent::Subscribe(resp) => {
                info!("Subscription confirmed: {:?}", resp.arg);
            }
            WebSocketEvent::Error { code, msg } => {
                error!("WebSocket error - Code: {}, Message: {}", code, msg);
            }
            _ => {
                // Ignore other events
            }
        }

        Ok(())
    }

    /// Process ticker data
    async fn process_ticker(&self, ticker: TickerData) -> Result<()> {
        let symbol = Symbol::new(&ticker.inst_id)?;
        let price = Price::new(
            ticker
                .last
                .parse()
                .map_err(|e| Error::ParseError(format!("{}", e)))?,
        )?;
        let timestamp = Utc::now(); // Use current time since ticker doesn't have exact timestamp

        // Quality control
        if let Err(e) = self
            .quality_control
            .validate_market_data(&symbol, &price, timestamp, None)
        {
            warn!("Ticker quality check failed: {}", e);
            return Ok(()); // Don't propagate QC errors
        }

        info!("Ticker {} - Last: {}", ticker.inst_id, ticker.last);
        Ok(())
    }

    /// Process candle data
    async fn process_candle(&self, candle_data: CandleData) -> Result<()> {
        let parsed = candle_data
            .parse()
            .map_err(|e| Error::ParseError(format!("{}", e)))?;

        // Skip unconfirmed candles
        if !parsed.is_confirmed {
            return Ok(());
        }

        let price = Price::new(parsed.close)?;
        let symbol = Symbol::new("UNKNOWN")?; // Need to track symbol from subscription
        let timestamp = chrono::DateTime::from_timestamp_millis(parsed.timestamp)
            .ok_or_else(|| Error::ParseError("Invalid timestamp".to_string()))?;

        // Quality control
        if let Err(e) = self
            .quality_control
            .validate_market_data(&symbol, &price, timestamp, None)
        {
            warn!("Candle quality check failed: {}", e);
            return Ok(());
        }

        // Store candle
        if let Some(ts) = &self.timescale {
            let candle = Candle {
                symbol: symbol.clone(),
                timestamp,
                interval: "1m".to_string(),
                open: Price::new(parsed.open)?,
                high: Price::new(parsed.high)?,
                low: Price::new(parsed.low)?,
                close: price,
                volume: Quantity::new(parsed.volume)?,
                quote_volume: parsed.volume,
                trade_count: 0,
                vwap: None,
            };
            ts.store_candle(&candle).await?;
        }

        info!(
            "Candle - O: {}, H: {}, L: {}, C: {}",
            parsed.open, parsed.high, parsed.low, parsed.close
        );
        Ok(())
    }

    /// Process trade data
    async fn process_trade(&self, trade: TradeData) -> Result<()> {
        let symbol = Symbol::new(&trade.inst_id)?;
        let price = Price::new(
            trade
                .px
                .parse()
                .map_err(|e| Error::ParseError(format!("{}", e)))?,
        )?;
        let timestamp = chrono::DateTime::from_timestamp_millis(
            trade
                .ts
                .parse()
                .map_err(|e| Error::ParseError(format!("{}", e)))?,
        )
        .ok_or_else(|| Error::ParseError("Invalid timestamp".to_string()))?;

        // Quality control
        if let Err(e) = self.quality_control.validate_market_data(
            &symbol,
            &price,
            timestamp,
            Some(&trade.trade_id),
        ) {
            warn!("Trade quality check failed: {}", e);
            return Ok(());
        }

        // Store tick
        if let Some(ts) = &self.timescale {
            let tick = Tick {
                symbol: symbol.clone(),
                timestamp,
                trade_id: trade.trade_id,
                price,
                quantity: Quantity::new(
                    trade
                        .sz
                        .parse()
                        .map_err(|e| Error::ParseError(format!("{}", e)))?,
                )?,
                side: trade.side,
                is_block_trade: false,
            };
            ts.store_tick(&tick).await?;
        }

        Ok(())
    }

    /// Stop the collector
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(ws) = self.ws_client.as_ref() {
            ws.disconnect()
                .await
                .map_err(|e| Error::WebSocketError(e))?;
        }

        Ok(())
    }

    /// Get quality control statistics
    pub fn get_stats(&self) -> crate::quality::QualityStats {
        self.quality_control.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_config_default() {
        let config = CollectorConfig::default();
        assert_eq!(config.symbols.len(), 1);
        assert!(config.symbols.contains(&"BTC-USDT".to_string()));
    }

    #[test]
    fn test_collector_creation() {
        let config = CollectorConfig::default();
        let collector = MarketDataCollector::new(config);
        assert!(collector.ws_client.is_none());
    }
}
