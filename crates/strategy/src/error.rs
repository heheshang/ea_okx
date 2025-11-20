//! Error types for strategy framework

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Strategy initialization error: {0}")]
    InitializationError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("State transition error: {0}")]
    StateTransitionError(String),

    #[error("Signal generation error: {0}")]
    SignalError(String),

    #[error("Core error: {0}")]
    CoreError(#[from] ea_okx_core::error::Error),

    // #[error("Data error: {0}")]
    // DataError(#[from] ea_okx_data::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
