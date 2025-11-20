use ea_okx_core::models::{OrderSide, OrderType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Commission model for calculating trading fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionModel {
    /// Maker fee rate (e.g., 0.001 for 0.1%)
    pub maker_rate: Decimal,
    
    /// Taker fee rate (e.g., 0.0015 for 0.15%)
    pub taker_rate: Decimal,
    
    /// Minimum commission per trade
    pub min_commission: Decimal,
}

impl Default for CommissionModel {
    fn default() -> Self {
        Self {
            maker_rate: dec!(0.001),  // 0.1% maker fee
            taker_rate: dec!(0.0015), // 0.15% taker fee
            min_commission: dec!(0.0),
        }
    }
}

impl CommissionModel {
    pub fn okx_spot() -> Self {
        Self {
            maker_rate: dec!(0.001),
            taker_rate: dec!(0.0015),
            min_commission: dec!(0.0),
        }
    }

    pub fn okx_futures() -> Self {
        Self {
            maker_rate: dec!(0.0002),
            taker_rate: dec!(0.0005),
            min_commission: dec!(0.0),
        }
    }

    /// Calculate commission for a trade
    pub fn calculate(
        &self,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Decimal {
        let notional = price * quantity;
        
        let rate = match order_type {
            OrderType::Limit | OrderType::PostOnly => self.maker_rate,
            OrderType::Market | OrderType::Ioc | OrderType::Fok => self.taker_rate,
            // Conditional orders use taker rate when triggered
            OrderType::StopLoss | OrderType::TakeProfit | OrderType::TrailingStop | OrderType::Iceberg => self.taker_rate,
        };
        
        let commission = notional * rate;
        commission.max(self.min_commission)
    }
}

/// Slippage model for simulating realistic execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageModel {
    /// Fixed slippage in basis points (e.g., 5 = 0.05%)
    pub fixed_bps: Decimal,
    
    /// Price impact coefficient (linear model)
    pub impact_coefficient: Decimal,
    
    /// Minimum slippage amount
    pub min_slippage: Decimal,
}

impl Default for SlippageModel {
    fn default() -> Self {
        Self {
            fixed_bps: dec!(5.0),      // 5 basis points = 0.05%
            impact_coefficient: dec!(0.0001),
            min_slippage: dec!(0.0),
        }
    }
}

impl SlippageModel {
    pub fn conservative() -> Self {
        Self {
            fixed_bps: dec!(10.0),     // 10 bps = 0.1%
            impact_coefficient: dec!(0.0002),
            min_slippage: dec!(0.0),
        }
    }

    pub fn aggressive() -> Self {
        Self {
            fixed_bps: dec!(3.0),      // 3 bps = 0.03%
            impact_coefficient: dec!(0.00005),
            min_slippage: dec!(0.0),
        }
    }

    /// Calculate slippage for a market order
    pub fn calculate_market(
        &self,
        side: OrderSide,
        price: Decimal,
        quantity: Decimal,
        avg_volume: Decimal,
    ) -> Decimal {
        // Fixed component
        let fixed_slippage = price * self.fixed_bps / dec!(10000.0);
        
        // Impact component (based on order size relative to avg volume)
        let volume_ratio = if avg_volume > Decimal::ZERO {
            quantity / avg_volume
        } else {
            Decimal::ZERO
        };
        
        let impact_slippage = price * self.impact_coefficient * volume_ratio;
        
        // Total slippage (always unfavorable)
        let total_slippage = fixed_slippage + impact_slippage;
        
        total_slippage.max(self.min_slippage)
    }

    /// Calculate slippage for a limit order (typically zero for filled limits)
    pub fn calculate_limit(
        &self,
        _side: OrderSide,
        _price: Decimal,
        _quantity: Decimal,
    ) -> Decimal {
        // Limit orders that get filled typically have no slippage
        // They execute at the specified price or better
        Decimal::ZERO
    }

    /// Apply slippage to a price
    pub fn apply_slippage(
        &self,
        side: OrderSide,
        price: Decimal,
        slippage: Decimal,
    ) -> Decimal {
        match side {
            OrderSide::Buy => price + slippage,   // Buy higher
            OrderSide::Sell => price - slippage,  // Sell lower
        }
    }
}

/// Combined cost model including commission and slippage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    pub commission: CommissionModel,
    pub slippage: SlippageModel,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            commission: CommissionModel::default(),
            slippage: SlippageModel::default(),
        }
    }
}

impl CostModel {
    pub fn okx_spot_conservative() -> Self {
        Self {
            commission: CommissionModel::okx_spot(),
            slippage: SlippageModel::conservative(),
        }
    }

    pub fn okx_futures_aggressive() -> Self {
        Self {
            commission: CommissionModel::okx_futures(),
            slippage: SlippageModel::aggressive(),
        }
    }

    /// Calculate total cost for a trade
    pub fn calculate_total_cost(
        &self,
        order_type: OrderType,
        side: OrderSide,
        price: Decimal,
        quantity: Decimal,
        avg_volume: Decimal,
    ) -> (Decimal, Decimal, Decimal) {
        // Calculate commission
        let commission = self.commission.calculate(order_type, price, quantity);
        
        // Calculate slippage
        let slippage = match order_type {
            OrderType::Market => {
                self.slippage.calculate_market(side, price, quantity, avg_volume)
            }
            _ => {
                self.slippage.calculate_limit(side, price, quantity)
            }
        };
        
        // Calculate effective execution price
        let execution_price = self.slippage.apply_slippage(side, price, slippage);
        
        (execution_price, commission, slippage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commission_calculation() {
        let model = CommissionModel::okx_spot();
        
        // Maker order: 0.1%
        let commission = model.calculate(
            OrderType::Limit,
            dec!(50000.0),  // $50,000
            dec!(1.0),      // 1 BTC
        );
        assert_eq!(commission, dec!(50.0)); // $50 commission
        
        // Taker order: 0.15%
        let commission = model.calculate(
            OrderType::Market,
            dec!(50000.0),
            dec!(1.0),
        );
        assert_eq!(commission, dec!(75.0)); // $75 commission
    }

    #[test]
    fn test_slippage_calculation() {
        let model = SlippageModel::default();
        
        // Market order with 10% volume
        let slippage = model.calculate_market(
            OrderSide::Buy,
            dec!(50000.0),
            dec!(0.1),      // 0.1 BTC
            dec!(1.0),      // 1 BTC avg volume
        );
        
        // Fixed: 50000 * 0.0005 = 25
        // Impact: 50000 * 0.0001 * 0.1 = 0.5
        // Total: 25.5
        assert!(slippage > dec!(25.0) && slippage < dec!(26.0));
    }

    #[test]
    fn test_apply_slippage() {
        let model = SlippageModel::default();
        
        // Buy side - price increases
        let buy_price = model.apply_slippage(
            OrderSide::Buy,
            dec!(50000.0),
            dec!(10.0),
        );
        assert_eq!(buy_price, dec!(50010.0));
        
        // Sell side - price decreases
        let sell_price = model.apply_slippage(
            OrderSide::Sell,
            dec!(50000.0),
            dec!(10.0),
        );
        assert_eq!(sell_price, dec!(49990.0));
    }

    #[test]
    fn test_total_cost() {
        let model = CostModel::okx_spot_conservative();
        
        let (exec_price, commission, slippage) = model.calculate_total_cost(
            OrderType::Market,
            OrderSide::Buy,
            dec!(50000.0),
            dec!(1.0),
            dec!(10.0),
        );
        
        // Should have commission and slippage
        assert!(commission > Decimal::ZERO);
        assert!(slippage > Decimal::ZERO);
        assert!(exec_price > dec!(50000.0)); // Buy side increases
    }
}
