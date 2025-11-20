//! Order model and related types

use crate::error::{Error, Result};
use crate::types::{Price, Quantity, Symbol};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl FromStr for OrderSide {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "buy" => Ok(OrderSide::Buy),
            "sell" => Ok(OrderSide::Sell),
            _ => Err(Error::InvalidOrderSide(s.to_string())),
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    PostOnly,
    Ioc, // Immediate or Cancel
    Fok, // Fill or Kill
    StopLoss,
    TakeProfit,
    TrailingStop,
    Iceberg,
}

impl FromStr for OrderType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "market" => Ok(OrderType::Market),
            "limit" => Ok(OrderType::Limit),
            "post_only" => Ok(OrderType::PostOnly),
            "ioc" => Ok(OrderType::Ioc),
            "fok" => Ok(OrderType::Fok),
            "stop_loss" => Ok(OrderType::StopLoss),
            "take_profit" => Ok(OrderType::TakeProfit),
            "trailing_stop" => Ok(OrderType::TrailingStop),
            "iceberg" => Ok(OrderType::Iceberg),
            _ => Err(Error::InvalidOrderType(s.to_string())),
        }
    }
}

/// Order status in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Created,
    Submitted,
    Partial,
    Filled,
    Cancelled,
    Rejected,
    Failed,
}

impl FromStr for OrderStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "created" => Ok(OrderStatus::Created),
            "submitted" => Ok(OrderStatus::Submitted),
            "partial" => Ok(OrderStatus::Partial),
            "filled" => Ok(OrderStatus::Filled),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "rejected" => Ok(OrderStatus::Rejected),
            "failed" => Ok(OrderStatus::Failed),
            _ => Err(Error::InvalidOrderStatus(s.to_string())),
        }
    }
}

/// Order entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Internal order ID
    pub id: Uuid,

    /// OKX order ID (after submission)
    pub okx_order_id: Option<String>,

    /// Client-assigned order ID
    pub client_order_id: String,

    /// Strategy ID that created this order
    pub strategy_id: Uuid,

    /// Trading symbol
    pub symbol: Symbol,

    /// Order side
    pub side: OrderSide,

    /// Order type
    pub order_type: OrderType,

    /// Order quantity
    pub quantity: Quantity,

    /// Limit price (None for market orders)
    pub price: Option<Price>,

    /// Average fill price
    pub avg_fill_price: Option<Price>,

    /// Filled quantity
    pub filled_quantity: Quantity,

    /// Order status
    pub status: OrderStatus,

    /// Rejection reason (if rejected)
    pub reject_reason: Option<String>,

    /// Order creation time
    pub created_at: DateTime<Utc>,

    /// Submission time
    pub submitted_at: Option<DateTime<Utc>>,

    /// First fill time
    pub first_fill_at: Option<DateTime<Utc>>,

    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,

    /// Latency from submission to first fill (milliseconds)
    pub latency_ms: Option<i64>,
}

impl Order {
    /// Creates a new order
    ///
    /// # Examples
    ///
    /// ```
    /// use ea_okx_core::models::{Order, OrderSide, OrderType};
    /// use ea_okx_core::types::{Symbol, Quantity};
    /// use uuid::Uuid;
    /// use rust_decimal_macros::dec;
    ///
    /// let order = Order::new(
    ///     Uuid::new_v4(),
    ///     Symbol::new("BTC-USDT").unwrap(),
    ///     OrderSide::Buy,
    ///     OrderType::Market,
    ///     Quantity::new(dec!(0.01)).unwrap(),
    ///     None,
    /// );
    /// ```
    pub fn new(
        strategy_id: Uuid,
        symbol: Symbol,
        side: OrderSide,
        order_type: OrderType,
        quantity: Quantity,
        price: Option<Price>,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();

        Self {
            id,
            okx_order_id: None,
            client_order_id: format!("ord_{}", id.simple()),
            strategy_id,
            symbol,
            side,
            order_type,
            quantity,
            price,
            avg_fill_price: None,
            filled_quantity: Quantity::new(crate::Decimal::ZERO).unwrap(),
            status: OrderStatus::Created,
            reject_reason: None,
            created_at: now,
            submitted_at: None,
            first_fill_at: None,
            completed_at: None,
            latency_ms: None,
        }
    }

    /// Checks if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// Checks if order is terminal (filled, cancelled, rejected, or failed)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Rejected
                | OrderStatus::Failed
        )
    }

    /// Checks if order is active (submitted or partially filled)
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::Submitted | OrderStatus::Partial)
    }

    /// Updates order status
    pub fn set_status(&mut self, status: OrderStatus) {
        self.status = status;

        if self.is_terminal() && self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }

    /// Marks order as submitted
    pub fn mark_submitted(&mut self, okx_order_id: String) {
        self.okx_order_id = Some(okx_order_id);
        self.submitted_at = Some(Utc::now());
        self.status = OrderStatus::Submitted;
    }

    /// Updates fill information
    pub fn update_fill(&mut self, filled_qty: Quantity, avg_price: Price) {
        self.filled_quantity = filled_qty;
        self.avg_fill_price = Some(avg_price);

        if self.first_fill_at.is_none() {
            self.first_fill_at = Some(Utc::now());

            // Calculate latency from submission to first fill
            if let Some(submitted_at) = self.submitted_at {
                let duration = Utc::now().signed_duration_since(submitted_at);
                self.latency_ms = Some(duration.num_milliseconds());
            }
        }

        // Update status based on fill
        if filled_qty >= self.quantity {
            self.status = OrderStatus::Filled;
            self.completed_at = Some(Utc::now());
        } else if filled_qty.as_decimal() > crate::Decimal::ZERO {
            self.status = OrderStatus::Partial;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_order_side_from_str() {
        assert_eq!("buy".parse::<OrderSide>().unwrap(), OrderSide::Buy);
        assert_eq!("SELL".parse::<OrderSide>().unwrap(), OrderSide::Sell);
        assert!("invalid".parse::<OrderSide>().is_err());
    }

    #[test]
    fn test_order_type_from_str() {
        assert_eq!("market".parse::<OrderType>().unwrap(), OrderType::Market);
        assert_eq!("LIMIT".parse::<OrderType>().unwrap(), OrderType::Limit);
        assert_eq!(
            "stop_loss".parse::<OrderType>().unwrap(),
            OrderType::StopLoss
        );
        assert!("invalid".parse::<OrderType>().is_err());
    }

    #[test]
    fn test_order_status_from_str() {
        assert_eq!(
            "created".parse::<OrderStatus>().unwrap(),
            OrderStatus::Created
        );
        assert_eq!(
            "FILLED".parse::<OrderStatus>().unwrap(),
            OrderStatus::Filled
        );
        assert!("invalid".parse::<OrderStatus>().is_err());
    }

    #[test]
    fn test_order_new() {
        let strategy_id = Uuid::new_v4();
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let quantity = Quantity::new(dec!(0.01)).unwrap();

        let order = Order::new(
            strategy_id,
            symbol.clone(),
            OrderSide::Buy,
            OrderType::Market,
            quantity,
            None,
        );

        assert_eq!(order.strategy_id, strategy_id);
        assert_eq!(order.symbol, symbol);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.status, OrderStatus::Created);
        assert!(!order.is_filled());
        assert!(!order.is_terminal());
    }

    #[test]
    fn test_order_lifecycle() {
        let order = Order::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            OrderType::Limit,
            Quantity::new(dec!(0.01)).unwrap(),
            Some(Price::new(dec!(42000)).unwrap()),
        );

        assert_eq!(order.status, OrderStatus::Created);
        assert!(!order.is_active());

        let mut order = order;
        order.mark_submitted("okx123".to_string());
        assert_eq!(order.status, OrderStatus::Submitted);
        assert!(order.is_active());
        assert!(order.okx_order_id.is_some());

        order.update_fill(
            Quantity::new(dec!(0.005)).unwrap(),
            Price::new(dec!(41995)).unwrap(),
        );
        assert_eq!(order.status, OrderStatus::Partial);
        assert!(order.is_active());
        assert!(order.first_fill_at.is_some());

        order.update_fill(
            Quantity::new(dec!(0.01)).unwrap(),
            Price::new(dec!(41998)).unwrap(),
        );
        assert_eq!(order.status, OrderStatus::Filled);
        assert!(order.is_filled());
        assert!(order.is_terminal());
        assert!(!order.is_active());
    }

    #[test]
    fn test_order_serialization() {
        let order = Order::new(
            Uuid::new_v4(),
            Symbol::new("ETH-USDT").unwrap(),
            OrderSide::Sell,
            OrderType::Limit,
            Quantity::new(dec!(1.0)).unwrap(),
            Some(Price::new(dec!(2500)).unwrap()),
        );

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();

        assert_eq!(order.id, deserialized.id);
        assert_eq!(order.symbol.as_str(), deserialized.symbol.as_str());
        assert_eq!(order.side, deserialized.side);
    }
}
