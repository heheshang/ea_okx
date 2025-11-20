//! Simple Moving Average Crossover Strategy
//!
//! This example demonstrates a basic trend-following strategy using two moving averages:
//! - Fast MA (20 periods)
//! - Slow MA (50 periods)
//!
//! **Trading Logic**:
//! - BUY signal when fast MA crosses above slow MA (golden cross)
//! - SELL signal when fast MA crosses below slow MA (death cross)
//!
//! **Risk Management**:
//! - Maximum position size: 20% of allocated capital
//! - Stop loss: 2% below entry
//! - Take profit: 5% above entry

use ea_okx_core::models::{Order, OrderSide, OrderType, Strategy, StrategyConfig};
use ea_okx_core::types::{Decimal, Price, Quantity, Symbol};
use ea_okx_core::Result;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACrossoverParams {
    /// Fast moving average period
    pub fast_period: usize,
    
    /// Slow moving average period
    pub slow_period: usize,
    
    /// Stop loss percentage
    pub stop_loss_pct: Decimal,
    
    /// Take profit percentage
    pub take_profit_pct: Decimal,
    
    /// Position size as percentage of capital
    pub position_size_pct: Decimal,
}

impl Default for MACrossoverParams {
    fn default() -> Self {
        Self {
            fast_period: 20,
            slow_period: 50,
            stop_loss_pct: dec!(0.02),   // 2%
            take_profit_pct: dec!(0.05),  // 5%
            position_size_pct: dec!(0.20), // 20%
        }
    }
}

/// Trading signal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Simple Moving Average Crossover Strategy
pub struct MACrossoverStrategy {
    /// Strategy ID
    id: Uuid,
    
    /// Trading symbol
    symbol: Symbol,
    
    /// Strategy parameters
    params: MACrossoverParams,
    
    /// Price history for fast MA
    fast_prices: VecDeque<Decimal>,
    
    /// Price history for slow MA
    slow_prices: VecDeque<Decimal>,
    
    /// Previous fast MA value
    prev_fast_ma: Option<Decimal>,
    
    /// Previous slow MA value
    prev_slow_ma: Option<Decimal>,
    
    /// Current position
    current_position: Option<Position>,
    
    /// Allocated capital
    capital: Decimal,
}

/// Position tracking
#[derive(Debug, Clone)]
struct Position {
    side: OrderSide,
    entry_price: Price,
    quantity: Quantity,
    stop_loss: Price,
    take_profit: Price,
}

impl MACrossoverStrategy {
    /// Creates a new MA Crossover strategy
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use ea_okx_core::types::Symbol;
    /// use rust_decimal_macros::dec;
    ///
    /// # fn main() -> ea_okx_core::Result<()> {
    /// let strategy = simple_ma_crossover::MACrossoverStrategy::new(
    ///     Uuid::new_v4(),
    ///     Symbol::new("BTC-USDT")?,
    ///     simple_ma_crossover::MACrossoverParams::default(),
    ///     dec!(10000),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        id: Uuid,
        symbol: Symbol,
        params: MACrossoverParams,
        capital: Decimal,
    ) -> Self {
        Self {
            id,
            symbol,
            params,
            fast_prices: VecDeque::with_capacity(params.fast_period),
            slow_prices: VecDeque::with_capacity(params.slow_period),
            prev_fast_ma: None,
            prev_slow_ma: None,
            current_position: None,
            capital,
        }
    }

    /// Updates strategy with new price data
    pub fn on_price(&mut self, price: Price) {
        let price_decimal = price.as_decimal();
        
        // Update fast MA buffer
        self.fast_prices.push_back(price_decimal);
        if self.fast_prices.len() > self.params.fast_period {
            self.fast_prices.pop_front();
        }
        
        // Update slow MA buffer
        self.slow_prices.push_back(price_decimal);
        if self.slow_prices.len() > self.params.slow_period {
            self.slow_prices.pop_front();
        }
    }

    /// Calculates moving average
    fn calculate_ma(&self, prices: &VecDeque<Decimal>) -> Option<Decimal> {
        if prices.is_empty() {
            return None;
        }
        
        let sum: Decimal = prices.iter().sum();
        Some(sum / Decimal::from(prices.len()))
    }

    /// Generates trading signal
    pub fn generate_signal(&mut self) -> Signal {
        // Need enough data for both MAs
        if self.fast_prices.len() < self.params.fast_period 
            || self.slow_prices.len() < self.params.slow_period {
            return Signal::Hold;
        }

        let fast_ma = self.calculate_ma(&self.fast_prices).unwrap();
        let slow_ma = self.calculate_ma(&self.slow_prices).unwrap();

        // Check for crossover
        let signal = if let (Some(prev_fast), Some(prev_slow)) = (self.prev_fast_ma, self.prev_slow_ma) {
            // Golden cross: fast MA crosses above slow MA
            if prev_fast <= prev_slow && fast_ma > slow_ma {
                Signal::Buy
            }
            // Death cross: fast MA crosses below slow MA
            else if prev_fast >= prev_slow && fast_ma < slow_ma {
                Signal::Sell
            } else {
                Signal::Hold
            }
        } else {
            Signal::Hold
        };

        // Update previous values
        self.prev_fast_ma = Some(fast_ma);
        self.prev_slow_ma = Some(slow_ma);

        signal
    }

    /// Creates an order based on signal
    pub fn create_order(&self, signal: Signal, current_price: Price) -> Option<Order> {
        match signal {
            Signal::Buy if self.current_position.is_none() => {
                // Calculate position size
                let position_value = self.capital * self.params.position_size_pct;
                let quantity = position_value / current_price.as_decimal();
                
                Some(Order::new(
                    self.id,
                    self.symbol.clone(),
                    OrderSide::Buy,
                    OrderType::Market,
                    Quantity::new(quantity).ok()?,
                    None,
                ))
            }
            Signal::Sell if self.current_position.is_some() => {
                let position = self.current_position.as_ref()?;
                
                Some(Order::new(
                    self.id,
                    self.symbol.clone(),
                    OrderSide::Sell,
                    OrderType::Market,
                    position.quantity,
                    None,
                ))
            }
            _ => None,
        }
    }

    /// Updates position after order fill
    pub fn on_order_filled(&mut self, order: &Order, fill_price: Price) {
        match order.side {
            OrderSide::Buy => {
                // Calculate stop loss and take profit
                let stop_loss_price = fill_price.as_decimal() * (Decimal::ONE - self.params.stop_loss_pct);
                let take_profit_price = fill_price.as_decimal() * (Decimal::ONE + self.params.take_profit_pct);
                
                self.current_position = Some(Position {
                    side: OrderSide::Buy,
                    entry_price: fill_price,
                    quantity: order.quantity,
                    stop_loss: Price::new(stop_loss_price).unwrap(),
                    take_profit: Price::new(take_profit_price).unwrap(),
                });
            }
            OrderSide::Sell => {
                self.current_position = None;
            }
        }
    }

    /// Checks if stop loss or take profit is hit
    pub fn check_exit_conditions(&self, current_price: Price) -> bool {
        if let Some(position) = &self.current_position {
            current_price <= position.stop_loss || current_price >= position.take_profit
        } else {
            false
        }
    }

    /// Returns current position
    pub fn position(&self) -> Option<&Position> {
        self.current_position.as_ref()
    }

    /// Returns unrealized PnL if in position
    pub fn unrealized_pnl(&self, current_price: Price) -> Option<Decimal> {
        self.current_position.as_ref().map(|pos| {
            let price_diff = current_price.as_decimal() - pos.entry_price.as_decimal();
            price_diff * pos.quantity.as_decimal()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = MACrossoverStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            MACrossoverParams::default(),
            dec!(10000),
        );
        
        assert_eq!(strategy.capital, dec!(10000));
        assert!(strategy.current_position.is_none());
    }

    #[test]
    fn test_ma_calculation() {
        let mut strategy = MACrossoverStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            MACrossoverParams {
                fast_period: 3,
                slow_period: 5,
                ..Default::default()
            },
            dec!(10000),
        );

        // Add prices
        strategy.on_price(Price::new(dec!(100)).unwrap());
        strategy.on_price(Price::new(dec!(110)).unwrap());
        strategy.on_price(Price::new(dec!(120)).unwrap());

        let ma = strategy.calculate_ma(&strategy.fast_prices).unwrap();
        // (100 + 110 + 120) / 3 = 110
        assert_eq!(ma, dec!(110));
    }

    #[test]
    fn test_golden_cross_signal() {
        let mut strategy = MACrossoverStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            MACrossoverParams {
                fast_period: 2,
                slow_period: 3,
                ..Default::default()
            },
            dec!(10000),
        );

        // Initial prices - slow MA above fast MA
        strategy.on_price(Price::new(dec!(90)).unwrap());
        strategy.on_price(Price::new(dec!(95)).unwrap());
        strategy.on_price(Price::new(dec!(100)).unwrap());
        let _ = strategy.generate_signal(); // Initialize prev values

        // Price surge - fast MA crosses above slow MA
        strategy.on_price(Price::new(dec!(110)).unwrap());
        let signal = strategy.generate_signal();
        
        assert_eq!(signal, Signal::Buy);
    }

    #[test]
    fn test_position_tracking() {
        let mut strategy = MACrossoverStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            MACrossoverParams::default(),
            dec!(10000),
        );

        let order = Order::new(
            strategy.id,
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            None,
        );

        let fill_price = Price::new(dec!(40000)).unwrap();
        strategy.on_order_filled(&order, fill_price);

        assert!(strategy.current_position.is_some());
        let position = strategy.position().unwrap();
        assert_eq!(position.entry_price, fill_price);
        
        // Stop loss should be 2% below
        let expected_stop = dec!(40000) * dec!(0.98);
        assert_eq!(position.stop_loss.as_decimal(), expected_stop);
        
        // Take profit should be 5% above
        let expected_tp = dec!(40000) * dec!(1.05);
        assert_eq!(position.take_profit.as_decimal(), expected_tp);
    }

    #[test]
    fn test_exit_conditions() {
        let mut strategy = MACrossoverStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            MACrossoverParams::default(),
            dec!(10000),
        );

        let order = Order::new(
            strategy.id,
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            None,
        );

        strategy.on_order_filled(&order, Price::new(dec!(40000)).unwrap());

        // Should exit if price hits stop loss
        assert!(strategy.check_exit_conditions(Price::new(dec!(39200)).unwrap()));
        
        // Should exit if price hits take profit
        assert!(strategy.check_exit_conditions(Price::new(dec!(42000)).unwrap()));
        
        // Should not exit if price in range
        assert!(!strategy.check_exit_conditions(Price::new(dec!(40500)).unwrap()));
    }

    #[test]
    fn test_unrealized_pnl() {
        let mut strategy = MACrossoverStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            MACrossoverParams::default(),
            dec!(10000),
        );

        let order = Order::new(
            strategy.id,
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            None,
        );

        strategy.on_order_filled(&order, Price::new(dec!(40000)).unwrap());

        // Price goes up 5%
        let pnl = strategy.unrealized_pnl(Price::new(dec!(42000)).unwrap()).unwrap();
        // (42000 - 40000) * 0.1 = 200
        assert_eq!(pnl, dec!(200));
    }
}
