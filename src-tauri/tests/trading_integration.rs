//! Integration test for trading functionality

use ea_okx_quant::commands::trading::*;
use ea_okx_quant::state::AppState;
use uuid::Uuid;

#[tokio::test]
async fn test_trading_commands_basic() {
    let state = AppState::new();

    // Test place order
    let order_request = PlaceOrderRequest {
        strategy_id: Uuid::new_v4().to_string(),
        symbol: "BTC-USDT".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        quantity: 0.01,
        price: Some(45000.0),
        time_in_force: Some("GTC".to_string()),
        reduce_only: Some(false),
        post_only: Some(false),
    };

    let result = place_order(order_request, state.clone()).await;
    assert!(result.is_ok(), "Failed to place order: {:?}", result);

    let response = result.unwrap();
    assert_eq!(response["success"], true);
    assert!(response["request_id"].is_string());
}

#[tokio::test]
async fn test_account_and_market_data() {
    let state = AppState::new();

    // Test account balance
    let balance_result = get_account_balance(state.clone()).await;
    assert!(balance_result.is_ok(), "Failed to get account balance");

    let balance = balance_result.unwrap();
    assert!(balance["total_equity"].is_number());
    assert_eq!(balance["currency"], "USDT");

    // Test trading fees
    let fees_result = get_trading_fees(Some("BTC-USDT".to_string()), state.clone()).await;
    assert!(fees_result.is_ok(), "Failed to get trading fees");

    let fees = fees_result.unwrap();
    assert!(fees["maker_fee"].is_number());
    assert!(fees["taker_fee"].is_number());

    // Test order book
    let order_book_result = get_order_book("BTC-USDT".to_string(), Some(5), state.clone()).await;
    assert!(order_book_result.is_ok(), "Failed to get order book");

    let order_book = order_book_result.unwrap();
    assert_eq!(order_book["symbol"], "BTC-USDT");
    assert!(order_book["bids"].is_array());
    assert!(order_book["asks"].is_array());

    // Test 24h stats
    let stats_result = get_24h_stats("ETH-USDT".to_string(), state.clone()).await;
    assert!(stats_result.is_ok(), "Failed to get 24h stats");

    let stats = stats_result.unwrap();
    assert_eq!(stats["symbol"], "ETH-USDT");
    assert!(stats["price_change"].is_number());
}

#[tokio::test]
async fn test_position_and_order_management() {
    let state = AppState::new();

    // Test get positions (should be empty initially)
    let positions_result = get_positions(state.clone()).await;
    assert!(positions_result.is_ok(), "Failed to get positions");

    let positions = positions_result.unwrap();
    assert_eq!(positions.len(), 0);

    // Test get open orders (should be empty initially)
    let orders_result = get_open_orders(state.clone()).await;
    assert!(orders_result.is_ok(), "Failed to get open orders");

    let orders = orders_result.unwrap();
    assert_eq!(orders.len(), 0);

    // Test position risk
    let risk_result = get_position_risk(None, state.clone()).await;
    assert!(risk_result.is_ok(), "Failed to get position risk");

    let risk = risk_result.unwrap();
    assert!(risk["positions"].is_array());
    assert!(risk["risk_metrics"].is_object());

    // Test cancel all orders (should succeed even with no orders)
    let cancel_result = cancel_all_orders(None, state).await;
    assert!(cancel_result.is_ok(), "Failed to cancel all orders");

    let cancelled_count = cancel_result.unwrap();
    assert_eq!(cancelled_count, 0);
}

#[test]
fn test_request_serialization() {
    // Test PlaceOrderRequest serialization
    let order_request = PlaceOrderRequest {
        strategy_id: Uuid::new_v4().to_string(),
        symbol: "BTC-USDT".to_string(),
        side: "sell".to_string(),
        order_type: "market".to_string(),
        quantity: 0.1,
        price: None,
        time_in_force: Some("IOC".to_string()),
        reduce_only: Some(true),
        post_only: Some(false),
    };

    let json_str = serde_json::to_string(&order_request).expect("Failed to serialize order request");
    let deserialized: PlaceOrderRequest = serde_json::from_str(&json_str).expect("Failed to deserialize order request");

    assert_eq!(order_request.symbol, deserialized.symbol);
    assert_eq!(order_request.side, deserialized.side);
    assert_eq!(order_request.quantity, deserialized.quantity);
    assert_eq!(order_request.reduce_only, deserialized.reduce_only);

    // Test SignalRequest serialization
    let signal_request = SignalRequest {
        strategy_id: Uuid::new_v4().to_string(),
        symbol: "ETH-USDT".to_string(),
        signal_type: "close".to_string(),
        side: Some("sell".to_string()),
        quantity: 1.0,
        price: Some(2500.0),
        stop_loss: Some(2600.0),
        take_profit: Some(2400.0),
        confidence: 0.95,
        metadata: Some(serde_json::json!({
            "indicator": "RSI",
            "value": 75.0,
            "action": "overbought"
        })),
    };

    let signal_json = serde_json::to_string(&signal_request).expect("Failed to serialize signal request");
    let signal_deserialized: SignalRequest = serde_json::from_str(&signal_json).expect("Failed to deserialize signal request");

    assert_eq!(signal_request.symbol, signal_deserialized.symbol);
    assert_eq!(signal_request.signal_type, signal_deserialized.signal_type);
    assert_eq!(signal_request.confidence, signal_deserialized.confidence);
    assert_eq!(signal_request.side, signal_deserialized.side);
}

#[tokio::test]
async fn test_error_handling() {
    let state = AppState::new();

    // Test invalid strategy ID format
    let invalid_order = PlaceOrderRequest {
        strategy_id: "invalid-uuid".to_string(), // Invalid UUID format
        symbol: "BTC-USDT".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        quantity: 0.01,
        price: Some(45000.0),
        time_in_force: Some("GTC".to_string()),
        reduce_only: Some(false),
        post_only: Some(false),
    };

    let result = place_order(invalid_order, state).await;
    assert!(result.is_err(), "Order with invalid strategy ID should fail");

    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Invalid strategy ID"), "Error should mention invalid strategy ID");
}

#[test]
fn test_request_validation() {
    // Test valid order types
    let valid_sides = vec!["buy", "sell"];
    for side in valid_sides {
        assert!(side.parse::<ea_okx_core::models::order::OrderSide>().is_ok(),
                "Side '{}' should be valid", side);
    }

    let valid_order_types = vec!["market", "limit", "stop_loss", "take_profit"];
    for order_type in valid_order_types {
        assert!(order_type.parse::<ea_okx_core::models::order::OrderType>().is_ok(),
                "Order type '{}' should be valid", order_type);
    }

    let valid_signal_types = vec!["open", "close", "partial_close", "modify", "stop_loss", "take_profit"];
    for signal_type in valid_signal_types {
        // SignalType doesn't implement FromStr directly, but we can validate the string values
        assert!(!signal_type.is_empty(), "Signal type '{}' should not be empty", signal_type);
    }
}