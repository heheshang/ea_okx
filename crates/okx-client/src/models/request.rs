//! Request models for OKX API

use serde::Serialize;

/// Order placement request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    /// Instrument ID (e.g., "BTC-USDT")
    pub inst_id: String,
    
    /// Trade mode: cash, cross, isolated
    pub td_mode: String,
    
    /// Order side: buy, sell
    pub side: String,
    
    /// Order type: market, limit, post_only, fok, ioc
    pub ord_type: String,
    
    /// Order size
    pub sz: String,
    
    /// Order price (for limit orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub px: Option<String>,
    
    /// Client order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
}

/// Cancel order request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderRequest {
    /// Instrument ID
    pub inst_id: String,
    
    /// Order ID (either this or clOrdId required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ord_id: Option<String>,
    
    /// Client order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
}
