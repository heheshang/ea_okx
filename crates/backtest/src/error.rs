use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("Data error: {0}")]
    // DataError(#[from] ea_okx_data::error::Error),

    #[error("Strategy error: {0}")]
    StrategyError(#[from] ea_okx_strategy::error::Error),

    #[error("Core error: {0}")]
    CoreError(#[from] ea_okx_core::error::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
}

pub type Result<T> = std::result::Result<T, Error>;
