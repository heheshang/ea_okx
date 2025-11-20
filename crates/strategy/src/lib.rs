//! Strategy framework for EA OKX Quantitative Trading System
//!
//! This crate provides the core strategy interface, lifecycle management,
//! and plugin architecture for implementing trading strategies.
//!
//! # Features
//!
//! - Trait-based strategy interface
//! - Strategy lifecycle state machine
//! - Hot-reload mechanism with state serialization
//! - Performance metrics tracking
//! - Signal generation framework

pub mod error;
pub mod lifecycle;
pub mod metrics;
pub mod signal;
pub mod traits;

pub use error::{Error, Result};
pub use lifecycle::{StrategyLifecycle, StrategyState};
pub use metrics::PerformanceMetrics;
pub use signal::{Signal, SignalType};
pub use traits::{MarketDataEvent, Strategy, StrategyConfig};
