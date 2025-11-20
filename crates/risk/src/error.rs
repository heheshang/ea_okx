use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Core error: {0}")]
    CoreError(#[from] ea_okx_core::error::Error),

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Insufficient margin: required {required}, available {available}")]
    InsufficientMargin { required: String, available: String },

    #[error("Position limit exceeded: {0}")]
    PositionLimitExceeded(String),

    #[error("Daily loss limit exceeded: {0}")]
    DailyLossLimitExceeded(String),

    #[error("Leverage limit exceeded: {0}")]
    LeverageLimitExceeded(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
