// ! Error types for the core domain

use thiserror::Error;

/// Main error type for the core domain
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid symbol format: {0}")]
    InvalidSymbol(String),

    #[error("Invalid price: {0}")]
    InvalidPrice(String),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),

    #[error("Invalid order type: {0}")]
    InvalidOrderType(String),

    #[error("Invalid order side: {0}")]
    InvalidOrderSide(String),

    #[error("Invalid order status: {0}")]
    InvalidOrderStatus(String),

    #[error("Invalid position side: {0}")]
    InvalidPositionSide(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Decimal conversion error: {0}")]
    DecimalError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias using the core Error type
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidSymbol("test".to_string());
        assert_eq!(err.to_string(), "Invalid symbol format: test");
    }

    #[test]
    fn test_error_from_serde() {
        let json_err = serde_json::from_str::<i32>("not a number");
        assert!(json_err.is_err());

        let err: Error = json_err.unwrap_err().into();
        assert!(matches!(err, Error::SerializationError(_)));
    }
}
