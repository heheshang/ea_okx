//! Strategy execution service for real-time trading

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use rust_decimal::prelude::ToPrimitive;

use ea_okx_core::{
    error::{Error, Result},
    models::{
        strategy::{Strategy, StrategyStatus},
        order::{Order, OrderSide, OrderType, OrderStatus},
        position::{Position, PositionSide},
        trade::Trade,
    },
    types::{Symbol, Price, Quantity, Decimal},
};

/// Execution signal from strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSignal {
    pub strategy_id: Uuid,
    pub symbol: Symbol,
    pub signal_type: SignalType,
    pub side: Option<OrderSide>,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub stop_loss: Option<Price>,
    pub take_profit: Option<Price>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

/// Types of execution signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// Open new position
    Open,
    /// Close existing position
    Close,
    /// Close position partially
    PartialClose,
    /// Modify position (add/reduce)
    Modify,
    /// Stop loss trigger
    StopLoss,
    /// Take profit trigger
    TakeProfit,
    /// Risk management (emergency close)
    RiskManagement,
}

/// Order execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub time_in_force: TimeInForce,
    pub reduce_only: bool,
    pub post_only: bool,
}

/// Time in force for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    GoodTillCancel,
    ImmediateOrCancel,
    FillOrKill,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub request_id: Uuid,
    pub success: bool,
    pub order: Option<Order>,
    pub trade: Option<Trade>,
    pub error: Option<String>,
    pub latency_ms: i64,
}

/// Strategy execution engine
#[derive(Clone)]
pub struct StrategyExecutionEngine {
    #[allow(dead_code)]
    strategies: Arc<RwLock<HashMap<String, Strategy>>>,
    orders: Arc<RwLock<HashMap<String, Order>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    trades: Arc<RwLock<Vec<Trade>>>,
    signal_tx: mpsc::UnboundedSender<ExecutionSignal>,
    monitor: Option<Arc<super::StrategyMonitorService>>,
}

impl StrategyExecutionEngine {
    /// Creates a new execution engine
    pub fn new() -> Self {
        let (signal_tx, _) = mpsc::unbounded_channel();

        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(RwLock::new(Vec::new())),
            signal_tx,
            monitor: None,
        }
    }

    /// Creates execution engine with monitor integration
    pub fn with_monitor(monitor: Arc<super::StrategyMonitorService>) -> Self {
        let mut engine = Self::new();
        engine.monitor = Some(monitor);
        engine
    }

    /// Submit execution signal from strategy
    pub async fn submit_signal(&self, signal: ExecutionSignal) -> Result<()> {
        if let Err(e) = self.signal_tx.send(signal.clone()) {
            return Err(Error::Internal(e.to_string()));
        }

        log::info!("Submitted signal: {:?} for strategy {:?}",
                  signal.signal_type, signal.strategy_id);
        Ok(())
    }

    /// Execute a single order
    pub async fn execute_order(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        log::info!("Executing order: {:?}", request);

        // Validate request
        self.validate_order_request(&request)?;

        // Create order
        let mut order = Order::new(
            request.strategy_id,
            request.symbol.clone(),
            request.side,
            request.order_type,
            request.quantity,
            request.price,
        );

        // Submit order to OKX (mock implementation for now)
        let okx_order_id = self.submit_to_okx(&order).await?;

        // Update order status
        order.mark_submitted(okx_order_id.clone());

        // Simulate order execution (in real implementation, this would be handled by WebSocket)
        let (execution_result, trade) = if self.simulate_execution(&mut order).await? {
            let trade = self.create_trade_record(&order)?;
            (true, Some(trade))
        } else {
            (false, None)
        };

        // Store order
        let mut orders = self.orders.write().await;
        orders.insert(order.id.to_string(), order.clone());

        // Update positions based on execution
        if execution_result {
            if let Some(ref trade) = trade {
                self.update_positions_from_trade(trade).await?;
            }
        }

        let latency = start_time.elapsed().as_millis() as i64;

        // Emit monitoring events
        if let Some(monitor) = &self.monitor {
            if execution_result {
                if let Some(ref trade) = trade {
                    let _ = monitor.emit_trade_executed(
                        order.strategy_id.to_string(),
                        trade.id.to_string(),
                        trade.symbol.as_str().to_string(),
                        format!("{:?}", trade.side),
                        trade.quantity.as_decimal().to_f64().unwrap_or(0.0),
                        trade.price.as_decimal().to_f64().unwrap_or(0.0),
                    ).await;
                }
            } else {
                let _ = monitor.emit_error(
                    order.strategy_id.to_string(),
                    "Order execution failed".to_string(),
                ).await;
            }
        }

        Ok(ExecutionResult {
            request_id: request.id,
            success: execution_result,
            order: Some(order),
            trade,
            error: None,
            latency_ms: latency,
        })
    }

    /// Process execution signal
    #[allow(dead_code)]
    async fn process_signal(&self, signal: ExecutionSignal) -> Result<()> {
        log::info!("Processing signal: {:?} for strategy {:?}",
                  signal.signal_type, signal.strategy_id);

        // Check if strategy is active
        let strategies = self.strategies.read().await;
        if let Some(strategy) = strategies.get(&signal.strategy_id.to_string()) {
            if strategy.status != StrategyStatus::Active {
                log::warn!("Strategy {} is not active (status: {:?})",
                          signal.strategy_id, strategy.status);
                return Ok(());
            }
        } else {
            log::warn!("Strategy {} not found", signal.strategy_id);
            return Ok(());
        }

        match signal.signal_type {
            SignalType::Open | SignalType::Modify => {
                self.execute_open_signal(signal).await?;
            }
            SignalType::Close | SignalType::PartialClose => {
                self.execute_close_signal(signal).await?;
            }
            SignalType::StopLoss | SignalType::TakeProfit | SignalType::RiskManagement => {
                self.execute_risk_signal(signal).await?;
            }
        }

        Ok(())
    }

    /// Execute open position signal
    #[allow(dead_code)]
    async fn execute_open_signal(&self, signal: ExecutionSignal) -> Result<()> {
        if let (Some(side), Some(price)) = (signal.side, signal.price) {
            let request = ExecutionRequest {
                id: Uuid::new_v4(),
                strategy_id: signal.strategy_id,
                symbol: signal.symbol,
                side,
                order_type: OrderType::Limit,
                quantity: signal.quantity,
                price: Some(price),
                time_in_force: TimeInForce::GoodTillCancel,
                reduce_only: false,
                post_only: false,
            };

            let _result = self.execute_order(request).await?;
        }
        Ok(())
    }

    /// Execute close position signal
    #[allow(dead_code)]
    async fn execute_close_signal(&self, signal: ExecutionSignal) -> Result<()> {
        let positions = self.positions.read().await;

        if let Some(position) = positions.get(&format!("{}-{}",
                                                   signal.strategy_id, signal.symbol.as_str())) {
            let close_side = match position.side {
                PositionSide::Long => OrderSide::Sell,
                PositionSide::Short => OrderSide::Buy,
                PositionSide::Net => {
                    // Determine side based on position quantity sign
                    if position.quantity.as_decimal() > Decimal::ZERO {
                        OrderSide::Sell
                    } else {
                        OrderSide::Buy
                    }
                }
            };

            let close_quantity = if signal.signal_type == SignalType::PartialClose {
                signal.quantity
            } else {
                position.quantity
            };

            let request = ExecutionRequest {
                id: Uuid::new_v4(),
                strategy_id: signal.strategy_id,
                symbol: signal.symbol,
                side: close_side,
                order_type: OrderType::Market,
                quantity: close_quantity,
                price: None,
                time_in_force: TimeInForce::ImmediateOrCancel,
                reduce_only: true,
                post_only: false,
            };

            let _result = self.execute_order(request).await?;
        }

        Ok(())
    }

    /// Execute risk management signal
    #[allow(dead_code)]
    async fn execute_risk_signal(&self, signal: ExecutionSignal) -> Result<()> {
        // High-priority execution - use market orders
        let positions = self.positions.read().await;

        if let Some(position) = positions.get(&format!("{}-{}",
                                                   signal.strategy_id, signal.symbol.as_str())) {
            let risk_side = match position.side {
                PositionSide::Long => OrderSide::Sell,
                PositionSide::Short => OrderSide::Buy,
                PositionSide::Net => {
                    if position.quantity.as_decimal() > Decimal::ZERO {
                        OrderSide::Sell
                    } else {
                        OrderSide::Buy
                    }
                }
            };

            let request = ExecutionRequest {
                id: Uuid::new_v4(),
                strategy_id: signal.strategy_id,
                symbol: signal.symbol,
                side: risk_side,
                order_type: OrderType::Market,
                quantity: position.quantity,
                price: None,
                time_in_force: TimeInForce::ImmediateOrCancel,
                reduce_only: true,
                post_only: false,
            };

            let _result = self.execute_order(request).await?;
        }

        Ok(())
    }

    /// Validate order request
    fn validate_order_request(&self, request: &ExecutionRequest) -> Result<()> {
        if request.quantity.as_decimal() <= Decimal::ZERO {
            return Err(Error::InvalidQuantity("Quantity must be positive".to_string()));
        }

        if let Some(price) = request.price {
            if price.as_decimal() <= Decimal::ZERO {
                return Err(Error::InvalidPrice("Price must be positive".to_string()));
            }
        }

        Ok(())
    }

    /// Submit order to OKX (mock implementation)
    async fn submit_to_okx(&self, _order: &Order) -> Result<String> {
        // In real implementation, this would call OKX API
        Ok(format!("okx_{}", Uuid::new_v4()))
    }

    /// Simulate order execution
    async fn simulate_execution(&self, order: &mut Order) -> Result<bool> {
        // Simulate random execution for demo
        let success = rand::random::<f64>() > 0.1; // 90% success rate

        if success {
            let fill_qty = order.quantity;
            let fill_price = order.price.unwrap_or_else(|| {
                // Simulate market price for market orders
                Price::new(Decimal::from_f64_retain(45000.0).unwrap()).unwrap()
            });

            order.update_fill(fill_qty, fill_price);
            Ok(true)
        } else {
            order.set_status(OrderStatus::Rejected);
            Ok(false)
        }
    }

    /// Create trade record from order
    fn create_trade_record(&self, order: &Order) -> Result<Trade> {
        let trade = Trade::new(
            order.strategy_id,
            order.client_order_id.clone(),
            order.symbol.clone(),
            order.side,
            order.order_type,
            order.filled_quantity,
            order.avg_fill_price.unwrap(),
            Decimal::from_f64_retain(0.001).unwrap(), // 0.1% commission
        );

        Ok(trade)
    }

    /// Update positions from trade execution
    async fn update_positions_from_trade(&self, trade: &Trade) -> Result<()> {
        let position_key = format!("{}-{}", trade.strategy_id, trade.symbol.as_str());
        let mut positions = self.positions.write().await;

        if let Some(position) = positions.get_mut(&position_key) {
            // Update existing position
            self.update_existing_position(position, trade)?;
        } else {
            // Create new position
            let new_position = self.create_position_from_trade(trade)?;
            positions.insert(position_key, new_position);
        }

        Ok(())
    }

    /// Update existing position from trade
    fn update_existing_position(&self, position: &mut Position, trade: &Trade) -> Result<()> {
        let trade_qty = trade.quantity.as_decimal();
        let trade_price = trade.price.as_decimal();

        // Calculate new position size and average price
        let current_qty = position.quantity.as_decimal();
        let current_entry_price = position.avg_entry_price.as_decimal();

        let is_same_side = match (trade.side, position.side) {
            (ea_okx_core::models::order::OrderSide::Buy, ea_okx_core::models::position::PositionSide::Long) |
            (ea_okx_core::models::order::OrderSide::Sell, ea_okx_core::models::position::PositionSide::Short) => true,
            _ => false,
        };

        let (new_qty, new_entry_price) = if is_same_side {
            // Adding to position
            let total_value = (current_qty * current_entry_price) + (trade_qty * trade_price);
            let new_total_qty = current_qty + trade_qty;
            let new_entry_price = if new_total_qty != Decimal::ZERO {
                total_value / new_total_qty
            } else {
                Decimal::ZERO
            };

            (new_total_qty, new_entry_price)
        } else {
            // Reducing position
            let new_total_qty = current_qty - trade_qty;
            let new_entry_price = if new_total_qty != Decimal::ZERO {
                current_entry_price // Keep original entry price for remaining position
            } else {
                Decimal::ZERO
            };

            (new_total_qty, new_entry_price)
        };

        position.quantity = Quantity::new(new_qty)
            .map_err(|e| Error::ValidationError(e.to_string()))?;
        position.avg_entry_price = Price::new(new_entry_price)
            .map_err(|e| Error::ValidationError(e.to_string()))?;
        position.last_updated = Utc::now();

        // Calculate realized PnL for closing trades
        if (trade.side == OrderSide::Buy && position.side == PositionSide::Short) ||
           (trade.side == OrderSide::Sell && position.side == PositionSide::Long) {
            let realized_pnl = self.calculate_realized_pnl(position, trade)?;
            position.realized_pnl += realized_pnl;
        }

        Ok(())
    }

    /// Create new position from trade
    fn create_position_from_trade(&self, trade: &Trade) -> Result<Position> {
        let position_side = match trade.side {
            OrderSide::Buy => PositionSide::Long,
            OrderSide::Sell => PositionSide::Short,
        };

        let position = Position::new(
            trade.strategy_id,
            trade.symbol.clone(),
            position_side,
            trade.quantity,
            trade.price,
        );

        Ok(position)
    }

    /// Calculate realized PnL for trade
    fn calculate_realized_pnl(&self, position: &Position, trade: &Trade) -> Result<Decimal> {
        let entry_price = position.avg_entry_price.as_decimal();
        let exit_price = trade.price.as_decimal();
        let trade_qty = trade.quantity.as_decimal();

        let pnl = match position.side {
            PositionSide::Long => {
                (exit_price - entry_price) * trade_qty
            }
            PositionSide::Short => {
                (entry_price - exit_price) * trade_qty
            }
            PositionSide::Net => {
                // Complex calculation for net positions
                Decimal::ZERO
            }
        };

        Ok(pnl)
    }

    /// Get all orders
    pub async fn get_orders(&self) -> Vec<Order> {
        self.orders.read().await.values().cloned().collect()
    }

    /// Get all positions
    pub async fn get_positions(&self) -> Vec<Position> {
        self.positions.read().await.values().cloned().collect()
    }

    /// Get all trades
    pub async fn get_trades(&self, limit: Option<usize>) -> Vec<Trade> {
        let trades = self.trades.read().await;
        match limit {
            Some(limit) => trades.iter().rev().take(limit).cloned().collect(),
            None => trades.iter().rev().cloned().collect(),
        }
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(order_id) {
            if order.is_active() {
                order.set_status(OrderStatus::Cancelled);

                // Notify monitor
                if let Some(monitor) = &self.monitor {
                    let _ = monitor.emit_error(
                        order.strategy_id.to_string(),
                        format!("Order {} cancelled", order_id),
                    ).await;
                }
            }
        }
        Ok(())
    }

    /// Get strategy statistics
    pub async fn get_strategy_stats(&self, strategy_id: &str) -> Result<serde_json::Value> {
        let orders = self.orders.read().await;
        let positions = self.positions.read().await;
        let trades = self.trades.read().await;

        let strategy_orders: Vec<&Order> = orders.values()
            .filter(|o| o.strategy_id.to_string() == strategy_id)
            .collect();

        let strategy_positions: Vec<&Position> = positions.values()
            .filter(|p| p.strategy_id.to_string() == strategy_id)
            .collect();

        let strategy_trades: Vec<&Trade> = trades.iter()
            .filter(|t| t.strategy_id.to_string() == strategy_id)
            .collect();

        let total_pnl = strategy_trades.iter()
            .filter_map(|t| t.realized_pnl)
            .fold(Decimal::ZERO, |acc, pnl| acc + pnl);

        let unrealized_pnl = strategy_positions.iter()
            .fold(Decimal::ZERO, |acc, p| acc + p.unrealized_pnl);

        Ok(serde_json::json!({
            "strategy_id": strategy_id,
            "total_orders": strategy_orders.len(),
            "open_orders": strategy_orders.iter().filter(|o| o.is_active()).count(),
            "total_trades": strategy_trades.len(),
            "open_positions": strategy_positions.len(),
            "realized_pnl": total_pnl.to_string(),
            "unrealized_pnl": unrealized_pnl.to_string(),
            "total_pnl": (total_pnl + unrealized_pnl).to_string(),
            "win_rate": if strategy_trades.is_empty() { 0.0 } else {
                strategy_trades.iter().filter(|t| {
                    t.realized_pnl.map_or(false, |pnl| pnl > Decimal::ZERO)
                }).count() as f64 / strategy_trades.len() as f64
            }
        }))
    }
}

impl Default for StrategyExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

