//! Integration tests for EA OKX Trading System

use ea_okx_core::{Symbol, Price, Quantity};
use rust_decimal_macros::dec;

#[test]
fn test_symbol_creation() {
    let symbol = Symbol::new("BTC-USDT").unwrap();
    assert_eq!(symbol.base(), "BTC");
    assert_eq!(symbol.quote(), "USDT");
}

#[test]
fn test_price_quantity_validation() {
    let price = Price::new(dec!(50000.0)).unwrap();
    let quantity = Quantity::new(dec!(1.5)).unwrap();
    
    assert_eq!(price.as_decimal(), dec!(50000.0));
    assert_eq!(quantity.as_decimal(), dec!(1.5));
}

#[test]
fn test_order_creation() {
    use ea_okx_core::models::{Order, OrderType, OrderSide};
    use uuid::Uuid;
    
    let order = Order::new(
        Uuid::new_v4(),
        Symbol::new("BTC-USDT").unwrap(),
        OrderType::Limit,
        OrderSide::Buy,
        Price::new(dec!(50000.0)).unwrap(),
        Quantity::new(dec!(0.1)).unwrap(),
        None,
    );
    
    assert_eq!(order.symbol.as_str(), "BTC-USDT");
    assert_eq!(order.price.as_decimal(), dec!(50000.0));
}
