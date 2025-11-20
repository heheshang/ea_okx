//! Core strategy trait definitions

use crate::error::Result;
use crate::metrics::PerformanceMetrics;
use crate::signal::Signal;
use async_trait::async_trait;
use ea_okx_core::models::Order;
use ea_okx_core::types::Symbol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Market data event
#[derive(Debug, Clone)]
pub enum MarketDataEvent {
    Ticker {
        symbol: Symbol,
        price: rust_decimal::Decimal,
        volume: rust_decimal::Decimal,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Candle {
        symbol: Symbol,
        open: rust_decimal::Decimal,
        high: rust_decimal::Decimal,
        low: rust_decimal::Decimal,
        close: rust_decimal::Decimal,
        volume: rust_decimal::Decimal,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Trade {
        symbol: Symbol,
        price: rust_decimal::Decimal,
        quantity: rust_decimal::Decimal,
        side: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategy_id: Uuid,
    pub name: String,
    pub version: String,
    pub symbols: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub risk_limits: RiskLimits,
}

/// Risk limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size: rust_decimal::Decimal,
    pub max_leverage: rust_decimal::Decimal,
    pub stop_loss_pct: rust_decimal::Decimal,
    pub take_profit_pct: Option<rust_decimal::Decimal>,
}

/// Core strategy trait
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Initialize strategy with configuration
    async fn initialize(&mut self, config: StrategyConfig) -> Result<()>;

    /// Process incoming market data
    async fn on_market_data(&mut self, event: MarketDataEvent) -> Result<()>;

    /// Generate trading signal
    async fn generate_signal(&self) -> Result<Signal>;

    /// Handle order fill notification
    async fn on_order_fill(&mut self, order: &Order) -> Result<()>;

    /// Handle order rejection
    async fn on_order_reject(&mut self, order: &Order, reason: &str) -> Result<()>;

    /// Get current performance metrics
    fn get_metrics(&self) -> PerformanceMetrics;

    /// Serialize strategy state for hot-reload
    fn serialize_state(&self) -> Result<serde_json::Value>;

    /// Deserialize and restore strategy state
    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<()>;

    /// Clean shutdown
    async fn shutdown(&mut self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_config_creation() {
        let config = StrategyConfig {
            strategy_id: Uuid::new_v4(),
            name: "Test Strategy".to_string(),
            version: "1.0.0".to_string(),
            symbols: vec!["BTC-USDT".to_string()],
            parameters: HashMap::new(),
            risk_limits: RiskLimits {
                max_position_size: rust_decimal::Decimal::new(1000, 0),
                max_leverage: rust_decimal::Decimal::new(3, 0),
                stop_loss_pct: rust_decimal::Decimal::new(2, 2),
                take_profit_pct: Some(rust_decimal::Decimal::new(5, 2)),
            },
        };

        assert_eq!(config.name, "Test Strategy");
        assert_eq!(config.version, "1.0.0");
    }
}
