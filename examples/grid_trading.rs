//! Grid Trading Strategy
//!
//! This strategy places buy and sell orders at regular price intervals (grid levels).
//! It profits from market volatility by buying low and selling high within a range.
//!
//! **Trading Logic**:
//! - Define a price range with upper and lower bounds
//! - Create N grid levels between bounds
//! - Place buy orders below current price and sell orders above
//! - When a buy order fills, place a sell order one level above
//! - When a sell order fills, place a buy order one level below
//!
//! **Parameters**:
//! - Grid levels: Number of price intervals (e.g., 10)
//! - Price range: Upper and lower price bounds
//! - Order size: Fixed size per grid level

use ea_okx_core::models::{Order, OrderSide, OrderType};
use ea_okx_core::types::{Decimal, Price, Quantity, Symbol};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use uuid::Uuid;

/// Grid level
#[derive(Debug, Clone)]
struct GridLevel {
    price: Price,
    order_id: Option<Uuid>,
    filled: bool,
}

/// Grid Trading Strategy
pub struct GridTradingStrategy {
    id: Uuid,
    symbol: Symbol,
    lower_bound: Price,
    upper_bound: Price,
    grid_levels: usize,
    order_size: Quantity,
    levels: Vec<GridLevel>,
    active_orders: HashMap<Uuid, GridLevel>,
}

impl GridTradingStrategy {
    /// Creates a new grid trading strategy
    pub fn new(
        id: Uuid,
        symbol: Symbol,
        lower_bound: Price,
        upper_bound: Price,
        grid_levels: usize,
        order_size: Quantity,
    ) -> Self {
        let mut levels = Vec::new();
        let price_step = (upper_bound.as_decimal() - lower_bound.as_decimal()) 
            / Decimal::from(grid_levels - 1);

        for i in 0..grid_levels {
            let price = lower_bound.as_decimal() + price_step * Decimal::from(i);
            levels.push(GridLevel {
                price: Price::new(price).unwrap(),
                order_id: None,
                filled: false,
            });
        }

        Self {
            id,
            symbol,
            lower_bound,
            upper_bound,
            grid_levels,
            order_size,
            levels,
            active_orders: HashMap::new(),
        }
    }

    /// Initializes grid orders based on current price
    pub fn initialize_grid(&mut self, current_price: Price) -> Vec<Order> {
        let mut orders = Vec::new();

        for level in &mut self.levels {
            // Place buy orders below current price
            if level.price < current_price {
                let order = Order::new(
                    self.id,
                    self.symbol.clone(),
                    OrderSide::Buy,
                    OrderType::Limit,
                    self.order_size,
                    Some(level.price),
                );
                level.order_id = Some(order.id);
                self.active_orders.insert(order.id, level.clone());
                orders.push(order);
            }
            // Place sell orders above current price
            else if level.price > current_price {
                let order = Order::new(
                    self.id,
                    self.symbol.clone(),
                    OrderSide::Sell,
                    OrderType::Limit,
                    self.order_size,
                    Some(level.price),
                );
                level.order_id = Some(order.id);
                self.active_orders.insert(order.id, level.clone());
                orders.push(order);
            }
        }

        orders
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let strategy = GridTradingStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            Price::new(dec!(38000)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
            5,
            Quantity::new(dec!(0.01)).unwrap(),
        );

        assert_eq!(strategy.levels.len(), 5);
        assert_eq!(strategy.levels[0].price.as_decimal(), dec!(38000));
        assert_eq!(strategy.levels[4].price.as_decimal(), dec!(42000));
    }

    #[test]
    fn test_grid_initialization() {
        let mut strategy = GridTradingStrategy::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            Price::new(dec!(38000)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
            5,
            Quantity::new(dec!(0.01)).unwrap(),
        );

        let orders = strategy.initialize_grid(Price::new(dec!(40000)).unwrap());
        
        // Should have orders above and below current price
        assert!(orders.len() > 0);
        
        // Check order sides
        let buy_orders: Vec<_> = orders.iter().filter(|o| o.side == OrderSide::Buy).collect();
        let sell_orders: Vec<_> = orders.iter().filter(|o| o.side == OrderSide::Sell).collect();
        
        assert!(buy_orders.len() > 0);
        assert!(sell_orders.len() > 0);
    }
}
