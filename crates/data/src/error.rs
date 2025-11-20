//! Error types for data layer

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Duplicate data detected: {0}")]
    DuplicateData(String),

    #[error("Stale data: {0}")]
    StaleData(String),

    #[error("Anomaly detected: {0}")]
    AnomalyDetected(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] ea_okx_client::Error),

    #[error("Core error: {0}")]
    CoreError(#[from] ea_okx_core::error::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::ValidationError("Invalid price".to_string());
        assert!(err.to_string().contains("Invalid price"));
    }

    #[test]
    fn test_duplicate_error() {
        let err = Error::DuplicateData("tick-123".to_string());
        assert!(err.to_string().contains("tick-123"));
    }
}
