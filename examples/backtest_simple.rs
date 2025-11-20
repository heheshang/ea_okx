use chrono::Utc;
use ea_okx_backtest::{BacktestConfig, BacktestEngine, CostModel, PositionSizing, MockDataSource};
use ea_okx_core::{Symbol, Candle, Price, Quantity};
use ea_okx_strategy::traits::{Strategy, StrategyConfig, MarketDataEvent};
use ea_okx_strategy::signal::{Signal, SignalType};
use ea_okx_strategy::metrics::PerformanceMetrics;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use async_trait::async_trait;

/// Simple moving average crossover strategy for backtesting
struct MACrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
    fast_ma: Vec<Decimal>,
    slow_ma: Vec<Decimal>,
    current_signal: SignalType,
}

impl MACrossoverStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            fast_ma: Vec::new(),
            slow_ma: Vec::new(),
            current_signal: SignalType::Hold,
        }
    }

    fn calculate_sma(prices: &[Decimal], period: usize) -> Option<Decimal> {
        if prices.len() < period {
            return None;
        }

        let sum: Decimal = prices.iter().rev().take(period).sum();
        Some(sum / Decimal::from(period))
    }
}

#[async_trait]
impl Strategy for MACrossoverStrategy {
    async fn initialize(&mut self, _config: StrategyConfig) -> ea_okx_strategy::error::Result<()> {
        println!("Initializing MA Crossover Strategy");
        println!("Fast period: {}", self.fast_period);
        println!("Slow period: {}", self.slow_period);
        Ok(())
    }

    async fn on_market_data(&mut self, event: MarketDataEvent) -> ea_okx_strategy::error::Result<()> {
        if let MarketDataEvent::Candle(candle) = event {
            let close = candle.close.as_decimal();
            
            // Add to price history (keep only what we need)
            let max_period = self.slow_period.max(self.fast_period);
            
            // Calculate MAs
            let mut prices = vec![close];
            if let Some(fast_ma) = Self::calculate_sma(&prices, self.fast_period) {
                self.fast_ma.push(fast_ma);
            }
            
            if let Some(slow_ma) = Self::calculate_sma(&prices, self.slow_period) {
                self.slow_ma.push(slow_ma);
            }
            
            // Limit history size
            if self.fast_ma.len() > max_period {
                self.fast_ma.remove(0);
            }
            if self.slow_ma.len() > max_period {
                self.slow_ma.remove(0);
            }
            
            // Generate signal on crossover
            if self.fast_ma.len() >= 2 && self.slow_ma.len() >= 2 {
                let prev_fast = self.fast_ma[self.fast_ma.len() - 2];
                let curr_fast = self.fast_ma[self.fast_ma.len() - 1];
                let prev_slow = self.slow_ma[self.slow_ma.len() - 2];
                let curr_slow = self.slow_ma[self.slow_ma.len() - 1];
                
                // Bullish crossover
                if prev_fast <= prev_slow && curr_fast > curr_slow {
                    self.current_signal = SignalType::Long;
                }
                // Bearish crossover
                else if prev_fast >= prev_slow && curr_fast < curr_slow {
                    self.current_signal = SignalType::Close;
                }
                else {
                    self.current_signal = SignalType::Hold;
                }
            }
        }

        Ok(())
    }

    fn generate_signal(&self) -> ea_okx_strategy::error::Result<Signal> {
        Ok(Signal {
            signal_type: self.current_signal.clone(),
            symbol: Symbol::new("BTC-USDT").unwrap(),
            strength: dec!(1.0),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    async fn on_order_fill(&mut self, order: &ea_okx_core::Order) -> ea_okx_strategy::error::Result<()> {
        println!(
            "Order filled: {:?} {} @ {}",
            order.side,
            order.symbol.as_str(),
            order.price.as_decimal()
        );
        Ok(())
    }

    async fn on_order_reject(
        &mut self,
        order: &ea_okx_core::Order,
        reason: &str,
    ) -> ea_okx_strategy::error::Result<()> {
        println!("Order rejected: {} - {}", order.id, reason);
        Ok(())
    }

    fn get_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics::default()
    }

    fn serialize_state(&self) -> ea_okx_strategy::error::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "fast_period": self.fast_period,
            "slow_period": self.slow_period,
        }))
    }

    fn deserialize_state(&mut self, _state: serde_json::Value) -> ea_okx_strategy::error::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> ea_okx_strategy::error::Result<()> {
        println!("Shutting down MA Crossover Strategy");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== EA OKX Backtest Example ===\n");

    // Configure backtest
    let config = BacktestConfig {
        initial_capital: dec!(100000.0),
        start_time: Utc::now() - chrono::Duration::days(90),
        end_time: Utc::now(),
        symbols: vec![Symbol::new("BTC-USDT").unwrap()],
        interval: "1H".to_string(),
        cost_model: CostModel::okx_spot_conservative(),
        verbose: true,
        max_positions: 1,
        position_sizing: PositionSizing::PercentOfEquity(dec!(0.95)), // 95% of capital
    };

    println!("Backtest Configuration:");
    println!("  Initial Capital: ${}", config.initial_capital);
    println!("  Period: {} to {}", config.start_time.format("%Y-%m-%d"), config.end_time.format("%Y-%m-%d"));
    println!("  Symbol: BTC-USDT");
    println!("  Interval: {}", config.interval);
    println!("  Max Positions: {}\n", config.max_positions);

    // Create strategy
    let strategy = Box::new(MACrossoverStrategy::new(10, 30));

    // Create mock data source with sample data
    let mut data_source = MockDataSource::new();
    
    // Generate sample candles for demonstration
    let mut sample_candles = Vec::new();
    let start_price = dec!(50000.0);
    let start_time = config.start_time;
    
    println!("Generating sample data for demonstration...\n");
    
    for i in 0..100 {
        let timestamp = start_time + chrono::Duration::hours(i as i64);
        // Simple sine wave pattern for demo
        let variation = ((i as f64 * 0.1).sin() * 2000.0) as i64;
        let close = start_price + Decimal::from(variation);
        
        let candle = Candle {
            symbol: Symbol::new("BTC-USDT").unwrap(),
            timestamp,
            interval: "1H".to_string(),
            open: Price::new(close - dec!(100.0)).unwrap(),
            high: Price::new(close + dec!(200.0)).unwrap(),
            low: Price::new(close - dec!(200.0)).unwrap(),
            close: Price::new(close).unwrap(),
            volume: Quantity::new(dec!(10.0)).unwrap(),
            quote_volume: dec!(500000.0),
            trade_count: 1000,
            vwap: close,
        };
        
        sample_candles.push(candle);
    }
    
    data_source.add_candles(Symbol::new("BTC-USDT").unwrap(), sample_candles);

    // Create and run backtest engine
    let mut engine = BacktestEngine::new(config, strategy, Box::new(data_source)).await?;

    println!("Running backtest...\n");
    let result = engine.run().await?;

    // Print results
    println!("{}", result.summary());

    // Additional analysis
    println!("\n=== Additional Analysis ===");
    println!("Risk-Adjusted Returns:");
    println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
    println!("  Sortino Ratio: {:.2}", result.sortino_ratio);
    println!("  Calmar Ratio: {:.2}", result.calmar_ratio);

    println!("\nCost Analysis:");
    println!("  Total Costs: ${:.2}", result.total_costs);
    println!("  As % of Capital: {:.2}%", 
        (result.total_costs / result.initial_capital) * dec!(100.0));
    
    if result.total_trades > 0 {
        println!("  Cost per Trade: ${:.2}", 
            result.total_costs / Decimal::from(result.total_trades));
    }

    // Save results to JSON
    let json_output = serde_json::to_string_pretty(&result)?;
    std::fs::write("backtest_results.json", json_output)?;
    println!("\nResults saved to backtest_results.json");

    Ok(())
}
