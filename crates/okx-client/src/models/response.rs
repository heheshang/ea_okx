//! Response models for OKX API

use serde::Deserialize;

/// Generic API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    /// Response code ("0" for success)
    pub code: String,
    
    /// Response message
    pub msg: String,
    
    /// Response data
    #[serde(default)]
    pub data: Vec<T>,
}

impl<T> ApiResponse<T> {
    /// Checks if the response is successful
    pub fn is_success(&self) -> bool {
        self.code == "0"
    }
}

/// Order response data from REST API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    /// Order ID
    pub ord_id: String,
    
    /// Client order ID
    pub cl_ord_id: String,
    
    /// Order state
    pub state: String,
}
