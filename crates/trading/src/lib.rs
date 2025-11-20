pub mod algorithms;
pub mod error;
pub mod order_manager;
pub mod state_machine;

pub use algorithms::{TwapConfig, TwapExecutor, TwapResult, VwapConfig, VwapExecutor, VwapResult, SliceExecution};
pub use error::{Error, Result};
pub use order_manager::{OrderManager, OrderManagerConfig, OrderManagerStats, OrderEvent};
pub use state_machine::{OrderState, OrderStateMachine, StateTransition};
