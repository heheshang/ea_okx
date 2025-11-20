use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Core error: {0}")]
    CoreError(#[from] ea_okx_core::error::Error),

    #[error("OKX client error: {0}")]
    ClientError(#[from] ea_okx_client::error::Error),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Reconciliation error: {0}")]
    ReconciliationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
