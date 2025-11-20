//! Trade record model

use crate::models::{OrderSide, OrderType};
use crate::types::{Decimal, Price, Quantity, Symbol};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trade record - represents a completed trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Trade ID
    pub id: Uuid,

    /// OKX order ID
    pub okx_order_id: Option<String>,

    /// Client order ID
    pub client_order_id: String,

    /// Strategy ID
    pub strategy_id: Uuid,

    /// Trading symbol
    pub symbol: Symbol,

    /// Order side
    pub side: OrderSide,

    /// Order type
    pub order_type: OrderType,

    /// Executed quantity
    pub quantity: Quantity,

    /// Execution price
    pub price: Price,

    /// Trading commission
    pub commission: Decimal,

    /// Commission asset
    pub commission_asset: String,

    /// Realized profit/loss
    pub realized_pnl: Option<Decimal>,

    /// Slippage in basis points
    pub slippage_bps: Option<i32>,

    /// Execution timestamp
    pub executed_at: DateTime<Utc>,

    /// Latency from signal to execution (milliseconds)
    pub latency_ms: Option<i64>,
}

impl Trade {
    /// Creates a new trade record
    pub fn new(
        strategy_id: Uuid,
        client_order_id: String,
        symbol: Symbol,
        side: OrderSide,
        order_type: OrderType,
        quantity: Quantity,
        price: Price,
        commission: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            okx_order_id: None,
            client_order_id,
            strategy_id,
            symbol,
            side,
            order_type,
            quantity,
            price,
            commission,
            commission_asset: "USDT".to_string(),
            realized_pnl: None,
            slippage_bps: None,
            executed_at: Utc::now(),
            latency_ms: None,
        }
    }

    /// Returns the trade value (quantity * price)
    pub fn trade_value(&self) -> Decimal {
        self.quantity.as_decimal() * self.price.as_decimal()
    }

    /// Returns net value after commission
    pub fn net_value(&self) -> Decimal {
        let gross = self.trade_value();
        match self.side {
            OrderSide::Buy => gross + self.commission,
            OrderSide::Sell => gross - self.commission,
        }
    }

    /// Calculates effective price including commission
    pub fn effective_price(&self) -> Decimal {
        self.net_value() / self.quantity.as_decimal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_trade_new() {
        let strategy_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let quantity = Quantity::new(dec!(0.1)).unwrap();
        let price = Price::new(dec!(42000)).unwrap();

        let trade = Trade::new(
            strategy_id,
            "ord_123".to_string(),
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Market,
            quantity,
            price,
            dec!(4.2),
        );

        assert_eq!(trade.strategy_id, strategy_id);
        assert_eq!(trade.symbol, symbol);
        assert_eq!(trade.side, OrderSide::Buy);
        assert_eq!(trade.commission, dec!(4.2));
    }

    #[test]
    fn test_trade_value() {
        let trade = Trade::new(
            Uuid::new_v4(),
            "ord_123".to_string(),
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
            dec!(4.2),
        );

        // 0.1 * 42000 = 4200
        assert_eq!(trade.trade_value(), dec!(4200));
    }

    #[test]
    fn test_trade_net_value_buy() {
        let trade = Trade::new(
            Uuid::new_v4(),
            "ord_123".to_string(),
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
            dec!(4.2),
        );

        // Buy: gross + commission = 4200 + 4.2 = 4204.2
        assert_eq!(trade.net_value(), dec!(4204.2));
    }

    #[test]
    fn test_trade_net_value_sell() {
        let trade = Trade::new(
            Uuid::new_v4(),
            "ord_123".to_string(),
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Sell,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
            dec!(4.2),
        );

        // Sell: gross - commission = 4200 - 4.2 = 4195.8
        assert_eq!(trade.net_value(), dec!(4195.8));
    }

    #[test]
    fn test_trade_effective_price() {
        let trade = Trade::new(
            Uuid::new_v4(),
            "ord_123".to_string(),
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Market,
            Quantity::new(dec!(0.1)).unwrap(),
            Price::new(dec!(42000)).unwrap(),
            dec!(4.2),
        );

        // Effective price = 4204.2 / 0.1 = 42042
        assert_eq!(trade.effective_price(), dec!(42042));
    }

    #[test]
    fn test_trade_serialization() {
        let trade = Trade::new(
            Uuid::new_v4(),
            "ord_123".to_string(),
            Symbol::new("ETH-USDT").unwrap(),
            OrderSide::Sell,
            OrderType::Limit,
            Quantity::new(dec!(1.0)).unwrap(),
            Price::new(dec!(2500)).unwrap(),
            dec!(2.5),
        );

        let json = serde_json::to_string(&trade).unwrap();
        let deserialized: Trade = serde_json::from_str(&json).unwrap();

        assert_eq!(trade.id, deserialized.id);
        assert_eq!(trade.symbol.as_str(), deserialized.symbol.as_str());
        assert_eq!(trade.side, deserialized.side);
    }
}
