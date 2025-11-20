use crate::error::{Error, Result};
use crate::events::Fill;
use ea_okx_core::{Symbol, Price, Quantity};
use ea_okx_core::models::{Order, OrderSide, Position, PositionSide};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Portfolio tracking for backtesting
#[derive(Debug, Clone)]
pub struct Portfolio {
    /// Initial capital
    pub initial_capital: Decimal,
    
    /// Current cash balance
    pub cash: Decimal,
    
    /// Open positions
    pub positions: HashMap<Symbol, Position>,
    
    /// Realized P&L
    pub realized_pnl: Decimal,
    
    /// Total commission paid
    pub total_commission: Decimal,
    
    /// Total slippage incurred
    pub total_slippage: Decimal,
    
    /// Equity curve (timestamp, equity)
    pub equity_curve: Vec<(chrono::DateTime<chrono::Utc>, Decimal)>,
    
    /// Current market prices for positions
    current_prices: HashMap<Symbol, Decimal>,
}

impl Portfolio {
    pub fn new(initial_capital: Decimal) -> Self {
        Self {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            realized_pnl: Decimal::ZERO,
            total_commission: Decimal::ZERO,
            total_slippage: Decimal::ZERO,
            equity_curve: Vec::new(),
            current_prices: HashMap::new(),
        }
    }

    /// Apply a fill to the portfolio
    pub fn apply_fill(&mut self, order: &Order, fill: &Fill) -> Result<()> {
        let cost = fill.price * fill.quantity;
        
        match order.side {
            OrderSide::Buy => {
                // Check if we have enough cash
                let total_cost = cost + fill.commission + fill.slippage;
                if self.cash < total_cost {
                    return Err(Error::ExecutionError(
                        "Insufficient cash for buy order".to_string()
                    ));
                }
                
                // Deduct cash
                self.cash -= total_cost;
                
                // Update or create position
                let position = self.positions.entry(order.symbol.clone())
                    .or_insert_with(|| Position::new(
                        uuid::Uuid::new_v4(), // strategy_id
                        order.symbol.clone(),
                        PositionSide::Long,
                        Quantity::new(Decimal::ZERO).unwrap(),
                        Price::new(Decimal::ZERO).unwrap(),
                    ));
                
                // Update position quantity and average price
                let old_quantity = position.quantity.as_decimal();
                let old_cost = old_quantity * position.avg_entry_price.as_decimal();
                let new_quantity = old_quantity + fill.quantity;
                let new_avg_price = (old_cost + cost) / new_quantity;
                
                position.quantity = Quantity::new(new_quantity)?;
                position.avg_entry_price = Price::new(new_avg_price)?;
            }
            
            OrderSide::Sell => {
                // Check if we have the position to sell
                if let Some(position) = self.positions.get_mut(&order.symbol) {
                    let position_qty = position.quantity.as_decimal();
                    
                    if position_qty < fill.quantity {
                        return Err(Error::ExecutionError(
                            "Insufficient position for sell order".to_string()
                        ));
                    }
                    
                    // Calculate realized PnL
                    let entry_cost = fill.quantity * position.avg_entry_price.as_decimal();
                    let exit_proceeds = cost;
                    let gross_pnl = exit_proceeds - entry_cost;
                    let net_pnl = gross_pnl - fill.commission - fill.slippage;
                    
                    self.realized_pnl += net_pnl;
                    self.cash += exit_proceeds - fill.commission - fill.slippage;
                    
                    // Update position
                    let new_qty = position_qty - fill.quantity;
                    
                    if new_qty <= Decimal::ZERO {
                        // Close position completely
                        self.positions.remove(&order.symbol);
                    } else {
                        // Reduce position
                        position.quantity = Quantity::new(new_qty)?;
                    }
                } else {
                    return Err(Error::ExecutionError(
                        "No position to sell".to_string()
                    ));
                }
            }
        }
        
        // Track costs
        self.total_commission += fill.commission;
        self.total_slippage += fill.slippage;
        
        // Record equity
        let equity = self.total_equity();
        self.equity_curve.push((fill.timestamp, equity));
        
        Ok(())
    }

    /// Update current market prices
    pub fn update_prices(&mut self, prices: &HashMap<Symbol, Decimal>) {
        for (symbol, price) in prices {
            self.current_prices.insert(symbol.clone(), *price);
            
            // Update unrealized PnL for positions
            if let Some(position) = self.positions.get_mut(symbol) {
                if let Ok(price_obj) = Price::new(*price) {
                    position.update_price(price_obj);
                }
            }
        }
    }

    /// Get current position for a symbol
    pub fn get_position(&self, symbol: &Symbol) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Get total equity (cash + unrealized PnL)
    pub fn total_equity(&self) -> Decimal {
        let positions_value: Decimal = self.positions.values()
            .map(|p| {
                let qty = p.quantity.as_decimal();
                let price = p.current_price.as_decimal();
                qty * price
            })
            .sum();
        
        self.cash + positions_value
    }

    /// Get number of open positions
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// Get unrealized PnL
    pub fn unrealized_pnl(&self) -> Decimal {
        self.positions.values()
            .map(|p| p.unrealized_pnl)
            .sum()
    }

    /// Get total PnL
    pub fn total_pnl(&self) -> Decimal {
        self.realized_pnl + self.unrealized_pnl()
    }

    /// Get return percentage
    pub fn return_pct(&self) -> Decimal {
        if self.initial_capital == Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        (self.total_equity() - self.initial_capital) / self.initial_capital
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio::new(dec!(10000.0));
        assert_eq!(portfolio.cash, dec!(10000.0));
        assert_eq!(portfolio.positions.len(), 0);
        assert_eq!(portfolio.total_equity(), dec!(10000.0));
    }

    #[test]
    fn test_buy_order() {
        let mut portfolio = Portfolio::new(dec!(10000.0));
        
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let order = Order::new(
            uuid::Uuid::new_v4(),
            symbol.clone(),
            OrderSide::Buy,
            ea_okx_core::OrderType::Market,
                      ea_okx_core::Quantity::new(dec!(0.1)).unwrap(),
            ea_okx_core::Price::new(dec!(50000.0)).unwrap(),
  
        );
        
        let fill = Fill {
            order_id: order.id,
            price: dec!(50000.0),
            quantity: dec!(0.1),
            commission: dec!(5.0),
            timestamp: chrono::Utc::now(),
            slippage: dec!(2.5),
        };
        portfolio.apply_fill(&order, &fill).unwrap();
        
        // Check cash: 10000 - (50000 * 0.1) - 5 - 2.5 = 4992.5
        assert_eq!(portfolio.cash, dec!(4992.5));
        
        // Check position
        assert_eq!(portfolio.positions.len(), 1);
        let position = portfolio.get_position(&symbol).unwrap();
        assert_eq!(position.quantity.as_decimal(), dec!(0.1));
    }
}