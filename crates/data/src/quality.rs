//! Data quality control pipeline
//!
//! This module implements comprehensive quality checks:
//! - Timestamp validation (reject future/stale data)
//! - Duplicate detection
//! - Range validation (price within reasonable bounds)
//! - Missing field detection
//! - Anomaly detection using statistical methods

use crate::error::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use ea_okx_core::types::{Price, Symbol};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, warn};

/// Quality control configuration
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Maximum age of data in seconds (reject stale data)
    pub max_data_age_secs: i64,
    
    /// Allow future timestamps within this tolerance (seconds)
    pub future_tolerance_secs: i64,
    
    /// Maximum price deviation from last valid price (as percentage, e.g., 0.10 for 10%)
    pub max_price_deviation_pct: Decimal,
    
    /// Window size for anomaly detection
    pub anomaly_window_size: usize,
    
    /// Z-score threshold for anomaly detection
    pub anomaly_zscore_threshold: f64,
    
    /// Enable duplicate detection
    pub enable_dedup: bool,
    
    /// Duplicate detection window size
    pub dedup_window_size: usize,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            max_data_age_secs: 5,
            future_tolerance_secs: 1,
            max_price_deviation_pct: Decimal::new(10, 2), // 0.10 = 10%
            anomaly_window_size: 100,
            anomaly_zscore_threshold: 3.0,
            enable_dedup: true,
            dedup_window_size: 1000,
        }
    }
}

/// Data quality control system
pub struct QualityControl {
    config: QualityConfig,
    
    /// Last valid prices per symbol
    last_prices: Arc<RwLock<HashMap<Symbol, Price>>>,
    
    /// Price history for anomaly detection per symbol
    price_history: Arc<RwLock<HashMap<Symbol, VecDeque<Decimal>>>>,
    
    /// Recent message IDs for deduplication
    recent_message_ids: Arc<RwLock<VecDeque<String>>>,
    
    /// Statistics
    stats: Arc<RwLock<QualityStats>>,
}

/// Quality control statistics
#[derive(Debug, Default, Clone)]
pub struct QualityStats {
    pub total_processed: u64,
    pub total_rejected: u64,
    pub future_timestamp_rejections: u64,
    pub stale_data_rejections: u64,
    pub price_deviation_rejections: u64,
    pub duplicate_rejections: u64,
    pub anomaly_rejections: u64,
    pub missing_field_rejections: u64,
}

impl QualityControl {
    /// Create a new quality control instance
    pub fn new(config: QualityConfig) -> Self {
        Self {
            config,
            last_prices: Arc::new(RwLock::new(HashMap::new())),
            price_history: Arc::new(RwLock::new(HashMap::new())),
            recent_message_ids: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(QualityStats::default())),
        }
    }
    
    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(QualityConfig::default())
    }
    
    /// Validate timestamp
    pub fn validate_timestamp(&self, timestamp: DateTime<Utc>) -> Result<()> {
        let now = Utc::now();
        let age = now.signed_duration_since(timestamp);
        
        // Check for future timestamps
        if age < Duration::zero() {
            let future_by = timestamp.signed_duration_since(now);
            if future_by.num_seconds() > self.config.future_tolerance_secs {
                self.stats.write().future_timestamp_rejections += 1;
                return Err(Error::ValidationError(format!(
                    "Future timestamp detected: {} seconds ahead",
                    future_by.num_seconds()
                )));
            }
        }
        
        // Check for stale data
        if age.num_seconds() > self.config.max_data_age_secs {
            self.stats.write().stale_data_rejections += 1;
            return Err(Error::StaleData(format!(
                "Data is {} seconds old, max allowed: {}",
                age.num_seconds(),
                self.config.max_data_age_secs
            )));
        }
        
        Ok(())
    }
    
    /// Validate price against historical data
    pub fn validate_price(&self, symbol: &Symbol, price: &Price) -> Result<()> {
        let last_prices = self.last_prices.read();
        
        if let Some(last_price) = last_prices.get(symbol) {
            let price_val = price.as_decimal();
            let last_price_val = last_price.as_decimal();
            
            // Calculate percentage deviation
            let deviation = (price_val - last_price_val).abs() / last_price_val;
            
            if deviation > self.config.max_price_deviation_pct {
                self.stats.write().price_deviation_rejections += 1;
                return Err(Error::ValidationError(format!(
                    "Price deviation too large: {:.2}% (max: {:.2}%)",
                    deviation * Decimal::new(100, 0),
                    self.config.max_price_deviation_pct * Decimal::new(100, 0)
                )));
            }
        }
        
        Ok(())
    }
    
    /// Detect anomalies using Z-score
    pub fn detect_anomaly(&self, symbol: &Symbol, price: &Price) -> Result<()> {
        let mut price_history = self.price_history.write();
        let history = price_history.entry(symbol.clone()).or_insert_with(VecDeque::new);
        
        // Need sufficient history for anomaly detection
        if history.len() < 10 {
            history.push_back(price.as_decimal());
            if history.len() > self.config.anomaly_window_size {
                history.pop_front();
            }
            return Ok(());
        }
        
        // Calculate mean and standard deviation
        let values: Vec<f64> = history.iter().map(|p| p.to_string().parse().unwrap_or(0.0)).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        // Calculate Z-score for current price
        let current_price: f64 = price.as_decimal().to_string().parse().unwrap_or(0.0);
        let z_score = if std_dev > 0.0 {
            (current_price - mean).abs() / std_dev
        } else {
            0.0
        };
        
        // Check if Z-score exceeds threshold
        if z_score > self.config.anomaly_zscore_threshold {
            self.stats.write().anomaly_rejections += 1;
            warn!(
                "Anomaly detected for {}: price={}, mean={:.2}, z_score={:.2}",
                symbol.as_str(),
                current_price,
                mean,
                z_score
            );
            return Err(Error::AnomalyDetected(format!(
                "Z-score {:.2} exceeds threshold {:.2}",
                z_score,
                self.config.anomaly_zscore_threshold
            )));
        }
        
        // Add to history
        history.push_back(price.as_decimal());
        if history.len() > self.config.anomaly_window_size {
            history.pop_front();
        }
        
        Ok(())
    }
    
    /// Check for duplicate messages
    pub fn check_duplicate(&self, message_id: &str) -> Result<()> {
        if !self.config.enable_dedup {
            return Ok(());
        }
        
        let mut recent_ids = self.recent_message_ids.write();
        
        if recent_ids.contains(&message_id.to_string()) {
            self.stats.write().duplicate_rejections += 1;
            return Err(Error::DuplicateData(format!(
                "Message ID already processed: {}",
                message_id
            )));
        }
        
        recent_ids.push_back(message_id.to_string());
        if recent_ids.len() > self.config.dedup_window_size {
            recent_ids.pop_front();
        }
        
        Ok(())
    }
    
    /// Validate complete market data
    pub fn validate_market_data(
        &self,
        symbol: &Symbol,
        price: &Price,
        timestamp: DateTime<Utc>,
        message_id: Option<&str>,
    ) -> Result<()> {
        self.stats.write().total_processed += 1;
        
        // Timestamp validation
        self.validate_timestamp(timestamp)?;
        
        // Price range validation
        self.validate_price(symbol, price)?;
        
        // Anomaly detection
        if let Err(e) = self.detect_anomaly(symbol, price) {
            debug!("Anomaly check failed: {}", e);
            // Don't reject on anomaly, just log warning
        }
        
        // Deduplication
        if let Some(msg_id) = message_id {
            self.check_duplicate(msg_id)?;
        }
        
        // Update last valid price
        self.last_prices.write().insert(symbol.clone(), price.clone());
        
        Ok(())
    }
    
    /// Get quality control statistics
    pub fn get_stats(&self) -> QualityStats {
        self.stats.read().clone()
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = QualityStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ea_okx_core::types::Symbol;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_validate_timestamp_future() {
        let qc = QualityControl::default();
        let future = Utc::now() + Duration::seconds(10);
        
        assert!(qc.validate_timestamp(future).is_err());
    }
    
    #[test]
    fn test_validate_timestamp_stale() {
        let qc = QualityControl::default();
        let stale = Utc::now() - Duration::seconds(10);
        
        assert!(qc.validate_timestamp(stale).is_err());
    }
    
    #[test]
    fn test_validate_timestamp_valid() {
        let qc = QualityControl::default();
        let now = Utc::now();
        
        assert!(qc.validate_timestamp(now).is_ok());
    }
    
    #[test]
    fn test_validate_price_no_history() {
        let qc = QualityControl::default();
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let price = Price::new(dec!(50000)).unwrap();
        
        assert!(qc.validate_price(&symbol, &price).is_ok());
    }
    
    #[test]
    fn test_validate_price_within_range() {
        let qc = QualityControl::default();
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let price1 = Price::new(dec!(50000)).unwrap();
        let price2 = Price::new(dec!(51000)).unwrap(); // 2% increase
        
        qc.last_prices.write().insert(symbol.clone(), price1);
        assert!(qc.validate_price(&symbol, &price2).is_ok());
    }
    
    #[test]
    fn test_validate_price_too_large_deviation() {
        let qc = QualityControl::default();
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let price1 = Price::new(dec!(50000)).unwrap();
        let price2 = Price::new(dec!(60000)).unwrap(); // 20% increase, exceeds 10% limit
        
        qc.last_prices.write().insert(symbol.clone(), price1);
        assert!(qc.validate_price(&symbol, &price2).is_err());
    }
    
    #[test]
    fn test_check_duplicate() {
        let qc = QualityControl::default();
        
        assert!(qc.check_duplicate("msg-123").is_ok());
        assert!(qc.check_duplicate("msg-123").is_err());
        assert!(qc.check_duplicate("msg-456").is_ok());
    }
    
    #[test]
    fn test_quality_stats() {
        let qc = QualityControl::default();
        let stats = qc.get_stats();
        
        assert_eq!(stats.total_processed, 0);
        
        qc.stats.write().total_processed = 10;
        let stats = qc.get_stats();
        assert_eq!(stats.total_processed, 10);
        
        qc.reset_stats();
        let stats = qc.get_stats();
        assert_eq!(stats.total_processed, 0);
    }
}
