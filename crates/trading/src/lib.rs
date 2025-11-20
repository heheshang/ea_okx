pub mod algorithms;
pub mod error;
pub mod order_manager;
pub mod state_machine;

pub use algorithms::{
    SliceExecution, TwapConfig, TwapExecutor, TwapResult, VwapConfig, VwapExecutor, VwapResult,
};
pub use error::{Error, Result};
pub use order_manager::{OrderEvent, OrderManager, OrderManagerConfig, OrderManagerStats};
pub use state_machine::{OrderState, OrderStateMachine, StateTransition};
