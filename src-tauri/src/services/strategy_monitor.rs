//! Strategy monitoring service for real-time updates via WebSocket

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use serde_json::Value as JsonValue;

use ea_okx_core::{
    models::strategy::{Strategy, StrategyStatus, StrategyMetrics},
    error::{Error, Result},
};

/// Strategy update event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StrategyUpdateEvent {
    /// Strategy status changed
    StatusChanged {
        strategy_id: String,
        old_status: StrategyStatus,
        new_status: StrategyStatus,
        timestamp: chrono::DateTime<Utc>,
    },
    /// New trade executed
    TradeExecuted {
        strategy_id: String,
        trade_id: String,
        symbol: String,
        side: String,
        amount: f64,
        price: f64,
        timestamp: chrono::DateTime<Utc>,
    },
    /// Performance metrics updated
    MetricsUpdated {
        strategy_id: String,
        metrics: StrategyMetrics,
        timestamp: chrono::DateTime<Utc>,
    },
    /// Signal generated
    SignalGenerated {
        strategy_id: String,
        signal_type: String,
        symbol: String,
        price: f64,
        confidence: f64,
        timestamp: chrono::DateTime<Utc>,
    },
    /// Error occurred
    Error {
        strategy_id: String,
        error_message: String,
        timestamp: chrono::DateTime<Utc>,
    },
    /// Position opened/closed
    PositionUpdate {
        strategy_id: String,
        symbol: String,
        side: String,
        size: f64,
        entry_price: Option<f64>,
        exit_price: Option<f64>,
        pnl: Option<f64>,
        timestamp: chrono::DateTime<Utc>,
    },
}

/// WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: String,
    pub event: StrategyUpdateEvent,
    pub timestamp: chrono::DateTime<Utc>,
}

/// Client subscription info
#[derive(Debug, Clone)]
pub struct ClientSubscription {
    #[allow(dead_code)]
    pub strategy_ids: Vec<String>,
    #[allow(dead_code)]
    pub event_types: Vec<String>,
    #[allow(dead_code)]
    pub sender: mpsc::UnboundedSender<WebSocketMessage>,
}

/// Strategy monitoring service
#[derive(Clone)]
pub struct StrategyMonitorService {
    strategies: Arc<RwLock<HashMap<String, Strategy>>>,
    clients: Arc<RwLock<HashMap<String, ClientSubscription>>>,
    event_tx: mpsc::UnboundedSender<StrategyUpdateEvent>,
}

impl StrategyMonitorService {
    /// Creates a new strategy monitoring service
    pub fn new() -> Self {
        let (event_tx, _) = mpsc::unbounded_channel();

        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Register a new client for real-time updates
    pub async fn subscribe_client(
        &self,
        strategy_ids: Vec<String>,
        event_types: Vec<String>,
    ) -> Result<mpsc::UnboundedReceiver<WebSocketMessage>> {
        let client_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        let subscription = ClientSubscription {
            strategy_ids,
            event_types,
            sender: tx,
        };

        let mut clients = self.clients.write().await;
        clients.insert(client_id.clone(), subscription);

        log::info!("Client subscribed: {}", client_id);
        Ok(rx)
    }

    /// Unsubscribe a client
    pub async fn unsubscribe_client(&self, client_id: &str) -> Result<()> {
        let mut clients = self.clients.write().await;
        if clients.remove(client_id).is_some() {
            log::info!("Client unsubscribed: {}", client_id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Client not found: {}", client_id)))
        }
    }

    /// Update strategy in the monitoring service
    pub async fn update_strategy(&self, strategy: Strategy) -> Result<()> {
        let strategy_id = strategy.id.to_string();
        let mut strategies = self.strategies.write().await;

        let old_status = strategies.get(&strategy_id).map(|s| s.status);
        strategies.insert(strategy_id.clone(), strategy.clone());

        // Emit status change event if status changed
        if let Some(old_status) = old_status {
            if old_status != strategy.status {
                self.emit_event(StrategyUpdateEvent::StatusChanged {
                    strategy_id,
                    old_status,
                    new_status: strategy.status,
                    timestamp: Utc::now(),
                }).await?;
            }
        }

        Ok(())
    }

    /// Emit a custom event
    pub async fn emit_event(&self, event: StrategyUpdateEvent) -> Result<()> {
        if let Err(e) = self.event_tx.send(event) {
            log::error!("Failed to emit event: {}", e);
            return Err(Error::Internal(e.to_string()));
        }
        Ok(())
    }

    /// Emit trade executed event
    pub async fn emit_trade_executed(
        &self,
        strategy_id: String,
        trade_id: String,
        symbol: String,
        side: String,
        amount: f64,
        price: f64,
    ) -> Result<()> {
        self.emit_event(StrategyUpdateEvent::TradeExecuted {
            strategy_id,
            trade_id,
            symbol,
            side,
            amount,
            price,
            timestamp: Utc::now(),
        }).await
    }

    /// Emit metrics updated event
    pub async fn emit_metrics_updated(
        &self,
        strategy_id: String,
        metrics: StrategyMetrics,
    ) -> Result<()> {
        self.emit_event(StrategyUpdateEvent::MetricsUpdated {
            strategy_id,
            metrics,
            timestamp: Utc::now(),
        }).await
    }

    /// Emit signal generated event
    pub async fn emit_signal_generated(
        &self,
        strategy_id: String,
        signal_type: String,
        symbol: String,
        price: f64,
        confidence: f64,
    ) -> Result<()> {
        self.emit_event(StrategyUpdateEvent::SignalGenerated {
            strategy_id,
            signal_type,
            symbol,
            price,
            confidence,
            timestamp: Utc::now(),
        }).await
    }

    /// Emit error event
    pub async fn emit_error(
        &self,
        strategy_id: String,
        error_message: String,
    ) -> Result<()> {
        self.emit_event(StrategyUpdateEvent::Error {
            strategy_id,
            error_message,
            timestamp: Utc::now(),
        }).await
    }

    /// Emit position update event
    pub async fn emit_position_update(
        &self,
        strategy_id: String,
        symbol: String,
        side: String,
        size: f64,
        entry_price: Option<f64>,
        exit_price: Option<f64>,
        pnl: Option<f64>,
    ) -> Result<()> {
        self.emit_event(StrategyUpdateEvent::PositionUpdate {
            strategy_id,
            symbol,
            side,
            size,
            entry_price,
            exit_price,
            pnl,
            timestamp: Utc::now(),
        }).await
    }

    
    /// Broadcast event to all interested clients
    #[allow(dead_code)]
    async fn broadcast_event(&self, event: StrategyUpdateEvent) -> Result<()> {
        let clients = self.clients.read().await;

        // Extract event type and strategy ID from event
        let (event_type, strategy_id) = match &event {
            StrategyUpdateEvent::StatusChanged { strategy_id, .. } => ("status_changed", strategy_id),
            StrategyUpdateEvent::TradeExecuted { strategy_id, .. } => ("trade_executed", strategy_id),
            StrategyUpdateEvent::MetricsUpdated { strategy_id, .. } => ("metrics_updated", strategy_id),
            StrategyUpdateEvent::SignalGenerated { strategy_id, .. } => ("signal_generated", strategy_id),
            StrategyUpdateEvent::Error { strategy_id, .. } => ("error", strategy_id),
            StrategyUpdateEvent::PositionUpdate { strategy_id, .. } => ("position_update", strategy_id),
        };

        let message = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            event: event.clone(),
            timestamp: Utc::now(),
        };

        // Send to all interested clients
        let mut failed_clients = Vec::new();

        for (client_id, subscription) in clients.iter() {
            // Check if client is interested in this strategy and event type
            let strategy_interested = subscription.strategy_ids.is_empty()
                || subscription.strategy_ids.contains(&strategy_id.to_string());
            let event_interested = subscription.event_types.is_empty()
                || subscription.event_types.contains(&event_type.to_string());

            if strategy_interested && event_interested {
                if subscription.sender.send(message.clone()).is_err() {
                    failed_clients.push(client_id.clone());
                }
            }
        }

        // Remove disconnected clients
        if !failed_clients.is_empty() {
            drop(clients);
            let mut clients = self.clients.write().await;
            for client_id in failed_clients {
                clients.remove(&client_id);
                log::info!("Removed disconnected client: {}", client_id);
            }
        }

        Ok(())
    }

    /// Get connected clients count
    pub async fn get_clients_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get strategy statistics
    pub async fn get_strategy_stats(&self) -> HashMap<String, JsonValue> {
        let strategies = self.strategies.read().await;
        let mut stats = HashMap::new();

        for (strategy_id, strategy) in strategies.iter() {
            let status_count = match strategy.status {
                StrategyStatus::Active => "active",
                StrategyStatus::Paused => "paused",
                StrategyStatus::Stopped => "stopped",
                StrategyStatus::Draft => "draft",
                StrategyStatus::Error => "error",
                _ => "other",
            };

            let strategy_stats = serde_json::json!({
                "id": strategy_id,
                "name": strategy.name,
                "status": status_count,
                "type": strategy.strategy_type,
                "updated_at": strategy.updated_at,
                "created_at": strategy.created_at,
            });

            stats.insert(strategy_id.clone(), strategy_stats);
        }

        stats
    }
}

impl Default for StrategyMonitorService {
    fn default() -> Self {
        Self::new()
    }
}