//! Authentication utilities for OKX API

use crate::error::{Error, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// OKX API credentials
#[derive(Clone)]
pub struct Credentials {
    api_key: String,
    secret_key: String,
    passphrase: String,
}

impl Credentials {
    /// Creates new credentials
    ///
    /// # Arguments
    ///
    /// * `api_key` - OKX API key
    /// * `secret_key` - OKX secret key
    /// * `passphrase` - OKX API passphrase
    ///
    /// # Examples
    ///
    /// ```
    /// use ea_okx_client::auth::Credentials;
    ///
    /// let credentials = Credentials::new("api-key", "secret-key", "passphrase");
    /// ```
    pub fn new(api_key: impl Into<String>, secret_key: impl Into<String>, passphrase: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            secret_key: secret_key.into(),
            passphrase: passphrase.into(),
        }
    }

    /// Returns the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the passphrase
    pub fn passphrase(&self) -> &str {
        &self.passphrase
    }

    /// Generates signature for OKX API request
    ///
    /// Signature = base64(HMAC-SHA256(secret_key, timestamp + method + request_path + body))
    ///
    /// # Arguments
    ///
    /// * `timestamp` - ISO 8601 timestamp
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `request_path` - API endpoint path with query parameters
    /// * `body` - Request body (empty string for GET requests)
    pub fn sign(&self, timestamp: &str, method: &str, request_path: &str, body: &str) -> Result<String> {
        let prehash = format!("{}{}{}{}", timestamp, method, request_path, body);
        
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| Error::AuthError(format!("Invalid secret key: {}", e)))?;
        
        mac.update(prehash.as_bytes());
        let result = mac.finalize();
        let signature = general_purpose::STANDARD.encode(result.into_bytes());
        
        Ok(signature)
    }

    /// Generates current timestamp in ISO 8601 format
    pub fn timestamp() -> String {
        Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }
}

/// Request signer for OKX API
pub struct RequestSigner {
    credentials: Credentials,
}

impl RequestSigner {
    /// Creates a new request signer
    pub fn new(credentials: Credentials) -> Self {
        Self { credentials }
    }

    /// Signs a request and returns authentication headers
    ///
    /// Returns a tuple of (timestamp, signature)
    pub fn sign_request(&self, method: &str, request_path: &str, body: &str) -> Result<(String, String)> {
        let timestamp = Credentials::timestamp();
        let signature = self.credentials.sign(&timestamp, method, request_path, body)?;
        Ok((timestamp, signature))
    }

    /// Returns the API key
    pub fn api_key(&self) -> &str {
        self.credentials.api_key()
    }

    /// Returns the passphrase
    pub fn passphrase(&self) -> &str {
        self.credentials.passphrase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_new() {
        let creds = Credentials::new("test-key", "test-secret", "test-pass");
        assert_eq!(creds.api_key(), "test-key");
        assert_eq!(creds.passphrase(), "test-pass");
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = Credentials::timestamp();
        assert!(timestamp.contains("T"));
        assert!(timestamp.ends_with("Z"));
    }

    #[test]
    fn test_sign() {
        let creds = Credentials::new("test-key", "test-secret", "test-pass");
        let timestamp = "2024-01-01T00:00:00.000Z";
        let signature = creds.sign(timestamp, "GET", "/api/v5/account/balance", "").unwrap();
        
        // Signature should be base64 encoded
        assert!(!signature.is_empty());
        assert!(general_purpose::STANDARD.decode(&signature).is_ok());
    }

    #[test]
    fn test_request_signer() {
        let creds = Credentials::new("test-key", "test-secret", "test-pass");
        let signer = RequestSigner::new(creds);
        
        let (timestamp, signature) = signer.sign_request("GET", "/api/v5/account/balance", "").unwrap();
        
        assert!(!timestamp.is_empty());
        assert!(!signature.is_empty());
        assert_eq!(signer.api_key(), "test-key");
    }

    #[test]
    fn test_sign_with_body() {
        let creds = Credentials::new("test-key", "test-secret", "test-pass");
        let body = r#"{"instId":"BTC-USDT","tdMode":"cash"}"#;
        let signature1 = creds.sign("2024-01-01T00:00:00.000Z", "POST", "/api/v5/trade/order", body).unwrap();
        let signature2 = creds.sign("2024-01-01T00:00:00.000Z", "POST", "/api/v5/trade/order", "").unwrap();
        
        // Different bodies should produce different signatures
        assert_ne!(signature1, signature2);
    }
}
