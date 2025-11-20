//! OKX API client library
//!
//! This crate provides a comprehensive client for the OKX cryptocurrency exchange API,
//! supporting both REST and WebSocket interfaces.
//!
//! # Features
//!
//! - REST API client with automatic authentication
//! - WebSocket client for real-time market data
//! - Rate limiting and retry logic
//! - Type-safe request/response models
//!
//! # Examples
//!
//! ```no_run
//! use ea_okx_client::rest::OkxRestClient;
//! use ea_okx_client::auth::Credentials;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let credentials = Credentials::new("api-key", "secret-key", "passphrase");
//!     let client = OkxRestClient::new(credentials, false)?;
//!     
//!     // Use the client...
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod error;
pub mod models;
pub mod rest;
pub mod websocket;

pub use auth::Credentials;
pub use error::{Error, Result};
pub use rest::OkxRestClient;
pub use websocket::OkxWebSocketClient;
