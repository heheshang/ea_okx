use crate::state::AppState;
use crate::services::strategy_monitor::StrategyUpdateEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Emitter;

/// WebSocket subscription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSubscription {
    pub strategy_ids: Vec<String>,
    pub event_types: Vec<String>,
}


/// Subscribe to strategy updates via WebSocket
#[tauri::command]
pub async fn subscribe_strategy_updates(
    subscription: WebSocketSubscription,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Client subscribing to strategy updates: {:?}", subscription);

    match state.strategy_monitor.subscribe_client(
        subscription.strategy_ids,
        subscription.event_types,
    ).await {
        Ok(mut receiver) => {
            let client_id = uuid::Uuid::new_v4().to_string();

            // Start a task to handle messages from the strategy monitor
            let app_handle_clone = app_handle.clone();

            tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    // Emit event to frontend
                    let event_name = match message.event {
                        StrategyUpdateEvent::StatusChanged { .. } => "strategy:status-changed",
                        StrategyUpdateEvent::TradeExecuted { .. } => "strategy:trade-executed",
                        StrategyUpdateEvent::MetricsUpdated { .. } => "strategy:metrics-updated",
                        StrategyUpdateEvent::SignalGenerated { .. } => "strategy:signal-generated",
                        StrategyUpdateEvent::Error { .. } => "strategy:error",
                        StrategyUpdateEvent::PositionUpdate { .. } => "strategy:position-update",
                    };

                    if let Err(e) = app_handle_clone.emit(&event_name, &message) {
                        log::error!("Failed to emit WebSocket event: {}", e);
                        break;
                    }
                }
            });

            Ok(client_id)
        }
        Err(e) => Err(format!("Failed to subscribe: {}", e))
    }
}

/// Unsubscribe from strategy updates
#[tauri::command]
pub async fn unsubscribe_strategy_updates(
    client_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Client unsubscribing: {}", client_id);

    match state.strategy_monitor.unsubscribe_client(&client_id).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to unsubscribe: {}", e))
    }
}

/// Get connected clients count
#[tauri::command]
pub async fn get_connected_clients_count(
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    Ok(state.strategy_monitor.get_clients_count().await)
}

/// Get real-time strategy statistics
#[tauri::command]
pub async fn get_realtime_strategy_stats(
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    Ok(state.strategy_monitor.get_strategy_stats().await)
}

/// Simulate a strategy signal (for testing)
#[tauri::command]
pub async fn simulate_strategy_signal(
    strategy_id: String,
    signal_type: String,
    symbol: String,
    price: f64,
    confidence: f64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Simulating signal for strategy {}: {:?} @ {}", strategy_id, signal_type, price);

    match state.strategy_monitor.emit_signal_generated(
        strategy_id,
        signal_type,
        symbol,
        price,
        confidence,
    ).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to emit signal: {}", e))
    }
}

/// Simulate a trade execution (for testing)
#[tauri::command]
pub async fn simulate_trade_execution(
    strategy_id: String,
    symbol: String,
    side: String,
    amount: f64,
    price: f64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let trade_id = uuid::Uuid::new_v4().to_string();

    log::info!("Simulating trade for strategy {}: {} {} @ {}", strategy_id, side, amount, price);

    match state.strategy_monitor.emit_trade_executed(
        strategy_id,
        trade_id,
        symbol,
        side,
        amount,
        price,
    ).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to emit trade: {}", e))
    }
}

/// Simulate a strategy error (for testing)
#[tauri::command]
pub async fn simulate_strategy_error(
    strategy_id: String,
    error_message: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::warn!("Simulating error for strategy {}: {}", strategy_id, error_message);

    match state.strategy_monitor.emit_error(strategy_id, error_message).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to emit error: {}", e))
    }
}

/// Simulate a position update (for testing)
#[tauri::command]
pub async fn simulate_position_update(
    strategy_id: String,
    symbol: String,
    side: String,
    size: f64,
    entry_price: Option<f64>,
    exit_price: Option<f64>,
    pnl: Option<f64>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Simulating position update for strategy {}: {} {} {}", strategy_id, symbol, side, size);

    match state.strategy_monitor.emit_position_update(
        strategy_id,
        symbol,
        side,
        size,
        entry_price,
        exit_price,
        pnl,
    ).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to emit position update: {}", e))
    }
}

/// Update strategy metrics (for real-time updates)
#[tauri::command]
pub async fn update_strategy_metrics(
    strategy_id: String,
    metrics: ea_okx_core::models::strategy::StrategyMetrics,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Updating metrics for strategy: {}", strategy_id);

    match state.strategy_monitor.emit_metrics_updated(strategy_id, metrics).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to update metrics: {}", e))
    }
}

/// Get WebSocket connection status
#[tauri::command]
pub async fn get_websocket_status(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let clients_count = state.strategy_monitor.get_clients_count().await;
    let strategy_stats = state.strategy_monitor.get_strategy_stats().await;
    let active_strategies = strategy_stats.values()
        .filter(|s| s["status"] == "active")
        .count();

    Ok(serde_json::json!({
        "connected_clients": clients_count,
        "active_strategies": active_strategies,
        "total_strategies": strategy_stats.len(),
        "status": "connected",
        "last_update": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get real-time market data subscription status
#[tauri::command]
pub async fn get_market_data_status(
    _state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // This would integrate with the OKX WebSocket client
    Ok(serde_json::json!({
        "status": "connected",
        "subscriptions": ["BTC-USDT", "ETH-USDT", "SOL-USDT"],
        "last_update": chrono::Utc::now().to_rfc3339(),
        "latency_ms": 15
    }))
}