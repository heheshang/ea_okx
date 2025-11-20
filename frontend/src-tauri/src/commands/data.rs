use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Subscribe to market data
#[tauri::command]
pub async fn subscribe_market_data(symbols: Vec<String>) -> Result<(), String> {
    log::info!("Subscribing to market data: {:?}", symbols);
    // TODO: Integrate with WebSocket service
    Ok(())
}

/// Get latest price
#[tauri::command]
pub async fn get_latest_price(symbol: String) -> Result<f64, String> {
    log::info!("Fetching latest price for: {}", symbol);
    // TODO: Integrate with data service
    Ok(45000.0) // Mock price
}

/// Get candles
#[tauri::command]
pub async fn get_candles(symbol: String, interval: String, limit: Option<usize>) -> Result<Vec<Candle>, String> {
    log::info!("Fetching candles for: {} (interval: {}, limit: {:?})", symbol, interval, limit);
    // TODO: Integrate with data service
    Ok(vec![])
}
