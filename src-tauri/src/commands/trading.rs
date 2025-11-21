use crate::state::AppState;
use crate::services::strategy_execution::{
    ExecutionRequest, ExecutionSignal, SignalType,
    TimeInForce,
};
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::ToPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub strategy_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub price: Option<f64>,
    pub time_in_force: Option<String>,
    pub reduce_only: Option<bool>,
    pub post_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRequest {
    pub strategy_id: String,
    pub symbol: String,
    pub signal_type: String,
    pub side: Option<String>,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub confidence: f64,
    pub metadata: Option<serde_json::Value>,
}

/// Place a new order
#[tauri::command]
pub async fn place_order(
    request: PlaceOrderRequest,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Placing order: {:?}", request);

    let strategy_id = uuid::Uuid::parse_str(&request.strategy_id)
        .map_err(|e| format!("Invalid strategy ID: {}", e))?;

    let symbol = ea_okx_core::types::Symbol::new(&request.symbol)
        .map_err(|e| format!("Invalid symbol: {}", e))?;

    let side = request.side.parse::<ea_okx_core::models::order::OrderSide>()
        .map_err(|e| format!("Invalid side: {}", e))?;

    let order_type = request.order_type.parse::<ea_okx_core::models::order::OrderType>()
        .map_err(|e| format!("Invalid order type: {}", e))?;

    let quantity = ea_okx_core::types::Quantity::new(
        rust_decimal::Decimal::from_f64_retain(request.quantity)
            .ok_or_else(|| "Invalid quantity".to_string())?
    ).map_err(|e| format!("Invalid quantity: {}", e))?;

    let price = request.price.map(|p| {
        ea_okx_core::types::Price::new(
            rust_decimal::Decimal::from_f64_retain(p)
                .ok_or_else(|| "Invalid price".to_string())?
        ).map_err(|e| format!("Invalid price: {}", e))
    }).transpose()?;

    let time_in_force = match request.time_in_force.as_deref().unwrap_or("GTC") {
        "GTC" => TimeInForce::GoodTillCancel,
        "IOC" => TimeInForce::ImmediateOrCancel,
        "FOK" => TimeInForce::FillOrKill,
        _ => return Err("Invalid time in force".to_string()),
    };

    let execution_request = ExecutionRequest {
        id: uuid::Uuid::new_v4(),
        strategy_id,
        symbol,
        side,
        order_type,
        quantity,
        price,
        time_in_force,
        reduce_only: request.reduce_only.unwrap_or(false),
        post_only: request.post_only.unwrap_or(false),
    };

    match state.execution_engine.execute_order(execution_request).await {
        Ok(result) => {
            let response = serde_json::json!({
                "success": result.success,
                "request_id": result.request_id.to_string(),
                "order": result.order,
                "trade": result.trade,
                "error": result.error,
                "latency_ms": result.latency_ms
            });
            Ok(response)
        }
        Err(e) => Err(format!("Order execution failed: {}", e))
    }
}

/// Cancel an order
#[tauri::command]
pub async fn cancel_order(
    order_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Cancelling order: {}", order_id);

    match state.execution_engine.cancel_order(&order_id).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to cancel order: {}", e))
    }
}

/// Get all open orders
#[tauri::command]
pub async fn get_open_orders(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ea_okx_core::models::order::Order>, String> {
    log::info!("Fetching open orders");

    let orders = state.execution_engine.get_orders().await;
    let open_orders: Vec<_> = orders.into_iter()
        .filter(|order| order.is_active())
        .collect();

    Ok(open_orders)
}

/// Get order history
#[tauri::command]
pub async fn get_order_history(
    limit: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ea_okx_core::models::order::Order>, String> {
    log::info!("Fetching order history (limit: {:?})", limit);

    let mut orders = state.execution_engine.get_orders().await;
    orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    match limit {
        Some(limit) => Ok(orders.into_iter().take(limit).collect()),
        None => Ok(orders),
    }
}

/// Get current positions
#[tauri::command]
pub async fn get_positions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ea_okx_core::models::position::Position>, String> {
    log::info!("Fetching positions");

    Ok(state.execution_engine.get_positions().await)
}

/// Get trade history
#[tauri::command]
pub async fn get_trades(
    limit: Option<usize>,
    strategy_id: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ea_okx_core::models::trade::Trade>, String> {
    log::info!("Fetching trades (limit: {:?}, strategy_id: {:?})", limit, strategy_id);

    let trades = state.execution_engine.get_trades(limit).await;

    match strategy_id {
        Some(strategy_id) => {
            let filtered_trades: Vec<_> = trades.into_iter()
                .filter(|trade| trade.strategy_id.to_string() == strategy_id)
                .collect();
            Ok(filtered_trades)
        }
        None => Ok(trades),
    }
}

/// Submit execution signal from strategy
#[tauri::command]
pub async fn submit_execution_signal(
    request: SignalRequest,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Submitting execution signal: {:?}", request);

    let strategy_id = uuid::Uuid::parse_str(&request.strategy_id)
        .map_err(|e| format!("Invalid strategy ID: {}", e))?;

    let symbol = ea_okx_core::types::Symbol::new(&request.symbol)
        .map_err(|e| format!("Invalid symbol: {}", e))?;

    let signal_type = match request.signal_type.as_str() {
        "open" => SignalType::Open,
        "close" => SignalType::Close,
        "partial_close" => SignalType::PartialClose,
        "modify" => SignalType::Modify,
        "stop_loss" => SignalType::StopLoss,
        "take_profit" => SignalType::TakeProfit,
        "risk_management" => SignalType::RiskManagement,
        _ => return Err("Invalid signal type".to_string()),
    };

    let side = request.side.map(|s| {
        s.parse::<ea_okx_core::models::order::OrderSide>()
            .map_err(|e| format!("Invalid side: {}", e))
    }).transpose()?;

    let quantity = ea_okx_core::types::Quantity::new(
        rust_decimal::Decimal::from_f64_retain(request.quantity)
            .ok_or_else(|| "Invalid quantity".to_string())?
    ).map_err(|e| format!("Invalid quantity: {}", e))?;

    let price = request.price.map(|p| {
        ea_okx_core::types::Price::new(
            rust_decimal::Decimal::from_f64_retain(p)
                .ok_or_else(|| "Invalid price".to_string())?
        ).map_err(|e| format!("Invalid price: {}", e))
    }).transpose()?;

    let stop_loss = request.stop_loss.map(|p| {
        ea_okx_core::types::Price::new(
            rust_decimal::Decimal::from_f64_retain(p)
                .ok_or_else(|| "Invalid stop loss".to_string())?
        ).map_err(|e| format!("Invalid stop loss: {}", e))
    }).transpose()?;

    let take_profit = request.take_profit.map(|p| {
        ea_okx_core::types::Price::new(
            rust_decimal::Decimal::from_f64_retain(p)
                .ok_or_else(|| "Invalid take profit".to_string())?
        ).map_err(|e| format!("Invalid take profit: {}", e))
    }).transpose()?;

    let signal = ExecutionSignal {
        strategy_id,
        symbol,
        signal_type,
        side,
        quantity,
        price,
        stop_loss,
        take_profit,
        confidence: request.confidence,
        metadata: request.metadata.unwrap_or(serde_json::Value::Null),
    };

    match state.execution_engine.submit_signal(signal).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to submit signal: {}", e))
    }
}

/// Close position
#[tauri::command]
pub async fn close_position(
    symbol: String,
    strategy_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Closing position: {} for strategy: {}", symbol, strategy_id);

    let strategy_uuid = uuid::Uuid::parse_str(&strategy_id)
        .map_err(|e| format!("Invalid strategy ID: {}", e))?;

    let symbol_type = ea_okx_core::types::Symbol::new(&symbol)
        .map_err(|e| format!("Invalid symbol: {}", e))?;

    // Create close signal
    let signal = ExecutionSignal {
        strategy_id: strategy_uuid,
        symbol: symbol_type,
        signal_type: SignalType::Close,
        side: None, // Will be determined by position
        quantity: ea_okx_core::types::Quantity::new(rust_decimal::Decimal::MAX)
            .map_err(|e| format!("Invalid quantity: {}", e))?,
        price: None,
        stop_loss: None,
        take_profit: None,
        confidence: 1.0,
        metadata: serde_json::json!({"action": "close_all"}),
    };

    match state.execution_engine.submit_signal(signal).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to close position: {}", e))
    }
}

/// Get strategy execution statistics
#[tauri::command]
pub async fn get_strategy_execution_stats(
    strategy_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Getting execution stats for strategy: {}", strategy_id);

    match state.execution_engine.get_strategy_stats(&strategy_id).await {
        Ok(stats) => Ok(stats),
        Err(e) => Err(format!("Failed to get stats: {}", e))
    }
}

/// Get account balance information
#[tauri::command]
pub async fn get_account_balance(
    _state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Fetching account balance");

    // Mock account balance - in real implementation, this would query OKX API
    let balance = serde_json::json!({
        "total_equity": 50000.0,
        "available_balance": 45000.0,
        "used_balance": 5000.0,
        "unrealized_pnl": 250.0,
        "margin_ratio": 0.15,
        "maintenance_margin": 750.0,
        "currency": "USDT",
        "last_updated": chrono::Utc::now().to_rfc3339()
    });

    Ok(balance)
}

/// Get trading fees information
#[tauri::command]
pub async fn get_trading_fees(
    symbol: Option<String>,
    _state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Fetching trading fees for symbol: {:?}", symbol);

    // Mock trading fees - in real implementation, this would query OKX API
    let fees = serde_json::json!({
        "maker_fee": 0.0008,  // 0.08%
        "taker_fee": 0.001,   // 0.10%
        "symbol": symbol.unwrap_or_else(|| "BTC-USDT".to_string()),
        "vip_level": 1,
        "fee_currency": "USDT"
    });

    Ok(fees)
}

/// Get market depth (order book)
#[tauri::command]
pub async fn get_order_book(
    symbol: String,
    depth: Option<usize>,
    _state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Fetching order book for: {} (depth: {:?})", symbol, depth);

    // Mock order book data - in real implementation, this would query OKX API
    let order_book = serde_json::json!({
        "symbol": symbol,
        "bids": [
            [44950.0, 0.5],
            [44900.0, 1.2],
            [44850.0, 0.8],
            [44800.0, 2.1],
            [44750.0, 1.5]
        ],
        "asks": [
            [45000.0, 0.3],
            [45050.0, 0.9],
            [45100.0, 1.1],
            [45150.0, 0.7],
            [45200.0, 1.8]
        ],
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(order_book)
}

/// Get 24h trading statistics
#[tauri::command]
pub async fn get_24h_stats(
    symbol: String,
    _state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Fetching 24h stats for: {}", symbol);

    // Mock 24h statistics - in real implementation, this would query OKX API
    let stats = serde_json::json!({
        "symbol": symbol,
        "price_change": 450.0,
        "price_change_percent": 1.01,
        "weighted_avg_price": 44775.0,
        "open_price": 44550.0,
        "high_price": 45200.0,
        "low_price": 44100.0,
        "last_price": 45000.0,
        "volume": 1250.8,
        "quote_volume": 56062500.0,
        "open_time": chrono::Utc::now().checked_sub_signed(chrono::Duration::hours(24)).unwrap().to_rfc3339(),
        "close_time": chrono::Utc::now().to_rfc3339(),
        "count": 45820
    });

    Ok(stats)
}

/// Batch cancel orders
#[tauri::command]
pub async fn cancel_all_orders(
    symbol: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    log::info!("Cancelling all orders for symbol: {:?}", symbol);

    let orders = state.execution_engine.get_orders().await;
    let mut cancelled_count = 0;

    for order in orders {
        if order.is_active() {
            if symbol.is_none() || order.symbol.as_str() == symbol.as_ref().unwrap() {
                if let Err(e) = state.execution_engine.cancel_order(&order.id.to_string()).await {
                    log::error!("Failed to cancel order {}: {}", order.id, e);
                } else {
                    cancelled_count += 1;
                }
            }
        }
    }

    Ok(cancelled_count)
}

/// Get position risk information
#[tauri::command]
pub async fn get_position_risk(
    symbol: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    log::info!("Fetching position risk for symbol: {:?}", symbol);

    let positions = state.execution_engine.get_positions().await;
    let mut total_unrealized_pnl = 0.0;
    let mut total_margin_used = 0.0;
    let mut position_count = 0;

    let positions_data: Vec<serde_json::Value> = positions
        .into_iter()
        .filter(|pos| symbol.is_none() || pos.symbol.as_str() == symbol.as_ref().unwrap())
        .map(|pos| {
            let unrealized_pnl = pos.unrealized_pnl.to_f64().unwrap_or(0.0);
            let margin_used = pos.quantity.as_decimal().to_f64().unwrap_or(0.0) *
                             pos.avg_entry_price.as_decimal().to_f64().unwrap_or(0.0) * 0.1; // 10% margin assumption

            total_unrealized_pnl += unrealized_pnl;
            total_margin_used += margin_used;
            position_count += 1;

            serde_json::json!({
                "symbol": pos.symbol.as_str(),
                "side": format!("{:?}", pos.side),
                "size": pos.quantity.as_decimal().to_string(),
                "entry_price": pos.avg_entry_price.as_decimal().to_string(),
                "mark_price": (pos.avg_entry_price.as_decimal().to_f64().unwrap_or(0.0) * 1.001).to_string(), // Mock mark price
                "unrealized_pnl": unrealized_pnl,
                "margin_used": margin_used,
                "leverage": 1.0,
                "last_updated": pos.last_updated.to_rfc3339()
            })
        })
        .collect();

    Ok(serde_json::json!({
        "positions": positions_data,
        "total_unrealized_pnl": total_unrealized_pnl,
        "total_margin_used": total_margin_used,
        "position_count": position_count,
        "risk_metrics": {
            "portfolio_var": 2500.0,  // Mock VaR
            "max_drawdown": 500.0,
            "sharpe_ratio": 1.25,
            "risk_score": 3.5
        }
    }))
}
