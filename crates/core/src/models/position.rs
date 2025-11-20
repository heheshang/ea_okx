//! Position model and related types

use crate::error::{Error, Result};
use crate::types::{Decimal, Price, Quantity, Symbol};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Position side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionSide {
    Long,
    Short,
    Net,
}

impl FromStr for PositionSide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "long" => Ok(PositionSide::Long),
            "short" => Ok(PositionSide::Short),
            "net" => Ok(PositionSide::Net),
            _ => Err(Error::InvalidPositionSide(s.to_string())),
        }
    }
}

/// Position entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Position ID
    pub id: Uuid,
    
    /// Strategy ID
    pub strategy_id: Uuid,
    
    /// Trading symbol
    pub symbol: Symbol,
    
    /// Position side
    pub side: PositionSide,
    
    /// Position quantity
    pub quantity: Quantity,
    
    /// Average entry price
    pub avg_entry_price: Price,
    
    /// Current market price
    pub current_price: Price,
    
    /// Unrealized profit/loss
    pub unrealized_pnl: Decimal,
    
    /// Realized profit/loss
    pub realized_pnl: Decimal,
    
    /// Margin requirement
    pub margin: Option<Decimal>,
    
    /// Leverage ratio
    pub leverage: Option<Decimal>,
    
    /// Liquidation price
    pub liquidation_price: Option<Price>,
    
    /// Position open time
    pub opened_at: DateTime<Utc>,
    
    /// Last update time
    pub last_updated: DateTime<Utc>,
}

impl Position {
    /// Creates a new position
    pub fn new(
        strategy_id: Uuid,
        symbol: Symbol,
        side: PositionSide,
        quantity: Quantity,
        entry_price: Price,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            strategy_id,
            symbol,
            side,
            quantity,
            avg_entry_price: entry_price,
            current_price: entry_price,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            margin: None,
            leverage: None,
            liquidation_price: None,
            opened_at: now,
            last_updated: now,
        }
    }

    /// Updates current price and recalculates unrealized PnL
    pub fn update_price(&mut self, current_price: Price) {
        self.current_price = current_price;
        self.unrealized_pnl = self.calculate_unrealized_pnl();
        self.last_updated = Utc::now();
    }

    /// Calculates unrealized profit/loss
    pub fn calculate_unrealized_pnl(&self) -> Decimal {
        let price_diff = self.current_price.as_decimal() - self.avg_entry_price.as_decimal();
        let qty = self.quantity.as_decimal();
        
        match self.side {
            PositionSide::Long => price_diff * qty,
            PositionSide::Short => -price_diff * qty,
            PositionSide::Net => price_diff * qty,
        }
    }

    /// Checks if position is closed
    pub fn is_closed(&self) -> bool {
        self.quantity.is_zero()
    }

    /// Returns position value at current price
    pub fn position_value(&self) -> Decimal {
        self.current_price.as_decimal() * self.quantity.as_decimal()
    }

    /// Returns position value at entry
    pub fn entry_value(&self) -> Decimal {
        self.avg_entry_price.as_decimal() * self.quantity.as_decimal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_position_side_from_str() {
        assert_eq!("long".parse::<PositionSide>().unwrap(), PositionSide::Long);
        assert_eq!("SHORT".parse::<PositionSide>().unwrap(), PositionSide::Short);
        assert_eq!("net".parse::<PositionSide>().unwrap(), PositionSide::Net);
        assert!("invalid".parse::<PositionSide>().is_err());
    }

    #[test]
    fn test_position_new() {
        let strategy_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let quantity = Quantity::new(dec!(0.1)).unwrap();
        let entry_price = Price::new(dec!(42000)).unwrap();
        
        let position = Position::new(strategy_id, symbol.clone(), PositionSide::Long, quantity, entry_price);
        
        assert_eq!(position.strategy_id, strategy_id);
        assert_eq!(position.symbol, symbol);
        assert_eq!(position.side, PositionSide::Long);
        assert_eq!(position.quantity, quantity);
        assert_eq!(position.unrealized_pnl, Decimal::ZERO);
    }

    #[test]
    fn test_position_long_pnl() {
        let position = Position::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            PositionSide::Long,
            Quantity::new(dec!(0.1)).unwrap(),
            Price::new(dec!(40000)).unwrap(),
        );
        
        let mut position = position;
        position.update_price(Price::new(dec!(42000)).unwrap());
        
        // Long position: (42000 - 40000) * 0.1 = 200
        assert_eq!(position.unrealized_pnl, dec!(200));
    }

    #[test]
    fn test_position_short_pnl() {
        let position = Position::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            PositionSide::Short,
            Quantity::new(dec!(0.1)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
        );
        
        let mut position = position;
        position.update_price(Price::new(dec!(40000)).unwrap());
        
        // Short position: -(40000 - 42000) * 0.1 = 200
        assert_eq!(position.unrealized_pnl, dec!(200));
    }

    #[test]
    fn test_position_value() {
        let position = Position::new(
            Uuid::new_v4(),
            Symbol::new("ETH-USDT").unwrap(),
            PositionSide::Long,
            Quantity::new(dec!(5.0)).unwrap(),
            Price::new(dec!(2500)).unwrap(),
        );
        
        assert_eq!(position.entry_value(), dec!(12500));
        assert_eq!(position.position_value(), dec!(12500));
        
        let mut position = position;
        position.update_price(Price::new(dec!(2600)).unwrap());
        assert_eq!(position.position_value(), dec!(13000));
    }
}
