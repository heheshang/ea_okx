//! Common types used throughout the trading system

use crate::error::{Error, Result};
use rust_decimal::Decimal as RustDecimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type alias for decimal precision
pub type Decimal = RustDecimal;

/// Trading pair symbol (e.g., BTC-USDT)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(String);

impl Symbol {
    /// Creates a new Symbol after validation
    ///
    /// # Examples
    ///
    /// ```
    /// use ea_okx_core::types::Symbol;
    ///
    /// let symbol = Symbol::new("BTC-USDT").unwrap();
    /// assert_eq!(symbol.as_str(), "BTC-USDT");
    /// ```
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();

        // Validate format: BASE-QUOTE
        if !s.contains('-') {
            return Err(Error::InvalidSymbol(format!(
                "Symbol must contain '-' separator: {}",
                s
            )));
        }

        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidSymbol(format!(
                "Symbol must have exactly one '-' separator: {}",
                s
            )));
        }

        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(Error::InvalidSymbol(format!(
                "Base and quote cannot be empty: {}",
                s
            )));
        }

        Ok(Self(s.to_uppercase()))
    }

    /// Returns the base currency
    pub fn base(&self) -> &str {
        self.0.split('-').next().unwrap()
    }

    /// Returns the quote currency
    pub fn quote(&self) -> &str {
        self.0.split('-').nth(1).unwrap()
    }

    /// Returns the symbol as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Symbol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

/// Price with 8 decimal precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(Decimal);

impl Price {
    /// Creates a new Price
    ///
    /// # Examples
    ///
    /// ```
    /// use ea_okx_core::types::Price;
    /// use rust_decimal_macros::dec;
    ///
    /// let price = Price::new(dec!(42000.50)).unwrap();
    /// assert_eq!(price.as_decimal(), dec!(42000.50));
    /// ```
    pub fn new(value: Decimal) -> Result<Self> {
        if value <= Decimal::ZERO {
            return Err(Error::InvalidPrice(format!(
                "Price must be positive: {}",
                value
            )));
        }
        Ok(Self(value))
    }

    /// Returns the underlying decimal value
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }

    /// Creates a Price from a float (for testing)
    pub fn from_f64(value: f64) -> Result<Self> {
        let decimal = Decimal::from_f64_retain(value)
            .ok_or_else(|| Error::DecimalError(format!("Invalid f64: {}", value)))?;
        Self::new(decimal)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Quantity with 8 decimal precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Quantity(Decimal);

impl Quantity {
    /// Creates a new Quantity
    ///
    /// # Examples
    ///
    /// ```
    /// use ea_okx_core::types::Quantity;
    /// use rust_decimal_macros::dec;
    ///
    /// let qty = Quantity::new(dec!(1.5)).unwrap();
    /// assert_eq!(qty.as_decimal(), dec!(1.5));
    /// ```
    pub fn new(value: Decimal) -> Result<Self> {
        if value < Decimal::ZERO {
            return Err(Error::InvalidQuantity(format!(
                "Quantity cannot be negative: {}",
                value
            )));
        }
        Ok(Self(value))
    }

    /// Returns the underlying decimal value
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }

    /// Creates a Quantity from a float (for testing)
    pub fn from_f64(value: f64) -> Result<Self> {
        let decimal = Decimal::from_f64_retain(value)
            .ok_or_else(|| Error::DecimalError(format!("Invalid f64: {}", value)))?;
        Self::new(decimal)
    }

    /// Checks if quantity is zero
    pub fn is_zero(&self) -> bool {
        self.0 == Decimal::ZERO
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_symbol_new_valid() {
        let symbol = Symbol::new("BTC-USDT").unwrap();
        assert_eq!(symbol.as_str(), "BTC-USDT");
        assert_eq!(symbol.base(), "BTC");
        assert_eq!(symbol.quote(), "USDT");
    }

    #[test]
    fn test_symbol_new_lowercase() {
        let symbol = Symbol::new("btc-usdt").unwrap();
        assert_eq!(symbol.as_str(), "BTC-USDT");
    }

    #[test]
    fn test_symbol_new_invalid_no_separator() {
        let result = Symbol::new("BTCUSDT");
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_new_invalid_multiple_separators() {
        let result = Symbol::new("BTC-USD-T");
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_new_invalid_empty_parts() {
        assert!(Symbol::new("-USDT").is_err());
        assert!(Symbol::new("BTC-").is_err());
    }

    #[test]
    fn test_symbol_from_str() {
        let symbol: Symbol = "ETH-BTC".parse().unwrap();
        assert_eq!(symbol.as_str(), "ETH-BTC");
    }

    #[test]
    fn test_price_new_valid() {
        let price = Price::new(dec!(42000.50)).unwrap();
        assert_eq!(price.as_decimal(), dec!(42000.50));
    }

    #[test]
    fn test_price_new_invalid_zero() {
        let result = Price::new(Decimal::ZERO);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_new_invalid_negative() {
        let result = Price::new(dec!(-100));
        assert!(result.is_err());
    }

    #[test]
    fn test_price_from_f64() {
        let price = Price::from_f64(42000.50).unwrap();
        assert!(price.as_decimal() > dec!(42000.49));
        assert!(price.as_decimal() < dec!(42000.51));
    }

    #[test]
    fn test_quantity_new_valid() {
        let qty = Quantity::new(dec!(1.5)).unwrap();
        assert_eq!(qty.as_decimal(), dec!(1.5));
    }

    #[test]
    fn test_quantity_new_zero_valid() {
        let qty = Quantity::new(Decimal::ZERO).unwrap();
        assert!(qty.is_zero());
    }

    #[test]
    fn test_quantity_new_invalid_negative() {
        let result = Quantity::new(dec!(-1.5));
        assert!(result.is_err());
    }

    #[test]
    fn test_quantity_from_f64() {
        let qty = Quantity::from_f64(1.5).unwrap();
        assert!(qty.as_decimal() > dec!(1.49));
        assert!(qty.as_decimal() < dec!(1.51));
    }

    #[test]
    fn test_price_ordering() {
        let price1 = Price::new(dec!(100)).unwrap();
        let price2 = Price::new(dec!(200)).unwrap();

        assert!(price1 < price2);
        assert!(price2 > price1);
    }

    #[test]
    fn test_quantity_ordering() {
        let qty1 = Quantity::new(dec!(1.0)).unwrap();
        let qty2 = Quantity::new(dec!(2.0)).unwrap();

        assert!(qty1 < qty2);
        assert!(qty2 > qty1);
    }
}
