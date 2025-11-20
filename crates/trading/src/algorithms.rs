use crate::error::Result;
use crate::order_manager::OrderManager;
use chrono::{DateTime, Duration, Timelike, Utc};
use ea_okx_core::models::{Order, OrderSide, OrderType};
use ea_okx_core::{Symbol, Price, Quantity};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// TWAP (Time-Weighted Average Price) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwapConfig {
    /// Total quantity to execute
    pub total_quantity: Quantity,
    
    /// Duration in minutes
    pub duration_minutes: u32,
    
    /// Interval between slices in seconds
    pub slice_interval_seconds: u32,
    
    /// Randomization percentage (0-25)
    pub randomization_pct: Decimal,
    
    /// Order type for child orders
    pub order_type: OrderType,
    
    /// Price offset in basis points
    pub price_offset_bps: i32,
    
    /// Use market order for final slice
    pub aggressive_on_final: bool,
}

impl Default for TwapConfig {
    fn default() -> Self {
        Self {
            total_quantity: Quantity::new(dec!(1.0)).unwrap(),
            duration_minutes: 30,
            slice_interval_seconds: 120,
            randomization_pct: dec!(10.0),
            order_type: OrderType::Limit,
            price_offset_bps: 0,
            aggressive_on_final: true,
        }
    }
}

/// VWAP (Volume-Weighted Average Price) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VwapConfig {
    /// Total quantity to execute
    pub total_quantity: Quantity,
    
    /// Start time
    pub start_time: DateTime<Utc>,
    
    /// End time
    pub end_time: DateTime<Utc>,
    
    /// Historical volume profile (hour -> volume percentage)
    pub volume_profile: Vec<(u32, Decimal)>,
    
    /// Minimum slice size
    pub min_slice_size: Quantity,
    
    /// Price offset in basis points
    pub price_offset_bps: i32,
}

impl Default for VwapConfig {
    fn default() -> Self {
        // Default volume profile based on typical crypto market patterns
        let volume_profile = vec![
            (0, dec!(2.0)), (1, dec!(1.5)), (2, dec!(1.0)), (3, dec!(1.0)),
            (4, dec!(1.5)), (5, dec!(2.0)), (6, dec!(3.0)), (7, dec!(4.0)),
            (8, dec!(5.0)), (9, dec!(6.0)), (10, dec!(7.0)), (11, dec!(6.0)),
            (12, dec!(7.0)), (13, dec!(8.0)), (14, dec!(9.0)), (15, dec!(8.0)),
            (16, dec!(7.0)), (17, dec!(6.0)), (18, dec!(5.0)), (19, dec!(4.0)),
            (20, dec!(3.5)), (21, dec!(3.0)), (22, dec!(2.5)), (23, dec!(2.0)),
        ];
        
        Self {
            total_quantity: Quantity::new(dec!(1.0)).unwrap(),
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::hours(4),
            volume_profile,
            min_slice_size: Quantity::new(dec!(0.001)).unwrap(),
            price_offset_bps: 0,
        }
    }
}

/// TWAP execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwapResult {
    pub total_executed: Quantity,
    pub average_price: Price,
    pub slices_executed: u32,
    pub slices_failed: u32,
    pub total_duration: Duration,
    pub slice_details: Vec<SliceExecution>,
}

/// VWAP execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VwapResult {
    pub total_executed: Quantity,
    pub average_price: Price,
    pub slices_executed: u32,
    pub total_duration: Duration,
    pub vwap_deviation_bps: Decimal,
}

/// Individual slice execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceExecution {
    pub slice_number: u32,
    pub target_quantity: Quantity,
    pub executed_quantity: Quantity,
    pub price: Price,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

/// TWAP executor
pub struct TwapExecutor {
    config: TwapConfig,
    symbol: Symbol,
    side: OrderSide,
    order_manager: Arc<OrderManager>,
}

impl TwapExecutor {
    pub fn new(
        config: TwapConfig,
        symbol: Symbol,
        side: OrderSide,
        order_manager: Arc<OrderManager>,
    ) -> Self {
        Self {
            config,
            symbol,
            side,
            order_manager,
        }
    }

    /// Execute TWAP algorithm
    pub async fn execute(&self, current_price: Price) -> Result<TwapResult> {
        info!(
            "Starting TWAP execution: {} {} @ {} over {} minutes",
            self.config.total_quantity.as_decimal(),
            self.symbol.as_str(),
            current_price.as_decimal(),
            self.config.duration_minutes
        );

        let start_time = Utc::now();
        
        // Calculate number of slices
        let total_seconds = self.config.duration_minutes as u64 * 60;
        let slice_count = (total_seconds / self.config.slice_interval_seconds as u64).max(1);
        let base_slice_size = self.config.total_quantity.as_decimal() / Decimal::from(slice_count);
        
        debug!("TWAP: {} slices of ~{} each", slice_count, base_slice_size);

        let mut remaining = self.config.total_quantity.as_decimal();
        let mut total_cost = Decimal::ZERO;
        let mut total_executed = Decimal::ZERO;
        let mut slice_details = Vec::new();
        let mut slices_executed = 0u32;
        let mut slices_failed = 0u32;

        for slice_num in 0..slice_count {
            if remaining <= Decimal::ZERO {
                break;
            }

            // Apply randomization
            let random_factor = if self.config.randomization_pct > Decimal::ZERO {
                let random_val = (rand::random::<f64>() - 0.5) * 2.0; // -1 to 1
                dec!(1.0) + (Decimal::from_f64_retain(random_val).unwrap_or(Decimal::ZERO) 
                    * self.config.randomization_pct / dec!(100.0))
            } else {
                dec!(1.0)
            };

            let slice_size = (base_slice_size * random_factor).min(remaining);
            
            // Determine if this is the final slice
            let is_final = slice_num == slice_count - 1 || slice_size >= remaining;
            
            // Choose order type
            let order_type = if is_final && self.config.aggressive_on_final {
                OrderType::Market
            } else {
                self.config.order_type
            };

            // Calculate price with offset
            let slice_price = self.calculate_price_with_offset(current_price);

            // Execute slice
            match self.execute_slice(slice_size, slice_price, order_type).await {
                Ok(executed_qty) => {
                    let executed_dec = executed_qty.as_decimal();
                    total_executed += executed_dec;
                    total_cost += executed_dec * slice_price.as_decimal();
                    remaining -= executed_dec;
                    slices_executed += 1;

                    slice_details.push(SliceExecution {
                        slice_number: slice_num as u32,
                        target_quantity: Quantity::new(slice_size).unwrap(),
                        executed_quantity: executed_qty,
                        price: slice_price,
                        timestamp: Utc::now(),
                        success: true,
                    });

                    debug!("TWAP slice {}/{} executed: {} @ {}", 
                        slice_num + 1, slice_count, executed_dec, slice_price.as_decimal());
                }
                Err(e) => {
                    warn!("TWAP slice {}/{} failed: {}", slice_num + 1, slice_count, e);
                    slices_failed += 1;

                    slice_details.push(SliceExecution {
                        slice_number: slice_num as u32,
                        target_quantity: Quantity::new(slice_size).unwrap(),
                        executed_quantity: Quantity::new(Decimal::ZERO).unwrap(),
                        price: slice_price,
                        timestamp: Utc::now(),
                        success: false,
                    });
                }
            }

            // Wait for next slice (unless it's the last one)
            if !is_final {
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    self.config.slice_interval_seconds as u64
                )).await;
            }
        }

        let avg_price = if total_executed > Decimal::ZERO {
            Price::new(total_cost / total_executed)?
        } else {
            current_price
        };

        let result = TwapResult {
            total_executed: Quantity::new(total_executed)?,
            average_price: avg_price,
            slices_executed,
            slices_failed,
            total_duration: Utc::now() - start_time,
            slice_details,
        };

        info!(
            "TWAP completed: executed {} @ avg {} ({}/{} slices)",
            total_executed, avg_price.as_decimal(), slices_executed, slice_count
        );

        Ok(result)
    }

    /// Execute a single slice
    async fn execute_slice(
        &self,
        quantity: Decimal,
        price: Price,
        order_type: OrderType,
    ) -> Result<Quantity> {
        let order = Order::new(
            Uuid::new_v4(),
            self.symbol.clone(),
            self.side,
            order_type,
            Quantity::new(quantity)?,
            Some(price),
        );

        // Submit order
        let order_id = self.order_manager.submit_order(order).await?;

        // Wait for fill (simplified - in production would monitor events)
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Return executed quantity (simplified)
        Ok(Quantity::new(quantity)?)
    }

    /// Calculate price with offset
    fn calculate_price_with_offset(&self, base_price: Price) -> Price {
        let offset_decimal = Decimal::from(self.config.price_offset_bps) / dec!(10000.0);
        let offset_amount = base_price.as_decimal() * offset_decimal;
        
        let adjusted_price = match self.side {
            OrderSide::Buy => base_price.as_decimal() + offset_amount,
            OrderSide::Sell => base_price.as_decimal() - offset_amount,
        };

        Price::new(adjusted_price).unwrap_or(base_price)
    }
}

/// VWAP executor
pub struct VwapExecutor {
    config: VwapConfig,
    symbol: Symbol,
    side: OrderSide,
    order_manager: Arc<OrderManager>,
}

impl VwapExecutor {
    pub fn new(
        config: VwapConfig,
        symbol: Symbol,
        side: OrderSide,
        order_manager: Arc<OrderManager>,
    ) -> Self {
        Self {
            config,
            symbol,
            side,
            order_manager,
        }
    }

    /// Execute VWAP algorithm
    pub async fn execute(&self, current_price: Price) -> Result<VwapResult> {
        info!(
            "Starting VWAP execution: {} {} from {} to {}",
            self.config.total_quantity.as_decimal(),
            self.symbol.as_str(),
            self.config.start_time,
            self.config.end_time
        );

        let start_time = Utc::now();
        let duration = self.config.end_time - self.config.start_time;
        let duration_hours = duration.num_hours() as u32;

        // Calculate quantity per hour based on volume profile
        let total_volume_weight: Decimal = self.config.volume_profile.iter()
            .map(|(_, weight)| weight)
            .sum();

        let mut remaining = self.config.total_quantity.as_decimal();
        let mut total_cost = Decimal::ZERO;
        let mut total_executed = Decimal::ZERO;
        let mut slices_executed = 0u32;

        for hour in 0..duration_hours {
            if remaining <= Decimal::ZERO {
                break;
            }

            // Get volume weight for this hour
            let hour_of_day = (self.config.start_time + Duration::hours(hour as i64))
                .hour();
            
            let volume_weight = self.config.volume_profile
                .iter()
                .find(|(h, _)| *h == hour_of_day)
                .map(|(_, w)| *w)
                .unwrap_or(dec!(4.0)); // Default weight

            // Calculate slice size based on volume profile
            let slice_ratio = volume_weight / total_volume_weight;
            let slice_size = (self.config.total_quantity.as_decimal() * slice_ratio)
                .max(self.config.min_slice_size.as_decimal())
                .min(remaining);

            // Execute slice
            let slice_price = self.calculate_price_with_offset(current_price);
            
            match self.execute_slice(slice_size, slice_price).await {
                Ok(executed_qty) => {
                    let executed_dec = executed_qty.as_decimal();
                    total_executed += executed_dec;
                    total_cost += executed_dec * slice_price.as_decimal();
                    remaining -= executed_dec;
                    slices_executed += 1;

                    debug!("VWAP hour {} executed: {} @ {}", 
                        hour, executed_dec, slice_price.as_decimal());
                }
                Err(e) => {
                    warn!("VWAP hour {} failed: {}", hour, e);
                }
            }

            // Wait for next hour
            if hour < duration_hours - 1 {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        }

        let avg_price = if total_executed > Decimal::ZERO {
            Price::new(total_cost / total_executed)?
        } else {
            current_price
        };

        // Calculate VWAP deviation
        let vwap_deviation_bps = ((avg_price.as_decimal() - current_price.as_decimal()) 
            / current_price.as_decimal()) * dec!(10000.0);

        let result = VwapResult {
            total_executed: Quantity::new(total_executed)?,
            average_price: avg_price,
            slices_executed,
            total_duration: Utc::now() - start_time,
            vwap_deviation_bps,
        };

        info!(
            "VWAP completed: executed {} @ avg {} (deviation: {} bps)",
            total_executed, avg_price.as_decimal(), vwap_deviation_bps
        );

        Ok(result)
    }

    /// Execute a single slice
    async fn execute_slice(&self, quantity: Decimal, price: Price) -> Result<Quantity> {
        let order = Order::new(
            Uuid::new_v4(),
            self.symbol.clone(),
            self.side,
            OrderType::Limit,
            Quantity::new(quantity)?,
            Some(price),
        );

        let order_id = self.order_manager.submit_order(order).await?;
        
        // Wait for fill (simplified)
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        Ok(Quantity::new(quantity)?)
    }

    /// Calculate price with offset
    fn calculate_price_with_offset(&self, base_price: Price) -> Price {
        let offset_decimal = Decimal::from(self.config.price_offset_bps) / dec!(10000.0);
        let offset_amount = base_price.as_decimal() * offset_decimal;
        
        let adjusted_price = match self.side {
            OrderSide::Buy => base_price.as_decimal() + offset_amount,
            OrderSide::Sell => base_price.as_decimal() - offset_amount,
        };

        Price::new(adjusted_price).unwrap_or(base_price)
    }
}

// Use a simple random implementation since we don't have rand crate
mod rand {
    use std::cell::Cell;
    
    thread_local! {
        static SEED: Cell<u64> = Cell::new(0x123456789abcdef);
    }
    
    pub fn random<T: From<f64>>() -> T {
        SEED.with(|seed| {
            let s = seed.get();
            let new_seed = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            seed.set(new_seed);
            T::from((new_seed >> 32) as f64 / u32::MAX as f64)
        })
    }
}
