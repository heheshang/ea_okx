//! Strategy management service

use chrono::Utc;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Import from core crate
use ea_okx_core::{
    error::{Error, Result},
    models::strategy::{Strategy, StrategyConfig, StrategyStatus},
};

/// Strategy service for managing trading strategies
#[derive(Clone)]
pub struct StrategyService {
    strategies: Arc<RwLock<HashMap<String, Strategy>>>,
    monitor: Option<Arc<super::StrategyMonitorService>>,
}

impl StrategyService {
    /// Creates a new strategy service
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            monitor: None,
        }
    }

    /// Creates a new strategy service with monitor integration
    pub fn with_monitor(monitor: Arc<super::StrategyMonitorService>) -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            monitor: Some(monitor),
        }
    }

    /// Creates a new strategy
    pub async fn create_strategy(
        &self,
        name: String,
        description: String,
        strategy_type: String,
        parameters: JsonValue,
        symbols: Vec<String>,
        allocated_capital: f64,
        created_by: String,
    ) -> Result<Strategy> {
        let config = StrategyConfig::new(
            parameters.clone(),
            symbols.into_iter().map(|s| ea_okx_core::types::Symbol::new(&s).unwrap()).collect(),
            rust_decimal::Decimal::from_str_exact(&allocated_capital.to_string()).unwrap_or_default(),
        );

        let strategy_name = name.clone();
        let strategy = Strategy::new(
            name,
            strategy_type,
            "1.0.0".to_string(),
            config,
            Uuid::parse_str(&created_by).unwrap_or_else(|_| Uuid::new_v4()),
        )?;

        let mut strategy = strategy;
        strategy.description = Some(description);

        let id = strategy.id.to_string();
        let mut strategies = self.strategies.write().await;
        strategies.insert(id.clone(), strategy.clone());

        // Notify monitor of strategy creation
        if let Some(monitor) = &self.monitor {
            let _ = monitor.update_strategy(strategy.clone()).await;
        }

        log::info!("Created strategy: {} ({})", strategy_name, id);
        Ok(strategy)
    }

    /// Gets all strategies
    pub async fn get_strategies(&self) -> Result<Vec<Strategy>> {
        let strategies = self.strategies.read().await;
        let mut result: Vec<Strategy> = strategies.values().cloned().collect();
        result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(result)
    }

    /// Gets a strategy by ID
    pub async fn get_strategy(&self, id: &str) -> Result<Strategy> {
        let strategies = self.strategies.read().await;
        strategies
            .get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Strategy not found: {}", id)))
    }

    /// Updates a strategy
    pub async fn update_strategy(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        parameters: Option<JsonValue>,
        symbols: Option<Vec<String>>,
        allocated_capital: Option<f64>,
    ) -> Result<Strategy> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(id).ok_or_else(|| {
            Error::NotFound(format!("Strategy not found: {}", id))
        })?;

        if let Some(name) = name {
            strategy.name = name;
        }

        if let Some(description) = description {
            strategy.description = Some(description);
        }

        if let Some(parameters) = parameters {
            strategy.config.parameters = parameters;
        }

        if let Some(symbols) = symbols {
            strategy.config.symbols = symbols.into_iter().map(|s| ea_okx_core::types::Symbol::new(&s).unwrap()).collect();
        }

        if let Some(allocated_capital) = allocated_capital {
            strategy.config.allocated_capital = rust_decimal::Decimal::from_str_exact(&allocated_capital.to_string()).unwrap_or_default();
        }

        strategy.updated_at = Utc::now();
        strategy.status = StrategyStatus::Draft; // Reset to draft after update

        let updated_strategy = strategy.clone();
        log::info!("Updated strategy: {} ({})", updated_strategy.name, id);
        Ok(updated_strategy)
    }

    /// Deletes a strategy
    pub async fn delete_strategy(&self, id: &str) -> Result<()> {
        let mut strategies = self.strategies.write().await;
        if strategies.remove(id).is_none() {
            return Err(Error::NotFound(format!("Strategy not found: {}", id)));
        }

        log::info!("Deleted strategy: {}", id);
        Ok(())
    }

    /// Starts a strategy
    pub async fn start_strategy(&self, id: &str) -> Result<()> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(id).ok_or_else(|| {
            Error::NotFound(format!("Strategy not found: {}", id))
        })?;

        match strategy.status {
            StrategyStatus::Draft | StrategyStatus::Paused | StrategyStatus::Stopped => {
                strategy.status = StrategyStatus::Active;
                strategy.updated_at = Utc::now();
                strategy.last_active_at = Some(Utc::now());

                log::info!("Started strategy: {} ({})", strategy.name, id);
                Ok(())
            }
            StrategyStatus::Active => {
                Err(Error::ValidationError("Strategy is already active".to_string()))
            }
            _ => {
                Err(Error::ValidationError(
                    "Cannot start strategy in current state".to_string()
                ))
            }
        }
    }

    /// Stops a strategy
    pub async fn stop_strategy(&self, id: &str, force: bool) -> Result<()> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(id).ok_or_else(|| {
            Error::NotFound(format!("Strategy not found: {}", id))
        })?;

        match strategy.status {
            StrategyStatus::Active | StrategyStatus::PaperTrading => {
                strategy.status = StrategyStatus::Stopped;
                strategy.updated_at = Utc::now();

                log::info!("Stopped strategy: {} ({})", strategy.name, id);
                Ok(())
            }
            StrategyStatus::Stopped => {
                Err(Error::ValidationError("Strategy is already stopped".to_string()))
            }
            _ if force => {
                strategy.status = StrategyStatus::Stopped;
                strategy.updated_at = Utc::now();

                log::info!("Force stopped strategy: {} ({})", strategy.name, id);
                Ok(())
            }
            _ => {
                Err(Error::ValidationError(
                    "Cannot stop strategy in current state".to_string()
                ))
            }
        }
    }

    /// Pauses a strategy
    pub async fn pause_strategy(&self, id: &str) -> Result<()> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(id).ok_or_else(|| {
            Error::NotFound(format!("Strategy not found: {}", id))
        })?;

        match strategy.status {
            StrategyStatus::Active => {
                strategy.status = StrategyStatus::Paused;
                strategy.updated_at = Utc::now();

                log::info!("Paused strategy: {} ({})", strategy.name, id);
                Ok(())
            }
            StrategyStatus::Paused => {
                Err(Error::ValidationError("Strategy is already paused".to_string()))
            }
            _ => {
                Err(Error::ValidationError(
                    "Cannot pause strategy in current state".to_string()
                ))
            }
        }
    }

    /// Gets strategy metrics
    pub async fn get_strategy_metrics(&self, id: &str) -> Result<serde_json::Value> {
        let strategies = self.strategies.read().await;
        let _strategy = strategies.get(id).ok_or_else(|| {
            Error::NotFound(format!("Strategy not found: {}", id))
        })?;

        // Return mock metrics for now
        Ok(serde_json::json!({
            "total_trades": 0,
            "win_rate": 0.0,
            "total_pnl": 0.0,
            "sharpe_ratio": 0.0,
            "max_drawdown": 0.0,
            "total_return": 0.0,
            "profit_factor": 0.0,
            "average_win": 0.0,
            "average_loss": 0.0,
            "largest_win": 0.0,
            "largest_loss": 0.0,
        }))
    }

  
    /// Duplicates a strategy
    pub async fn duplicate_strategy(&self, id: &str, new_name: String) -> Result<Strategy> {
        let strategies = self.strategies.read().await;
        let strategy = strategies.get(id).ok_or_else(|| {
            Error::NotFound(format!("Strategy not found: {}", id))
        })?;

        let new_id = Uuid::new_v4();
        let now = Utc::now();
        let original_name = strategy.name.clone();
        let new_name_clone = new_name.clone();

        let mut new_strategy = strategy.clone();
        new_strategy.id = new_id;
        new_strategy.name = new_name;
        new_strategy.status = StrategyStatus::Draft;
        new_strategy.created_at = now;
        new_strategy.updated_at = now;
        new_strategy.last_active_at = None;

        drop(strategies); // Release read lock before acquiring write lock

        let mut strategies = self.strategies.write().await;
        strategies.insert(new_id.to_string(), new_strategy.clone());

        log::info!("Duplicated strategy: {} -> {}", original_name, new_name_clone);
        Ok(new_strategy)
    }

    /// Initializes default strategies
    pub async fn initialize_default_strategies(&self) -> Result<()> {
        let default_user_id = Uuid::new_v4().to_string();

        // Create some default strategy templates
        let ma_crossover = self.create_strategy(
            "MA Crossover BTC".to_string(),
            "Moving average crossover strategy for Bitcoin".to_string(),
            "ma_crossover".to_string(),
            serde_json::json!({
                "short_period": 20,
                "long_period": 50,
                "ma_type": "EMA",
                "position_size": 0.2
            }),
            vec!["BTC-USDT".to_string()],
            10000.0,
            default_user_id.clone(),
        ).await?;

        let grid_trading = self.create_strategy(
            "Grid Trading ETH".to_string(),
            "Grid trading strategy for Ethereum".to_string(),
            "grid_trading".to_string(),
            serde_json::json!({
                "grid_levels": 10,
                "price_range": 5.0,
                "order_size": 100.0,
                "grid_type": "arithmetic"
            }),
            vec!["ETH-USDT".to_string()],
            15000.0,
            default_user_id.clone(),
        ).await?;

        let rsi_strategy = self.create_strategy(
            "RSI Strategy SOL".to_string(),
            "RSI mean reversion strategy for Solana".to_string(),
            "rsi_strategy".to_string(),
            serde_json::json!({
                "rsi_period": 14,
                "oversold": 30,
                "overbought": 70,
                "position_size": 0.4
            }),
            vec!["SOL-USDT".to_string()],
            8000.0,
            default_user_id,
        ).await?;

        log::info!("Initialized default strategies: {}, {}, {}",
                  ma_crossover.name, grid_trading.name, rsi_strategy.name);
        Ok(())
    }
}

impl Default for StrategyService {
    fn default() -> Self {
        Self::new()
    }
}