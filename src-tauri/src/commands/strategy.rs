use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ea_okx_core::models::strategy as strategy_models;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub symbols: Vec<String>,
    pub allocated_capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStrategyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub symbols: Option<Vec<String>>,
    pub allocated_capital: Option<f64>,
}

/// Get all strategies
#[tauri::command]
pub async fn get_strategies(
    state: tauri::State<'_, AppState>,
    _filter: Option<strategy_models::StrategyFilter>,
) -> Result<strategy_models::StrategyResponse<strategy_models::StrategyListResponse>, String> {
    log::info!("Fetching all strategies");

    match state.strategy_service.get_strategies().await {
        Ok(strategies) => {
            let total = strategies.len();
            let response = strategy_models::StrategyResponse {
                success: true,
                data: Some(strategy_models::StrategyListResponse {
                    strategies,
                    total,
                }),
                error: None,
            };
            Ok(response)
        }
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get strategy by ID
#[tauri::command]
pub async fn get_strategy(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<strategy_models::Strategy>, String> {
    log::info!("Fetching strategy: {}", id);

    match state.strategy_service.get_strategy(&id).await {
        Ok(strategy) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(strategy),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Create new strategy
#[tauri::command]
pub async fn create_strategy(
    request: CreateStrategyRequest,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<strategy_models::Strategy>, String> {
    log::info!("Creating strategy: {}", request.name);

    match state.strategy_service.create_strategy(
        request.name,
        request.description,
        "custom".to_string(), // Default strategy type
        serde_json::to_value(request.parameters).unwrap_or_default(),
        request.symbols,
        request.allocated_capital,
        "default-user".to_string(), // Default user ID
    ).await {
        Ok(strategy) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(strategy),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Update strategy
#[tauri::command]
pub async fn update_strategy(
    id: String,
    request: UpdateStrategyRequest,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<strategy_models::Strategy>, String> {
    log::info!("Updating strategy: {}", id);

    let parameters = request.parameters.map(|p| serde_json::to_value(p).unwrap_or_default());

    match state.strategy_service.update_strategy(
        &id,
        request.name,
        request.description,
        parameters,
        request.symbols,
        request.allocated_capital,
    ).await {
        Ok(strategy) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(strategy),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Delete strategy
#[tauri::command]
pub async fn delete_strategy(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<()>, String> {
    log::info!("Deleting strategy: {}", id);

    match state.strategy_service.delete_strategy(&id).await {
        Ok(_) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(()),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Start strategy
#[tauri::command]
pub async fn start_strategy(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<()>, String> {
    log::info!("Starting strategy: {}", id);

    match state.strategy_service.start_strategy(&id).await {
        Ok(_) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(()),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Stop strategy
#[tauri::command]
pub async fn stop_strategy(
    id: String,
    force: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<()>, String> {
    log::info!("Stopping strategy: {}", id);

    match state.strategy_service.stop_strategy(&id, force.unwrap_or(false)).await {
        Ok(_) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(()),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Pause strategy
#[tauri::command]
pub async fn pause_strategy(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<()>, String> {
    log::info!("Pausing strategy: {}", id);

    match state.strategy_service.pause_strategy(&id).await {
        Ok(_) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(()),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get strategy metrics
#[tauri::command]
pub async fn get_strategy_metrics(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<serde_json::Value>, String> {
    log::info!("Fetching metrics for strategy: {}", id);

    match state.strategy_service.get_strategy_metrics(&id).await {
        Ok(metrics) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(metrics),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Duplicate strategy
#[tauri::command]
pub async fn duplicate_strategy(
    id: String,
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<strategy_models::StrategyResponse<strategy_models::Strategy>, String> {
    log::info!("Duplicating strategy: {} as {}", id, name);

    match state.strategy_service.duplicate_strategy(&id, name).await {
        Ok(strategy) => Ok(strategy_models::StrategyResponse {
            success: true,
            data: Some(strategy),
            error: None,
        }),
        Err(e) => Ok(strategy_models::StrategyResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}
