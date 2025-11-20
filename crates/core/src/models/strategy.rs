//! Strategy model and configuration

use crate::error::{Error, Result};
use crate::types::{Decimal, Symbol};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

/// Strategy status in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrategyStatus {
    Draft,
    Validating,
    Backtesting,
    PaperTrading,
    Active,
    Paused,
    Stopped,
    Archived,
}

impl FromStr for StrategyStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(StrategyStatus::Draft),
            "validating" => Ok(StrategyStatus::Validating),
            "backtesting" => Ok(StrategyStatus::Backtesting),
            "papertrading" => Ok(StrategyStatus::PaperTrading),
            "active" => Ok(StrategyStatus::Active),
            "paused" => Ok(StrategyStatus::Paused),
            "stopped" => Ok(StrategyStatus::Stopped),
            "archived" => Ok(StrategyStatus::Archived),
            _ => Err(Error::ValidationError(format!("Invalid strategy status: {}", s))),
        }
    }
}

/// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Strategy parameters (JSON)
    pub parameters: JsonValue,
    
    /// Risk limits
    pub risk_limits: JsonValue,
    
    /// Allowed trading symbols
    pub symbols: Vec<Symbol>,
    
    /// Allocated capital
    pub allocated_capital: Decimal,
    
    /// Maximum position size
    pub max_position_size: Decimal,
    
    /// Maximum leverage
    pub max_leverage: Decimal,
}

impl StrategyConfig {
    /// Creates a new strategy configuration
    pub fn new(
        parameters: JsonValue,
        symbols: Vec<Symbol>,
        allocated_capital: Decimal,
    ) -> Self {
        Self {
            parameters,
            risk_limits: serde_json::json!({}),
            symbols,
            allocated_capital,
            max_position_size: allocated_capital * Decimal::from_str("0.2").unwrap(), // 20% default
            max_leverage: Decimal::from(3), // 3x default
        }
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<()> {
        if self.allocated_capital <= Decimal::ZERO {
            return Err(Error::ValidationError("Allocated capital must be positive".to_string()));
        }
        
        if self.max_leverage <= Decimal::ZERO || self.max_leverage > Decimal::from(10) {
            return Err(Error::ValidationError("Leverage must be between 0 and 10".to_string()));
        }
        
        if self.symbols.is_empty() {
            return Err(Error::ValidationError("At least one symbol is required".to_string()));
        }
        
        Ok(())
    }
}

/// Strategy entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    /// Strategy ID
    pub id: Uuid,
    
    /// Strategy name
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Strategy type
    pub strategy_type: String,
    
    /// Version
    pub version: String,
    
    /// Configuration
    pub config: StrategyConfig,
    
    /// Current status
    pub status: StrategyStatus,
    
    /// Created by user ID
    pub created_by: Uuid,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Last update time
    pub updated_at: DateTime<Utc>,
    
    /// Deployment time
    pub deployed_at: Option<DateTime<Utc>>,
    
    /// Stop time
    pub stopped_at: Option<DateTime<Utc>>,
}

impl Strategy {
    /// Creates a new strategy
    pub fn new(
        name: String,
        strategy_type: String,
        version: String,
        config: StrategyConfig,
        created_by: Uuid,
    ) -> Result<Self> {
        config.validate()?;
        
        let now = Utc::now();
        
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            strategy_type,
            version,
            config,
            status: StrategyStatus::Draft,
            created_by,
            created_at: now,
            updated_at: now,
            deployed_at: None,
            stopped_at: None,
        })
    }

    /// Updates strategy status
    pub fn set_status(&mut self, status: StrategyStatus) {
        self.status = status;
        self.updated_at = Utc::now();
        
        if status == StrategyStatus::Active && self.deployed_at.is_none() {
            self.deployed_at = Some(Utc::now());
        }
        
        if matches!(status, StrategyStatus::Stopped | StrategyStatus::Archived) && self.stopped_at.is_none() {
            self.stopped_at = Some(Utc::now());
        }
    }

    /// Checks if strategy is active
    pub fn is_active(&self) -> bool {
        self.status == StrategyStatus::Active
    }

    /// Checks if strategy can trade
    pub fn can_trade(&self) -> bool {
        matches!(self.status, StrategyStatus::Active | StrategyStatus::PaperTrading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_strategy_status_from_str() {
        assert_eq!("draft".parse::<StrategyStatus>().unwrap(), StrategyStatus::Draft);
        assert_eq!("ACTIVE".parse::<StrategyStatus>().unwrap(), StrategyStatus::Active);
        assert!("invalid".parse::<StrategyStatus>().is_err());
    }

    #[test]
    fn test_strategy_config_new() {
        let symbols = vec![Symbol::new("BTC-USDT").unwrap()];
        let config = StrategyConfig::new(
            serde_json::json!({"period": 20}),
            symbols.clone(),
            dec!(10000),
        );
        
        assert_eq!(config.symbols, symbols);
        assert_eq!(config.allocated_capital, dec!(10000));
        assert_eq!(config.max_position_size, dec!(2000)); // 20%
        assert_eq!(config.max_leverage, dec!(3));
    }

    #[test]
    fn test_strategy_config_validate() {
        let symbols = vec![Symbol::new("BTC-USDT").unwrap()];
        let config = StrategyConfig::new(
            serde_json::json!({}),
            symbols,
            dec!(10000),
        );
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_strategy_config_validate_invalid_capital() {
        let symbols = vec![Symbol::new("BTC-USDT").unwrap()];
        let mut config = StrategyConfig::new(
            serde_json::json!({}),
            symbols,
            dec!(10000),
        );
        
        config.allocated_capital = Decimal::ZERO;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_strategy_new() {
        let symbols = vec![Symbol::new("BTC-USDT").unwrap()];
        let config = StrategyConfig::new(
            serde_json::json!({"period": 20}),
            symbols,
            dec!(10000),
        );
        
        let strategy = Strategy::new(
            "MA Crossover".to_string(),
            "trend_following".to_string(),
            "1.0.0".to_string(),
            config,
            Uuid::new_v4(),
        ).unwrap();
        
        assert_eq!(strategy.name, "MA Crossover");
        assert_eq!(strategy.status, StrategyStatus::Draft);
        assert!(!strategy.is_active());
        assert!(!strategy.can_trade());
    }

    #[test]
    fn test_strategy_lifecycle() {
        let symbols = vec![Symbol::new("BTC-USDT").unwrap()];
        let config = StrategyConfig::new(
            serde_json::json!({}),
            symbols,
            dec!(10000),
        );
        
        let mut strategy = Strategy::new(
            "Test Strategy".to_string(),
            "test".to_string(),
            "1.0.0".to_string(),
            config,
            Uuid::new_v4(),
        ).unwrap();
        
        assert_eq!(strategy.status, StrategyStatus::Draft);
        
        strategy.set_status(StrategyStatus::Backtesting);
        assert_eq!(strategy.status, StrategyStatus::Backtesting);
        
        strategy.set_status(StrategyStatus::Active);
        assert!(strategy.is_active());
        assert!(strategy.can_trade());
        assert!(strategy.deployed_at.is_some());
        
        strategy.set_status(StrategyStatus::Paused);
        assert!(!strategy.is_active());
        assert!(!strategy.can_trade());
        
        strategy.set_status(StrategyStatus::Stopped);
        assert!(strategy.stopped_at.is_some());
    }
}
