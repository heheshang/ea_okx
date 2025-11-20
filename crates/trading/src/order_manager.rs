use crate::error::{Error, Result};
use crate::state_machine::{OrderState, OrderStateMachine};
use chrono::{DateTime, Duration, Utc};
use ea_okx_client::OkxRestClient;
use ea_okx_core::models::{Order, OrderStatus};
use ea_okx_core::{Symbol, Price, Quantity};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Order manager configuration
#[derive(Debug, Clone)]
pub struct OrderManagerConfig {
    /// Reconciliation interval in seconds
    pub reconciliation_interval_secs: u64,
    
    /// Order timeout in seconds
    pub order_timeout_secs: u64,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Retry backoff multiplier
    pub retry_backoff_ms: u64,
}

impl Default for OrderManagerConfig {
    fn default() -> Self {
        Self {
            reconciliation_interval_secs: 10,
            order_timeout_secs: 30,
            max_retries: 3,
            retry_backoff_ms: 1000,
        }
    }
}

/// Order with state machine
#[derive(Debug, Clone)]
struct ManagedOrder {
    order: Order,
    state_machine: OrderStateMachine,
    retry_count: u32,
    last_sync: DateTime<Utc>,
}

/// Order event types
#[derive(Debug, Clone)]
pub enum OrderEvent {
    OrderCreated(Uuid),
    OrderSubmitted(Uuid),
    OrderAcknowledged { order_id: Uuid, exchange_id: String },
    OrderPartiallyFilled { order_id: Uuid, filled_qty: Quantity },
    OrderFilled { order_id: Uuid, avg_price: Price },
    OrderCancelled(Uuid),
    OrderRejected { order_id: Uuid, reason: String },
    OrderFailed { order_id: Uuid, reason: String },
    OrderExpired(Uuid),
}

/// Main order manager
pub struct OrderManager {
    config: OrderManagerConfig,
    client: Arc<OkxRestClient>,
    
    /// Active orders indexed by internal ID
    orders: Arc<RwLock<HashMap<Uuid, ManagedOrder>>>,
    
    /// Map exchange order ID to internal ID
    exchange_id_map: Arc<RwLock<HashMap<String, Uuid>>>,
    
    /// Event channel
    event_tx: mpsc::UnboundedSender<OrderEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<OrderEvent>>>>,
}

impl OrderManager {
    /// Create new order manager
    pub fn new(config: OrderManagerConfig, client: Arc<OkxRestClient>) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            config,
            client,
            orders: Arc::new(RwLock::new(HashMap::new())),
            exchange_id_map: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
        }
    }

    /// Submit a new order
    pub async fn submit_order(&self, mut order: Order) -> Result<Uuid> {
        let order_id = order.id;
        
        let price_str = order.price.map(|p| p.as_decimal().to_string()).unwrap_or_else(|| "market".to_string());
        debug!("Submitting order {}: {:?} {} @ {}", 
            order_id, order.side, order.symbol.as_str(), price_str);
        
        // Create state machine
        let mut state_machine = OrderStateMachine::new(order_id);
        state_machine.transition(OrderState::Validated, "Pre-trade checks passed")?;
        
        // Store order
        let managed_order = ManagedOrder {
            order: order.clone(),
            state_machine,
            retry_count: 0,
            last_sync: Utc::now(),
        };
        
        self.orders.write().insert(order_id, managed_order);
        
        // Emit event
        let _ = self.event_tx.send(OrderEvent::OrderCreated(order_id));
        
        // Submit to exchange (async)
        let self_clone = Self {
            config: self.config.clone(),
            client: self.client.clone(),
            orders: self.orders.clone(),
            exchange_id_map: self.exchange_id_map.clone(),
            event_tx: self.event_tx.clone(),
            event_rx: self.event_rx.clone(),
        };
        
        tokio::spawn(async move {
            if let Err(e) = self_clone.submit_to_exchange(order_id).await {
                error!("Failed to submit order {}: {}", order_id, e);
                let _ = self_clone.event_tx.send(OrderEvent::OrderFailed {
                    order_id,
                    reason: e.to_string(),
                });
            }
        });
        
        Ok(order_id)
    }

    /// Submit order to exchange
    async fn submit_to_exchange(&self, order_id: Uuid) -> Result<()> {
        // Get order
        let order = {
            let orders = self.orders.read();
            orders.get(&order_id)
                .ok_or_else(|| Error::OrderNotFound(order_id.to_string()))?
                .order.clone()
        };
        
        // Update state
        {
            let mut orders = self.orders.write();
            if let Some(managed) = orders.get_mut(&order_id) {
                managed.state_machine.transition(OrderState::Submitted, "Sending to exchange")?;
            }
        }
        
        let _ = self.event_tx.send(OrderEvent::OrderSubmitted(order_id));
        
        // Submit via REST API
        // Note: This would call the actual OKX client
        // For now, we'll simulate acknowledgment
        info!("Order {} submitted to exchange", order_id);
        
        // Simulate exchange response
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let exchange_id = format!("OKX-{}", order_id);
        
        // Update state
        {
            let mut orders = self.orders.write();
            if let Some(managed) = orders.get_mut(&order_id) {
                managed.state_machine.transition(OrderState::Acknowledged, "Exchange confirmed")?;
            }
        }
        
        // Map exchange ID
        self.exchange_id_map.write().insert(exchange_id.clone(), order_id);
        
        let _ = self.event_tx.send(OrderEvent::OrderAcknowledged {
            order_id,
            exchange_id,
        });
        
        Ok(())
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        // Check if order can be cancelled
        {
            let orders = self.orders.read();
            let managed = orders.get(&order_id)
                .ok_or_else(|| Error::OrderNotFound(order_id.to_string()))?;
            
            if !managed.state_machine.current_state.can_cancel() {
                return Err(Error::ExecutionError(format!(
                    "Order {} cannot be cancelled in state {:?}",
                    order_id, managed.state_machine.current_state
                )));
            }
        }
        
        info!("Cancelling order {}", order_id);
        
        // Send cancel request to exchange
        // (Would use actual OKX client here)
        
        // Update state
        {
            let mut orders = self.orders.write();
            if let Some(managed) = orders.get_mut(&order_id) {
                managed.state_machine.transition(OrderState::Cancelled, "User requested")?;
            }
        }
        
        let _ = self.event_tx.send(OrderEvent::OrderCancelled(order_id));
        
        Ok(())
    }

    /// Get order status
    pub fn get_order(&self, order_id: Uuid) -> Option<(Order, OrderState)> {
        let orders = self.orders.read();
        orders.get(&order_id).map(|managed| {
            (managed.order.clone(), managed.state_machine.current_state)
        })
    }

    /// Get all active orders
    pub fn get_active_orders(&self) -> Vec<(Order, OrderState)> {
        let orders = self.orders.read();
        orders.values()
            .filter(|m| m.state_machine.is_active())
            .map(|m| (m.order.clone(), m.state_machine.current_state))
            .collect()
    }

    /// Start reconciliation loop
    pub async fn start_reconciliation(&self) {
        let interval = Duration::seconds(self.config.reconciliation_interval_secs as i64);
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(
            self.config.reconciliation_interval_secs
        ));
        
        loop {
            ticker.tick().await;
            
            if let Err(e) = self.reconcile().await {
                error!("Reconciliation error: {}", e);
            }
        }
    }

    /// Reconcile orders with exchange
    async fn reconcile(&self) -> Result<()> {
        debug!("Starting order reconciliation");
        
        let active_orders: Vec<Uuid> = {
            let orders = self.orders.read();
            orders.values()
                .filter(|m| m.state_machine.is_active())
                .map(|m| m.order.id)
                .collect()
        };
        
        for order_id in active_orders {
            // Check order timeout
            let should_timeout = {
                let orders = self.orders.read();
                if let Some(managed) = orders.get(&order_id) {
                    let time_in_state = managed.state_machine.time_in_state();
                    time_in_state.num_seconds() > self.config.order_timeout_secs as i64
                } else {
                    false
                }
            };
            
            if should_timeout {
                warn!("Order {} timed out", order_id);
                let mut orders = self.orders.write();
                if let Some(managed) = orders.get_mut(&order_id) {
                    let _ = managed.state_machine.transition(OrderState::Expired, "Timeout");
                }
                let _ = self.event_tx.send(OrderEvent::OrderExpired(order_id));
            }
            
            // Fetch order status from exchange
            // (Would query actual OKX API here)
        }
        
        debug!("Reconciliation completed");
        Ok(())
    }

    /// Get event receiver
    pub fn subscribe_events(&self) -> Option<mpsc::UnboundedReceiver<OrderEvent>> {
        self.event_rx.write().take()
    }

    /// Get statistics
    pub fn get_stats(&self) -> OrderManagerStats {
        let orders = self.orders.read();
        
        let mut stats = OrderManagerStats::default();
        stats.total_orders = orders.len();
        
        for managed in orders.values() {
            match managed.state_machine.current_state {
                OrderState::Filled => stats.filled_orders += 1,
                OrderState::Cancelled => stats.cancelled_orders += 1,
                OrderState::Rejected => stats.rejected_orders += 1,
                OrderState::Failed => stats.failed_orders += 1,
                _ => stats.active_orders += 1,
            }
        }
        
        stats
    }
}

/// Order manager statistics
#[derive(Debug, Default, Clone)]
pub struct OrderManagerStats {
    pub total_orders: usize,
    pub active_orders: usize,
    pub filled_orders: usize,
    pub cancelled_orders: usize,
    pub rejected_orders: usize,
    pub failed_orders: usize,
}
