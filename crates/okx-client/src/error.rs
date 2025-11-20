//! Error types for OKX client

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("WebSocket connection error: {0}")]
    WebSocketConnection(String),

    #[error("WebSocket send error: {0}")]
    WebSocketSend(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("API error: {code} - {message}")]
    ApiError { code: String, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::AuthError("Invalid API key".to_string());
        assert!(err.to_string().contains("Invalid API key"));
    }
}
