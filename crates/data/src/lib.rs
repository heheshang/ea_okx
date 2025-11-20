//! Data layer for EA OKX Quantitative Trading System
//!
//! This crate provides market data ingestion, quality control,
//! and storage functionality.
//!
//! # Features
//!
//! - Market data collection from WebSocket streams
//! - Real-time data quality validation
//! - Deduplication and anomaly detection
//! - TimescaleDB and Redis integration
//! - Automatic data enrichment

pub mod collector;
pub mod error;
pub mod quality;
pub mod storage;

pub use collector::MarketDataCollector;
pub use error::{Error, Result};
pub use quality::QualityControl;
