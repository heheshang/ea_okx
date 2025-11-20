use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use ea_okx_core::models::{Order, OrderStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Order state in the execution lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderState {
    /// Order created but not validated
    Created,
    /// Passed pre-trade validation
    Validated,
    /// Submitted to exchange
    Submitted,
    /// Acknowledged by exchange
    Acknowledged,
    /// Partially filled
    PartiallyFilled,
    /// Completely filled
    Filled,
    /// Cancelled by user/system
    Cancelled,
    /// Rejected by exchange
    Rejected,
    /// Failed to submit
    Failed,
    /// Expired (timeout)
    Expired,
}

impl OrderState {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderState::Filled
                | OrderState::Cancelled
                | OrderState::Rejected
                | OrderState::Failed
                | OrderState::Expired
        )
    }

    /// Check if order can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderState::Created
                | OrderState::Validated
                | OrderState::Submitted
                | OrderState::Acknowledged
                | OrderState::PartiallyFilled
        )
    }
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: OrderState,
    pub to_state: OrderState,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Order state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStateMachine {
    pub order_id: Uuid,
    pub current_state: OrderState,
    pub transitions: Vec<StateTransition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrderStateMachine {
    /// Create new state machine for an order
    pub fn new(order_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            order_id,
            current_state: OrderState::Created,
            transitions: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Attempt to transition to a new state
    pub fn transition(
        &mut self,
        to_state: OrderState,
        reason: impl Into<String>,
    ) -> Result<()> {
        if !self.is_valid_transition(to_state) {
            return Err(Error::InvalidStateTransition(format!(
                "Cannot transition from {:?} to {:?} for order {}",
                self.current_state, to_state, self.order_id
            )));
        }

        let transition = StateTransition {
            from_state: self.current_state,
            to_state,
            timestamp: Utc::now(),
            reason: reason.into(),
            metadata: HashMap::new(),
        };

        self.transitions.push(transition);
        self.current_state = to_state;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if a transition is valid
    fn is_valid_transition(&self, to_state: OrderState) -> bool {
        use OrderState::*;

        // Can't transition from terminal states
        if self.current_state.is_terminal() {
            return false;
        }

        // Same state is always valid (for updates)
        if self.current_state == to_state {
            return true;
        }

        // Define valid transitions
        matches!(
            (self.current_state, to_state),
            (Created, Validated)
                | (Created, Rejected)
                | (Created, Failed)
                | (Validated, Submitted)
                | (Validated, Rejected)
                | (Validated, Cancelled)
                | (Submitted, Acknowledged)
                | (Submitted, Rejected)
                | (Submitted, Failed)
                | (Submitted, Cancelled)
                | (Submitted, Expired)
                | (Acknowledged, PartiallyFilled)
                | (Acknowledged, Filled)
                | (Acknowledged, Cancelled)
                | (Acknowledged, Rejected)
                | (PartiallyFilled, Filled)
                | (PartiallyFilled, Cancelled)
        )
    }

    /// Get time in current state
    pub fn time_in_state(&self) -> chrono::Duration {
        Utc::now() - self.updated_at
    }

    /// Get total lifetime
    pub fn lifetime(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Check if order is active (not terminal)
    pub fn is_active(&self) -> bool {
        !self.current_state.is_terminal()
    }
}

/// Convert OrderStatus to OrderState
impl From<OrderStatus> for OrderState {
    fn from(status: OrderStatus) -> Self {
        match status {
            OrderStatus::Created => OrderState::Created,
            OrderStatus::Submitted => OrderState::Submitted,
            OrderStatus::Partial => OrderState::PartiallyFilled,
            OrderStatus::Filled => OrderState::Filled,
            OrderStatus::Cancelled => OrderState::Cancelled,
            OrderStatus::Rejected => OrderState::Rejected,
            OrderStatus::Failed => OrderState::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let order_id = Uuid::new_v4();
        let sm = OrderStateMachine::new(order_id);
        
        assert_eq!(sm.current_state, OrderState::Created);
        assert_eq!(sm.transitions.len(), 0);
        assert!(sm.is_active());
    }

    #[test]
    fn test_valid_transitions() {
        let mut sm = OrderStateMachine::new(Uuid::new_v4());
        
        // Created -> Validated
        assert!(sm.transition(OrderState::Validated, "Passed validation").is_ok());
        assert_eq!(sm.current_state, OrderState::Validated);
        assert_eq!(sm.transitions.len(), 1);
        
        // Validated -> Submitted
        assert!(sm.transition(OrderState::Submitted, "Sent to exchange").is_ok());
        assert_eq!(sm.current_state, OrderState::Submitted);
        
        // Submitted -> Acknowledged
        assert!(sm.transition(OrderState::Acknowledged, "Exchange confirmed").is_ok());
        
        // Acknowledged -> Filled
        assert!(sm.transition(OrderState::Filled, "Order filled").is_ok());
        assert_eq!(sm.current_state, OrderState::Filled);
        assert!(!sm.is_active());
    }

    #[test]
    fn test_invalid_transitions() {
        let mut sm = OrderStateMachine::new(Uuid::new_v4());
        
        // Can't go directly from Created to Filled
        assert!(sm.transition(OrderState::Filled, "Invalid").is_err());
        
        // Transition to Filled first
        sm.transition(OrderState::Validated, "OK").unwrap();
        sm.transition(OrderState::Submitted, "OK").unwrap();
        sm.transition(OrderState::Acknowledged, "OK").unwrap();
        sm.transition(OrderState::Filled, "OK").unwrap();
        
        // Can't transition from terminal state
        assert!(sm.transition(OrderState::Cancelled, "Terminal state").is_err());
    }

    #[test]
    fn test_terminal_states() {
        assert!(OrderState::Filled.is_terminal());
        assert!(OrderState::Cancelled.is_terminal());
        assert!(OrderState::Rejected.is_terminal());
        assert!(OrderState::Failed.is_terminal());
        assert!(OrderState::Expired.is_terminal());
        
        assert!(!OrderState::Created.is_terminal());
        assert!(!OrderState::Validated.is_terminal());
        assert!(!OrderState::Submitted.is_terminal());
    }

    #[test]
    fn test_can_cancel() {
        assert!(OrderState::Created.can_cancel());
        assert!(OrderState::Validated.can_cancel());
        assert!(OrderState::Submitted.can_cancel());
        assert!(OrderState::Acknowledged.can_cancel());
        assert!(OrderState::PartiallyFilled.can_cancel());
        
        assert!(!OrderState::Filled.can_cancel());
        assert!(!OrderState::Cancelled.can_cancel());
        assert!(!OrderState::Rejected.can_cancel());
    }
}
