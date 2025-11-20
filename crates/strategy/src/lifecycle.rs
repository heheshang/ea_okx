//! Strategy lifecycle state machine

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Strategy lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyState {
    Draft,
    Validating,
    ValidationFailed,
    Backtesting,
    BacktestFailed,
    PaperTrading,
    PaperFailed,
    ReadyForLive,
    Active,
    Paused,
    Stopped,
    Archived,
}

/// Strategy lifecycle manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyLifecycle {
    current_state: StrategyState,
    state_history: Vec<StateTransition>,
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    from_state: StrategyState,
    to_state: StrategyState,
    timestamp: DateTime<Utc>,
    reason: String,
}

impl StrategyLifecycle {
    pub fn new() -> Self {
        Self {
            current_state: StrategyState::Draft,
            state_history: vec![],
        }
    }

    pub fn current_state(&self) -> StrategyState {
        self.current_state
    }

    pub fn transition(&mut self, to_state: StrategyState, reason: impl Into<String>) -> Result<()> {
        if !self.is_valid_transition(to_state) {
            return Err(Error::StateTransitionError(format!(
                "Invalid transition from {:?} to {:?}",
                self.current_state, to_state
            )));
        }

        self.state_history.push(StateTransition {
            from_state: self.current_state,
            to_state,
            timestamp: Utc::now(),
            reason: reason.into(),
        });

        self.current_state = to_state;
        Ok(())
    }

    fn is_valid_transition(&self, to_state: StrategyState) -> bool {
        use StrategyState::*;

        matches!(
            (self.current_state, to_state),
            (Draft, Validating)
                | (Validating, ValidationFailed)
                | (Validating, Backtesting)
                | (ValidationFailed, Draft)
                | (Backtesting, BacktestFailed)
                | (Backtesting, PaperTrading)
                | (BacktestFailed, Draft)
                | (PaperTrading, PaperFailed)
                | (PaperTrading, ReadyForLive)
                | (PaperFailed, Draft)
                | (ReadyForLive, Active)
                | (Active, Paused)
                | (Paused, Active)
                | (Active, Stopped)
                | (Paused, Stopped)
                | (Stopped, Archived)
        )
    }
}

impl Default for StrategyLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let lifecycle = StrategyLifecycle::new();
        assert_eq!(lifecycle.current_state(), StrategyState::Draft);
    }

    #[test]
    fn test_valid_transition() {
        let mut lifecycle = StrategyLifecycle::new();
        assert!(lifecycle
            .transition(StrategyState::Validating, "Start validation")
            .is_ok());
        assert_eq!(lifecycle.current_state(), StrategyState::Validating);
    }

    #[test]
    fn test_invalid_transition() {
        let mut lifecycle = StrategyLifecycle::new();
        assert!(lifecycle
            .transition(StrategyState::Active, "Invalid")
            .is_err());
    }
}
