//! Core domain models and types for the EA OKX quantitative trading system.
//!
//! This crate provides fundamental types used across the entire system:
//! - Symbol types for trading pairs
//! - Price and quantity types with precise decimal arithmetic
//! - Order and position models
//! - Error types
//!
//! # Examples
//!
//! ```
//! use ea_okx_core::types::Symbol;
//!
//! let symbol = Symbol::new("BTC-USDT").unwrap();
//! assert_eq!(symbol.base(), "BTC");
//! assert_eq!(symbol.quote(), "USDT");
//! ```

pub mod error;
pub mod models;
pub mod types;

// Re-export common types for convenience
pub use error::{Error, Result};
pub use types::{Decimal, Price, Quantity, Symbol};
