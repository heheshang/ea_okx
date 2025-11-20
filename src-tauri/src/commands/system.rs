use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_latency: f64,
    pub active_strategies: usize,
    pub total_orders: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: String, // INFO, WARNING, ERROR, CRITICAL
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    pub strategy_id: String,
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: usize,
}

/// Get system metrics
#[tauri::command]
pub async fn get_system_metrics() -> Result<SystemMetrics, String> {
    log::info!("Fetching system metrics");
    Ok(SystemMetrics {
        cpu_usage: 35.5,
        memory_usage: 62.3,
        network_latency: 45.2,
        active_strategies: 3,
        total_orders: 150,
    })
}

/// Get alerts
#[tauri::command]
pub async fn get_alerts(limit: Option<usize>) -> Result<Vec<Alert>, String> {
    log::info!("Fetching alerts (limit: {:?})", limit);
    Ok(vec![])
}

/// Run backtest
#[tauri::command]
pub async fn run_backtest(request: BacktestRequest) -> Result<String, String> {
    log::info!("Starting backtest: {:?}", request);
    // TODO: Integrate with backtest engine
    // Return backtest job ID
    Ok(format!("backtest_{}", uuid::Uuid::new_v4()))
}

/// Get backtest results
#[tauri::command]
pub async fn get_backtest_results(backtest_id: String) -> Result<BacktestResult, String> {
    log::info!("Fetching backtest results: {}", backtest_id);
    // TODO: Integrate with backtest engine
    Ok(BacktestResult {
        total_return: 0.45,
        sharpe_ratio: 1.85,
        max_drawdown: 0.12,
        win_rate: 0.65,
        total_trades: 150,
    })
}
