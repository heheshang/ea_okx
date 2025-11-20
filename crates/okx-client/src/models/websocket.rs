//! WebSocket message models for OKX API
//!
//! This module contains all data structures for WebSocket communication,
//! including subscription requests, channel types, and event messages.

use crate::error::{Error, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// WebSocket channel types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Channel {
    /// Ticker channel - real-time ticker updates
    Tickers,
    /// Candle channel - OHLCV data
    #[serde(rename = "candle1m")]
    Candle1m,
    #[serde(rename = "candle5m")]
    Candle5m,
    #[serde(rename = "candle15m")]
    Candle15m,
    #[serde(rename = "candle1H")]
    Candle1h,
    #[serde(rename = "candle4H")]
    Candle4h,
    #[serde(rename = "candle1D")]
    Candle1d,
    /// Order book channels
    Books5,    // Top 5 levels
    Books50,   // Top 50 levels
    BooksL2Tbt, // Level 2 tick-by-tick
    /// Recent trades
    Trades,
    /// Account channel (private)
    Account,
    /// Position channel (private)
    Positions,
    /// Order channel (private)
    Orders,
    /// Balance and position channel (private)
    BalanceAndPosition,
}

impl Channel {
    /// Check if channel is public (doesn't require authentication)
    pub fn is_public(&self) -> bool {
        !matches!(
            self,
            Channel::Account
                | Channel::Positions
                | Channel::Orders
                | Channel::BalanceAndPosition
        )
    }
    
    /// Get channel name as string
    pub fn as_str(&self) -> &str {
        match self {
            Channel::Tickers => "tickers",
            Channel::Candle1m => "candle1m",
            Channel::Candle5m => "candle5m",
            Channel::Candle15m => "candle15m",
            Channel::Candle1h => "candle1H",
            Channel::Candle4h => "candle4H",
            Channel::Candle1d => "candle1D",
            Channel::Books5 => "books5",
            Channel::Books50 => "books50",
            Channel::BooksL2Tbt => "books-l2-tbt",
            Channel::Trades => "trades",
            Channel::Account => "account",
            Channel::Positions => "positions",
            Channel::Orders => "orders",
            Channel::BalanceAndPosition => "balance_and_position",
        }
    }
}

/// Subscription request
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionRequest {
    pub channel: Channel,
    pub instrument_id: Option<String>,
}

impl SubscriptionRequest {
    /// Create a new subscription request
    pub fn new(channel: Channel, instrument_id: impl Into<String>) -> Self {
        Self {
            channel,
            instrument_id: Some(instrument_id.into()),
        }
    }
    
    /// Create a subscription request without instrument ID (for account channels)
    pub fn new_account(channel: Channel) -> Self {
        Self {
            channel,
            instrument_id: None,
        }
    }
    
    /// Convert to JSON for WebSocket message
    pub fn to_json(&self) -> Value {
        if let Some(inst_id) = &self.instrument_id {
            serde_json::json!({
                "channel": self.channel.as_str(),
                "instId": inst_id
            })
        } else {
            serde_json::json!({
                "channel": self.channel.as_str()
            })
        }
    }
}

/// Subscription response from OKX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub event: String,
    pub arg: Value,
    pub code: Option<String>,
    pub msg: Option<String>,
}

/// WebSocket event types
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    /// Subscription confirmation
    Subscribe(SubscriptionResponse),
    /// Unsubscription confirmation
    Unsubscribe(SubscriptionResponse),
    /// Error event
    Error { code: String, msg: String },
    /// Login/authentication response
    Login { code: String, msg: String },
    /// Market data events
    Ticker(TickerData),
    Candle(CandleData),
    OrderBook(OrderBookData),
    Trade(TradeData),
    /// Account events
    Account(AccountData),
    Position(PositionData),
    Order(OrderData),
}

impl WebSocketEvent {
    /// Parse WebSocket event from JSON
    pub fn from_json(value: &Value) -> Result<Self> {
        // Check if it's a response event (subscribe, unsubscribe, error, login)
        if let Some(event) = value.get("event").and_then(|v| v.as_str()) {
            match event {
                "subscribe" => {
                    let response: SubscriptionResponse = serde_json::from_value(value.clone())
                        .map_err(|e| Error::ParseError(format!("Invalid subscribe response: {}", e)))?;
                    return Ok(WebSocketEvent::Subscribe(response));
                }
                "unsubscribe" => {
                    let response: SubscriptionResponse = serde_json::from_value(value.clone())
                        .map_err(|e| Error::ParseError(format!("Invalid unsubscribe response: {}", e)))?;
                    return Ok(WebSocketEvent::Unsubscribe(response));
                }
                "error" => {
                    let code = value.get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let msg = value.get("msg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();
                    return Ok(WebSocketEvent::Error { code, msg });
                }
                "login" => {
                    let code = value.get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let msg = value.get("msg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    return Ok(WebSocketEvent::Login { code, msg });
                }
                _ => {
                    return Err(Error::ParseError(format!("Unknown event type: {}", event)));
                }
            }
        }
        
        // Check if it's a data push (has "arg" and "data" fields)
        if let Some(arg) = value.get("arg") {
            let channel = arg.get("channel")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::ParseError("Missing channel field".to_string()))?;
            
            let data = value.get("data")
                .ok_or_else(|| Error::ParseError("Missing data field".to_string()))?;
            
            return Self::parse_data_event(channel, data);
        }
        
        Err(Error::ParseError("Invalid WebSocket message format".to_string()))
    }
    
    /// Parse data event based on channel type
    fn parse_data_event(channel: &str, data: &Value) -> Result<Self> {
        match channel {
            "tickers" => {
                let ticker: TickerData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid ticker data: {}", e)))?;
                Ok(WebSocketEvent::Ticker(ticker))
            }
            ch if ch.starts_with("candle") => {
                let candle: CandleData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid candle data: {}", e)))?;
                Ok(WebSocketEvent::Candle(candle))
            }
            "books5" | "books50" | "books-l2-tbt" => {
                let book: OrderBookData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid order book data: {}", e)))?;
                Ok(WebSocketEvent::OrderBook(book))
            }
            "trades" => {
                let trade: TradeData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid trade data: {}", e)))?;
                Ok(WebSocketEvent::Trade(trade))
            }
            "account" => {
                let account: AccountData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid account data: {}", e)))?;
                Ok(WebSocketEvent::Account(account))
            }
            "positions" => {
                let position: PositionData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid position data: {}", e)))?;
                Ok(WebSocketEvent::Position(position))
            }
            "orders" => {
                let order: OrderData = serde_json::from_value(data.clone())
                    .map_err(|e| Error::ParseError(format!("Invalid order data: {}", e)))?;
                Ok(WebSocketEvent::Order(order))
            }
            _ => Err(Error::ParseError(format!("Unknown channel: {}", channel))),
        }
    }
}

/// Ticker data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerData {
    pub inst_type: String,
    pub inst_id: String,
    pub last: String,
    pub last_sz: String,
    pub ask_px: String,
    pub ask_sz: String,
    pub bid_px: String,
    pub bid_sz: String,
    pub open_24h: String,
    pub high_24h: String,
    pub low_24h: String,
    pub vol_ccy_24h: String,
    pub vol_24h: String,
    pub ts: String,
    pub sod_utc0: Option<String>,
    pub sod_utc8: Option<String>,
}

/// Candle/OHLCV data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleData {
    /// Timestamp (milliseconds)
    #[serde(rename = "ts")]
    pub timestamp: String,
    /// Open price
    #[serde(rename = "o")]
    pub open: String,
    /// High price
    #[serde(rename = "h")]
    pub high: String,
    /// Low price
    #[serde(rename = "l")]
    pub low: String,
    /// Close price
    #[serde(rename = "c")]
    pub close: String,
    /// Volume in base currency
    #[serde(rename = "vol")]
    pub volume: String,
    /// Volume in quote currency
    #[serde(rename = "volCcy")]
    pub volume_currency: String,
    /// Volume in USD
    #[serde(rename = "volCcyQuote")]
    pub volume_usd: Option<String>,
    /// Confirm: 0 = candle not closed, 1 = candle closed
    #[serde(rename = "confirm")]
    pub confirm: String,
}

impl CandleData {
    /// Parse into typed values
    pub fn parse(&self) -> Result<ParsedCandle> {
        Ok(ParsedCandle {
            timestamp: self.timestamp.parse()
                .map_err(|e| Error::ParseError(format!("Invalid timestamp: {}", e)))?,
            open: self.open.parse()
                .map_err(|e| Error::ParseError(format!("Invalid open price: {}", e)))?,
            high: self.high.parse()
                .map_err(|e| Error::ParseError(format!("Invalid high price: {}", e)))?,
            low: self.low.parse()
                .map_err(|e| Error::ParseError(format!("Invalid low price: {}", e)))?,
            close: self.close.parse()
                .map_err(|e| Error::ParseError(format!("Invalid close price: {}", e)))?,
            volume: self.volume.parse()
                .map_err(|e| Error::ParseError(format!("Invalid volume: {}", e)))?,
            is_confirmed: self.confirm == "1",
        })
    }
}

/// Parsed candle data with typed values
#[derive(Debug, Clone)]
pub struct ParsedCandle {
    pub timestamp: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub is_confirmed: bool,
}

/// Order book data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookData {
    pub asks: Vec<BookLevel>,
    pub bids: Vec<BookLevel>,
    pub ts: String,
    pub checksum: Option<i32>,
    pub prev_seq_id: Option<i64>,
    pub seq_id: Option<i64>,
}

/// Order book level [price, quantity, deprecated, num_orders]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel(pub String, pub String, pub String, pub String);

impl BookLevel {
    /// Get price
    pub fn price(&self) -> Result<Decimal> {
        self.0.parse()
            .map_err(|e| Error::ParseError(format!("Invalid price: {}", e)))
    }
    
    /// Get quantity
    pub fn quantity(&self) -> Result<Decimal> {
        self.1.parse()
            .map_err(|e| Error::ParseError(format!("Invalid quantity: {}", e)))
    }
    
    /// Get number of orders
    pub fn num_orders(&self) -> Result<u32> {
        self.3.parse()
            .map_err(|e| Error::ParseError(format!("Invalid num_orders: {}", e)))
    }
}

/// Trade data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeData {
    pub inst_id: String,
    pub trade_id: String,
    pub px: String,
    pub sz: String,
    pub side: String,
    pub ts: String,
    pub count: Option<String>,
}

/// Account data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub u_time: String,
    pub total_eq: String,
    pub iso_eq: Option<String>,
    pub adj_eq: Option<String>,
    pub ord_froz: Option<String>,
    pub imr: Option<String>,
    pub mmr: Option<String>,
    pub notional_usd: Option<String>,
    pub mgn_ratio: Option<String>,
    pub details: Vec<AccountDetail>,
}

/// Account detail (per currency)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetail {
    pub ccy: String,
    pub eq: String,
    pub cash_bal: String,
    pub u_time: String,
    pub iso_eq: Option<String>,
    pub avail_eq: Option<String>,
    pub dis_eq: Option<String>,
    pub avail_bal: Option<String>,
    pub frozen_bal: Option<String>,
    pub ord_frozen: Option<String>,
    pub liab: Option<String>,
    pub upl: Option<String>,
    pub upl_liab: Option<String>,
    pub cross_liab: Option<String>,
    pub iso_liab: Option<String>,
    pub mgn_ratio: Option<String>,
    pub interest: Option<String>,
    pub twap: Option<String>,
    pub max_loan: Option<String>,
    pub eq_usd: Option<String>,
    pub notional_lever: Option<String>,
}

/// Position data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    pub inst_type: String,
    pub inst_id: String,
    pub mgn_mode: String,
    pub pos_id: String,
    pub pos_side: String,
    pub pos: String,
    pub base_bal: Option<String>,
    pub quote_bal: Option<String>,
    pub pos_ccy: Option<String>,
    pub avail_pos: String,
    pub avg_px: String,
    pub upl: String,
    pub upl_ratio: String,
    pub upl_last_px: Option<String>,
    pub upl_ratio_last_px: Option<String>,
    pub inst_type_field: Option<String>,
    pub mgn_ratio: Option<String>,
    pub notional_usd: String,
    pub adl: Option<String>,
    pub liq_px: Option<String>,
    pub mark_px: Option<String>,
    pub imr: Option<String>,
    pub margin: Option<String>,
    pub mgn_rate: Option<String>,
    pub liab: Option<String>,
    pub liab_ccy: Option<String>,
    pub interest: Option<String>,
    pub trade_id: Option<String>,
    pub opt_val: Option<String>,
    pub pending_close_ord_liab_val: Option<String>,
    pub u_time: String,
    pub c_time: String,
}

/// Order data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    pub inst_type: String,
    pub inst_id: String,
    pub ccy: Option<String>,
    pub ord_id: String,
    pub cl_ord_id: String,
    pub tag: Option<String>,
    pub px: String,
    pub sz: String,
    pub pnl: Option<String>,
    pub ord_type: String,
    pub side: String,
    pub pos_side: Option<String>,
    pub td_mode: String,
    pub fill_px: String,
    pub fill_sz: String,
    pub acc_fill_sz: String,
    pub fill_notional_usd: Option<String>,
    pub fill_time: Option<String>,
    pub avg_px: String,
    pub state: String,
    pub lever: Option<String>,
    pub attach_algo_cl_ord_id: Option<String>,
    pub tp_trigger_px: Option<String>,
    pub tp_trigger_px_type: Option<String>,
    pub tp_ord_px: Option<String>,
    pub sl_trigger_px: Option<String>,
    pub sl_trigger_px_type: Option<String>,
    pub sl_ord_px: Option<String>,
    pub fee_ccy: Option<String>,
    pub fee: Option<String>,
    pub rebate_ccy: Option<String>,
    pub rebate: Option<String>,
    pub category: Option<String>,
    pub u_time: String,
    pub c_time: String,
    pub req_id: Option<String>,
    pub amend_result: Option<String>,
    pub code: Option<String>,
    pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_channel_is_public() {
        assert!(Channel::Tickers.is_public());
        assert!(Channel::Candle1m.is_public());
        assert!(Channel::Books5.is_public());
        assert!(Channel::Trades.is_public());
        
        assert!(!Channel::Account.is_public());
        assert!(!Channel::Positions.is_public());
        assert!(!Channel::Orders.is_public());
    }
    
    #[test]
    fn test_subscription_request_to_json() {
        let req = SubscriptionRequest::new(Channel::Tickers, "BTC-USDT");
        let json = req.to_json();
        
        assert_eq!(json["channel"], "tickers");
        assert_eq!(json["instId"], "BTC-USDT");
    }
    
    #[test]
    fn test_subscription_request_account() {
        let req = SubscriptionRequest::new_account(Channel::Account);
        let json = req.to_json();
        
        assert_eq!(json["channel"], "account");
        assert!(json.get("instId").is_none());
    }
    
    #[test]
    fn test_parse_candle_data() {
        let candle = CandleData {
            timestamp: "1234567890000".to_string(),
            open: "50000.00".to_string(),
            high: "51000.00".to_string(),
            low: "49000.00".to_string(),
            close: "50500.00".to_string(),
            volume: "100.5".to_string(),
            volume_currency: "5050000.00".to_string(),
            volume_usd: Some("5050000.00".to_string()),
            confirm: "1".to_string(),
        };
        
        let parsed = candle.parse().unwrap();
        assert_eq!(parsed.timestamp, 1234567890000);
        assert_eq!(parsed.open, Decimal::new(5000000, 2));
        assert_eq!(parsed.is_confirmed, true);
    }
    
    #[test]
    fn test_book_level_parsing() {
        let level = BookLevel(
            "50000.50".to_string(),
            "1.5".to_string(),
            "0".to_string(),
            "10".to_string(),
        );
        
        assert_eq!(level.price().unwrap(), Decimal::new(5000050, 2));
        assert_eq!(level.quantity().unwrap(), Decimal::new(15, 1));
        assert_eq!(level.num_orders().unwrap(), 10);
    }
    
    #[test]
    fn test_websocket_event_parse_error() {
        let json = serde_json::json!({
            "event": "error",
            "code": "60012",
            "msg": "Invalid request"
        });
        
        let event = WebSocketEvent::from_json(&json).unwrap();
        match event {
            WebSocketEvent::Error { code, msg } => {
                assert_eq!(code, "60012");
                assert_eq!(msg, "Invalid request");
            }
            _ => panic!("Expected Error event"),
        }
    }
    
    #[test]
    fn test_websocket_event_login() {
        let json = serde_json::json!({
            "event": "login",
            "code": "0",
            "msg": "Login successful"
        });
        
        let event = WebSocketEvent::from_json(&json).unwrap();
        match event {
            WebSocketEvent::Login { code, msg } => {
                assert_eq!(code, "0");
                assert_eq!(msg, "Login successful");
            }
            _ => panic!("Expected Login event"),
        }
    }
}
