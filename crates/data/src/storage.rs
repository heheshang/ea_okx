//! Data storage layer
//!
//! This module provides interfaces for storing market data
//! in TimescaleDB and Redis.

use crate::error::Result;
use chrono::{DateTime, Utc};
use ea_okx_core::types::{Price, Quantity, Symbol};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database row for OHLCV data
#[derive(Debug, FromRow)]
struct CandleRow {
    symbol: String,
    timestamp: DateTime<Utc>,
    interval: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    quote_volume: Decimal,
    trade_count: i32,
    vwap: Option<Decimal>,
}

/// OHLCV candle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub timestamp: DateTime<Utc>,
    pub interval: String,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
    pub quote_volume: Decimal,
    pub trade_count: i32,
    pub vwap: Option<Decimal>,
}

/// Trade tick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: Symbol,
    pub timestamp: DateTime<Utc>,
    pub trade_id: String,
    pub price: Price,
    pub quantity: Quantity,
    pub side: String, // "buy" or "sell"
    pub is_block_trade: bool,
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: Symbol,
    pub timestamp: DateTime<Utc>,
    pub bids: Vec<(Price, Quantity)>,
    pub asks: Vec<(Price, Quantity)>,
    pub checksum: Option<i32>,
    pub depth_level: String,
}

/// Storage interface for TimescaleDB
pub struct TimescaleStorage {
    pool: sqlx::PgPool,
}

impl TimescaleStorage {
    /// Create a new TimescaleDB storage instance
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(connection_string)
            .await
            .map_err(|e| crate::error::Error::Internal(format!("Failed to connect to database: {}", e)))?;
        
        Ok(Self { pool })
    }
    
    /// Store candle data
    pub async fn store_candle(&self, candle: &Candle) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO market_ohlcv (
                symbol, timestamp, interval, open, high, low, close, 
                volume, quote_volume, trade_count, vwap
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (symbol, interval, timestamp) DO UPDATE
            SET open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                volume = EXCLUDED.volume,
                quote_volume = EXCLUDED.quote_volume,
                trade_count = EXCLUDED.trade_count,
                vwap = EXCLUDED.vwap
            "#,
        )
        .bind(candle.symbol.as_str())
        .bind(candle.timestamp)
        .bind(&candle.interval)
        .bind(candle.open.as_decimal())
        .bind(candle.high.as_decimal())
        .bind(candle.low.as_decimal())
        .bind(candle.close.as_decimal())
        .bind(candle.volume.as_decimal())
        .bind(candle.quote_volume)
        .bind(candle.trade_count)
        .bind(candle.vwap)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Store tick data
    pub async fn store_tick(&self, tick: &Tick) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO market_ticks (
                symbol, timestamp, trade_id, price, quantity, side, is_block_trade
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (trade_id) DO NOTHING
            "#,
        )
        .bind(tick.symbol.as_str())
        .bind(tick.timestamp)
        .bind(&tick.trade_id)
        .bind(tick.price.as_decimal())
        .bind(tick.quantity.as_decimal())
        .bind(&tick.side)
        .bind(tick.is_block_trade)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Store order book snapshot
    pub async fn store_orderbook(&self, snapshot: &OrderBookSnapshot) -> Result<()> {
        // Convert bids and asks to JSONB
        let bids_json = serde_json::to_value(
            snapshot.bids.iter()
                .map(|(p, q)| vec![p.as_decimal(), q.as_decimal()])
                .collect::<Vec<_>>()
        )?;
        
        let asks_json = serde_json::to_value(
            snapshot.asks.iter()
                .map(|(p, q)| vec![p.as_decimal(), q.as_decimal()])
                .collect::<Vec<_>>()
        )?;
        
        sqlx::query(
            r#"
            INSERT INTO order_book_snapshots (
                symbol, timestamp, bids, asks, checksum, depth_level
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(snapshot.symbol.as_str())
        .bind(snapshot.timestamp)
        .bind(bids_json)
        .bind(asks_json)
        .bind(snapshot.checksum)
        .bind(&snapshot.depth_level)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Query candles within time range
    pub async fn query_candles(
        &self,
        symbol: &Symbol,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>> {
        let rows: Vec<CandleRow> = sqlx::query_as(
            r#"
            SELECT symbol, timestamp, interval, open, high, low, close,
                   volume, quote_volume, trade_count, vwap
            FROM market_ohlcv
            WHERE symbol = $1 AND interval = $2
              AND timestamp >= $3 AND timestamp < $4
            ORDER BY timestamp ASC
            "#,
        )
        .bind(symbol.as_str())
        .bind(interval)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;
        
        let candles = rows.into_iter().map(|row| Candle {
            symbol: Symbol::new(&row.symbol).unwrap(),
            timestamp: row.timestamp,
            interval: row.interval,
            open: Price::new(row.open).unwrap(),
            high: Price::new(row.high).unwrap(),
            low: Price::new(row.low).unwrap(),
            close: Price::new(row.close).unwrap(),
            volume: Quantity::new(row.volume).unwrap(),
            quote_volume: row.quote_volume,
            trade_count: row.trade_count,
            vwap: row.vwap,
        }).collect();
        
        Ok(candles)
    }
    
    /// Get latest candle
    pub async fn get_latest_candle(
        &self,
        symbol: &Symbol,
        interval: &str,
    ) -> Result<Option<Candle>> {
        let row: Option<CandleRow> = sqlx::query_as(
            r#"
            SELECT symbol, timestamp, interval, open, high, low, close,
                   volume, quote_volume, trade_count, vwap
            FROM market_ohlcv
            WHERE symbol = $1 AND interval = $2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(symbol.as_str())
        .bind(interval)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|row| Candle {
            symbol: Symbol::new(&row.symbol).unwrap(),
            timestamp: row.timestamp,
            interval: row.interval,
            open: Price::new(row.open).unwrap(),
            high: Price::new(row.high).unwrap(),
            low: Price::new(row.low).unwrap(),
            close: Price::new(row.close).unwrap(),
            volume: Quantity::new(row.volume).unwrap(),
            quote_volume: row.quote_volume,
            trade_count: row.trade_count,
            vwap: row.vwap,
        }))
    }
}

/// Storage interface for Redis cache
pub struct RedisStorage {
    client: redis::Client,
}

impl RedisStorage {
    /// Create a new Redis storage instance
    pub fn new(connection_string: &str) -> Result<Self> {
        let client = redis::Client::open(connection_string)?;
        Ok(Self { client })
    }
    
    /// Cache latest candle
    pub async fn cache_latest_candle(&self, candle: &Candle) -> Result<()> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("candle:{}:{}", candle.symbol.as_str(), candle.interval);
        let value = serde_json::to_string(candle)?;
        
        redis::cmd("SET")
            .arg(&key)
            .arg(&value)
            .arg("EX")
            .arg(3600) // 1 hour expiry
            .query_async::<_, ()>(&mut con)
            .await?;
        
        Ok(())
    }
    
    /// Get latest candle from cache
    pub async fn get_latest_candle(&self, symbol: &Symbol, interval: &str) -> Result<Option<Candle>> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("candle:{}:{}", symbol.as_str(), interval);
        
        let value: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut con)
            .await?;
        
        if let Some(v) = value {
            let candle: Candle = serde_json::from_str(&v)?;
            Ok(Some(candle))
        } else {
            Ok(None)
        }
    }
    
    /// Cache latest price
    pub async fn cache_price(&self, symbol: &Symbol, price: &Price) -> Result<()> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("price:{}", symbol.as_str());
        
        redis::cmd("SET")
            .arg(&key)
            .arg(price.as_decimal().to_string())
            .arg("EX")
            .arg(60) // 1 minute expiry
            .query_async::<_, ()>(&mut con)
            .await?;
        
        Ok(())
    }
    
    /// Get latest price from cache
    pub async fn get_price(&self, symbol: &Symbol) -> Result<Option<Price>> {
        let mut con = self.client.get_async_connection().await?;
        let key = format!("price:{}", symbol.as_str());
        
        let value: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut con)
            .await?;
        
        if let Some(v) = value {
            let decimal = v.parse::<Decimal>()
                .map_err(|e| crate::error::Error::Internal(format!("Failed to parse price: {}", e)))?;
            Ok(Some(Price::new(decimal)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ea_okx_core::types::Symbol;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_candle_creation() {
        let symbol = Symbol::new("BTC-USDT").unwrap();
        let candle = Candle {
            symbol: symbol.clone(),
            timestamp: Utc::now(),
            interval: "1m".to_string(),
            open: Price::new(dec!(50000)).unwrap(),
            high: Price::new(dec!(50100)).unwrap(),
            low: Price::new(dec!(49900)).unwrap(),
            close: Price::new(dec!(50050)).unwrap(),
            volume: Quantity::new(dec!(10.5)).unwrap(),
            quote_volume: dec!(525000),
            trade_count: 150,
            vwap: Some(dec!(50000)),
        };
        
        assert_eq!(candle.symbol, symbol);
        assert_eq!(candle.interval, "1m");
    }
    
    #[test]
    fn test_tick_creation() {
        let symbol = Symbol::new("ETH-USDT").unwrap();
        let tick = Tick {
            symbol: symbol.clone(),
            timestamp: Utc::now(),
            trade_id: "12345".to_string(),
            price: Price::new(dec!(3000)).unwrap(),
            quantity: Quantity::new(dec!(1.5)).unwrap(),
            side: "buy".to_string(),
            is_block_trade: false,
        };
        
        assert_eq!(tick.symbol, symbol);
        assert_eq!(tick.side, "buy");
    }
}
