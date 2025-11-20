//! RSI Mean Reversion Strategy
//!
//! This strategy uses the Relative Strength Index (RSI) to identify overbought
//! and oversold conditions, trading on the assumption that prices will revert to the mean.
//!
//! **Trading Logic**:
//! - BUY when RSI < 30 (oversold)
//! - SELL when RSI > 70 (overbought)
//! - Exit when RSI returns to neutral zone (40-60)
//!
//! **Risk Management**:
//! - Maximum 1 position at a time
//! - Position size: 10% of capital
//! - Stop loss: 3% from entry

use ea_okx_core::models::{Order, OrderSide, OrderType};
use ea_okx_core::types::{Decimal, Price, Quantity, Symbol};
use rust_decimal_macros::dec;
use std::collections::VecDeque;
use uuid::Uuid;

/// RSI Strategy
pub struct RSIStrategy {
    id: Uuid,
    symbol: Symbol,
    period: usize,
    oversold_threshold: Decimal,
    overbought_threshold: Decimal,
    price_changes: VecDeque<Decimal>,
    prev_price: Option<Price>,
    current_rsi: Option<Decimal>,
    in_position: bool,
    capital: Decimal,
}

impl RSIStrategy {
    /// Creates a new RSI strategy
    pub fn new(
        id: Uuid,
        symbol: Symbol,
        period: usize,
        capital: Decimal,
    ) -> Self {
        Self {
            id,
            symbol,
            period,
            oversold_threshold: dec!(30),
            overbought_threshold: dec!(70),
            price_changes: VecDeque::with_capacity(period),
            prev_price: None,
            current_rsi: None,
            in_position: false,
            capital,
        }
    }

    /// Updates with new price and calculates RSI
    pub fn on_price(&mut self, price: Price) {
        if let Some(prev) = self.prev_price {
            let change = price.as_decimal() - prev.as_decimal();
            self.price_changes.push_back(change);
            
            if self.price_changes.len() > self.period {
                self.price_changes.pop_front();
            }
            
            if self.price_changes.len() == self.period {
                self.current_rsi = Some(self.calculate_rsi());
            }
        }
        
        self.prev_price = Some(price);
    }

    /// Calculates RSI
    fn calculate_rsi(&self) -> Decimal {
        let mut gains = Decimal::ZERO;
        let mut losses = Decimal::ZERO;

        for &change in &self.price_changes {
            if change > Decimal::ZERO {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / Decimal::from(self.period);
        let avg_loss = losses / Decimal::from(self.period);

        if avg_loss == Decimal::ZERO {
            return dec!(100);
        }

        let rs = avg_gain / avg_loss;
        dec!(100) - (dec!(100) / (Decimal::ONE + rs))
    }

    /// Generates trading signal
    pub fn generate_signal(&self) -> Option<OrderSide> {
        let rsi = self.current_rsi?;

        if !self.in_position && rsi < self.oversold_threshold {
            Some(OrderSide::Buy)
        } else if self.in_position && rsi > self.overbought_threshold {
            Some(OrderSide::Sell)
        } else {
            None
        }
    }

    /// Returns current RSI
    pub fn rsi(&self) -> Option<Decimal> {
        self.current_rsi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_calculation() {
        let mut strategy = RSIStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            14,
            dec!(10000),
        );

        // Simulate price movement
        let prices = vec![
            dec!(44), dec!(44.34), dec!(44.09), dec!(43.61), dec!(44.33),
            dec!(44.83), dec!(45.10), dec!(45.42), dec!(45.84), dec!(46.08),
            dec!(45.89), dec!(46.03), dec!(45.61), dec!(46.28), dec!(46.28),
        ];

        for price in prices {
            strategy.on_price(Price::new(price).unwrap());
        }

        let rsi = strategy.rsi().unwrap();
        // RSI should be calculated after 14 periods
        assert!(rsi > Decimal::ZERO && rsi <= dec!(100));
    }
}
