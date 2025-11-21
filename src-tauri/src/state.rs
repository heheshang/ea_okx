//! Application state

use crate::services::{StrategyService, StrategyMonitorService, StrategyExecutionEngine};
use std::sync::Arc;

/// Application state shared across all commands
#[derive(Clone)]
pub struct AppState {
    pub strategy_service: Arc<StrategyService>,
    pub strategy_monitor: Arc<StrategyMonitorService>,
    pub execution_engine: Arc<StrategyExecutionEngine>,
}

impl AppState {
    /// Creates a new application state
    pub fn new() -> Self {
        let strategy_monitor = Arc::new(StrategyMonitorService::new());
        let strategy_service = Arc::new(StrategyService::with_monitor(strategy_monitor.clone()));
        let execution_engine = Arc::new(StrategyExecutionEngine::with_monitor(strategy_monitor.clone()));

        Self {
            strategy_service,
            strategy_monitor,
            execution_engine,
        }
    }

    /// Initializes the application state (should be called from within a Tokio context)
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize default strategies
        self.strategy_service.initialize_default_strategies().await?;
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}