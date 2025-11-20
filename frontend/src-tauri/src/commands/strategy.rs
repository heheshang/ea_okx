use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String, // Draft, Validating, Backtesting, PaperTrading, ReadyForLive, Active, Paused, Stopped, Archived
    pub parameters: HashMap<String, serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub total_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}

/// Get all strategies
#[tauri::command]
pub async fn get_strategies() -> Result<Vec<Strategy>, String> {
    // TODO: Integrate with actual strategy service
    log::info!("Fetching all strategies");
    
    // Mock data for now
    Ok(vec![
        Strategy {
            id: "strat_001".to_string(),
            name: "MA Crossover Strategy".to_string(),
            description: "Simple moving average crossover strategy".to_string(),
            status: "Active".to_string(),
            parameters: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        },
    ])
}

/// Get strategy by ID
#[tauri::command]
pub async fn get_strategy(id: String) -> Result<Strategy, String> {
    log::info!("Fetching strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(Strategy {
        id: id.clone(),
        name: "MA Crossover Strategy".to_string(),
        description: "Simple moving average crossover strategy".to_string(),
        status: "Active".to_string(),
        parameters: HashMap::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Create new strategy
#[tauri::command]
pub async fn create_strategy(request: CreateStrategyRequest) -> Result<Strategy, String> {
    log::info!("Creating strategy: {}", request.name);
    
    // TODO: Integrate with actual strategy service
    Ok(Strategy {
        id: format!("strat_{}", uuid::Uuid::new_v4()),
        name: request.name,
        description: request.description,
        status: "Draft".to_string(),
        parameters: request.parameters,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Update strategy
#[tauri::command]
pub async fn update_strategy(id: String, request: CreateStrategyRequest) -> Result<Strategy, String> {
    log::info!("Updating strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(Strategy {
        id,
        name: request.name,
        description: request.description,
        status: "Draft".to_string(),
        parameters: request.parameters,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Delete strategy
#[tauri::command]
pub async fn delete_strategy(id: String) -> Result<(), String> {
    log::info!("Deleting strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(())
}

/// Start strategy
#[tauri::command]
pub async fn start_strategy(id: String) -> Result<(), String> {
    log::info!("Starting strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(())
}

/// Stop strategy
#[tauri::command]
pub async fn stop_strategy(id: String) -> Result<(), String> {
    log::info!("Stopping strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(())
}

/// Pause strategy
#[tauri::command]
pub async fn pause_strategy(id: String) -> Result<(), String> {
    log::info!("Pausing strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(())
}

/// Get strategy metrics
#[tauri::command]
pub async fn get_strategy_metrics(id: String) -> Result<StrategyMetrics, String> {
    log::info!("Fetching metrics for strategy: {}", id);
    
    // TODO: Integrate with actual strategy service
    Ok(StrategyMetrics {
        total_trades: 150,
        win_rate: 0.65,
        total_pnl: 12500.50,
        sharpe_ratio: 1.85,
        max_drawdown: 0.12,
    })
}
