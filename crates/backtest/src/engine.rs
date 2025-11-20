use crate::cost_model::CostModel;
use crate::error::{Error, Result};
use crate::events::{ExecutionEvent, Fill, MarketEvent, Trade};
use crate::portfolio::Portfolio;
use crate::results::BacktestResult;
use chrono::{DateTime, Utc};
use ea_okx_core::{Symbol, Price, Quantity};
use ea_okx_core::models::{Order, OrderSide, OrderType, PositionSide};

// Candle structure for backtesting (duplicated from ea_okx_data to avoid sqlx dependency)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}
// use ea_okx_data::storage::TimescaleStorage;  // Disabled due to sqlx compile-time requirements
use ea_okx_strategy::traits::{Strategy, StrategyConfig, RiskLimits};
use ea_okx_strategy::signal::{Signal, SignalType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{HashMap, VecDeque};
use tracing::{debug, info, warn};
use uuid::Uuid;
use async_trait::async_trait;

/// Mock storage interface for backtesting (until TimescaleDB is configured)
#[async_trait]
pub trait HistoricalDataSource: Send + Sync {
    async fn query_candles(
        &self,
        symbol: &Symbol,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>>;
}

/// In-memory mock storage for testing
pub struct MockDataSource {
    candles: HashMap<String, Vec<Candle>>,
}

impl MockDataSource {
    pub fn new() -> Self {
        Self {
            candles: HashMap::new(),
        }
    }

    pub fn add_candles(&mut self, symbol: Symbol, candles: Vec<Candle>) {
        self.candles.insert(symbol.as_str().to_string(), candles);
    }
}

#[async_trait]
impl HistoricalDataSource for MockDataSource {
    async fn query_candles(
        &self,
        symbol: &Symbol,
        _interval: &str,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<Vec<Candle>> {
        Ok(self.candles
            .get(symbol.as_str())
            .cloned()
            .unwrap_or_default())
    }
}

/// Configuration for backtest execution
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Initial capital in quote currency
    pub initial_capital: Decimal,
    
    /// Start time for backtest
    pub start_time: DateTime<Utc>,
    
    /// End time for backtest
    pub end_time: DateTime<Utc>,
    
    /// Symbols to trade
    pub symbols: Vec<Symbol>,
    
    /// Candle interval for data
    pub interval: String,
    
    /// Cost model for realistic execution
    pub cost_model: CostModel,
    
    /// Enable detailed logging
    pub verbose: bool,
    
    /// Maximum number of open positions
    pub max_positions: usize,
    
    /// Position sizing mode
    pub position_sizing: PositionSizing,
}

#[derive(Debug, Clone)]
pub enum PositionSizing {
    /// Fixed amount per trade
    Fixed(Decimal),
    
    /// Percentage of equity per trade
    PercentOfEquity(Decimal),
    
    /// Kelly criterion based sizing
    Kelly { win_rate: Decimal, win_loss_ratio: Decimal },
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: dec!(100000.0),
            start_time: Utc::now() - chrono::Duration::days(365),
            end_time: Utc::now(),
            symbols: vec![Symbol::new("BTC-USDT").unwrap()],
            interval: "1H".to_string(),
            cost_model: CostModel::default(),
            verbose: false,
            max_positions: 5,
            position_sizing: PositionSizing::PercentOfEquity(dec!(0.1)),
        }
    }
}

/// Main backtesting engine
pub struct BacktestEngine {
    config: BacktestConfig,
    strategy: Box<dyn Strategy>,
    portfolio: Portfolio,
    storage: Box<dyn HistoricalDataSource>,
    
    /// Event queue sorted by timestamp
    events: VecDeque<MarketEvent>,
    
    /// Pending orders
    pending_orders: HashMap<Uuid, Order>,
    
    /// Execution history
    executions: Vec<ExecutionEvent>,
    
    /// Trade history
    trades: Vec<Trade>,
    
    /// Current market prices
    current_prices: HashMap<Symbol, Decimal>,
    
    /// Average volumes for slippage calculation
    avg_volumes: HashMap<Symbol, Decimal>,
}

impl BacktestEngine {
    pub async fn new(
        config: BacktestConfig,
        strategy: Box<dyn Strategy>,
        storage: Box<dyn HistoricalDataSource>,
    ) -> Result<Self> {
        let portfolio = Portfolio::new(config.initial_capital);
        
        Ok(Self {
            config,
            strategy,
            portfolio,
            storage,
            events: VecDeque::new(),
            pending_orders: HashMap::new(),
            executions: Vec::new(),
            trades: Vec::new(),
            current_prices: HashMap::new(),
            avg_volumes: HashMap::new(),
        })
    }

    /// Load historical market data
    async fn load_data(&mut self) -> Result<()> {
        info!(
            "Loading market data from {} to {}",
            self.config.start_time, self.config.end_time
        );

        for symbol in &self.config.symbols {
            let candles = self.storage.query_candles(
                symbol,
                &self.config.interval,
                self.config.start_time,
                self.config.end_time,
            ).await?;

            info!("Loaded {} candles for {}", candles.len(), symbol.as_str());

            if candles.is_empty() {
                return Err(Error::InsufficientData(format!(
                    "No data found for {} in the specified time range",
                    symbol.as_str()
                )));
            }

            // Convert candles to market events
            for candle in candles {
                self.events.push_back(MarketEvent::Candle(candle));
            }
        }

        // Sort events by timestamp
        let mut events_vec: Vec<_> = self.events.drain(..).collect();
        events_vec.sort_by_key(|e| e.timestamp());
        self.events.extend(events_vec);

        info!("Total events loaded: {}", self.events.len());
        Ok(())
    }

    /// Run the backtest
    pub async fn run(&mut self) -> Result<BacktestResult> {
        info!("Starting backtest...");
        
        // Load historical data
        self.load_data().await?;
        
        // Initialize strategy
        let strategy_config = StrategyConfig {
            strategy_id: Uuid::new_v4(),
            name: "Backtest Strategy".to_string(),
            version: "1.0.0".to_string(),
            symbols: self.config.symbols.iter().map(|s| s.as_str().to_string()).collect(),
            parameters: HashMap::new(),
            risk_limits: RiskLimits {
                max_position_size: dec!(10000.0),
                max_leverage: dec!(3.0),
                stop_loss_pct: dec!(0.02),
                take_profit_pct: Some(dec!(0.05)),
            },
        };
        
        self.strategy.initialize(strategy_config).await?;
        
        let mut event_count = 0;
        let total_events = self.events.len();
        
        // Process events chronologically
        while let Some(event) = self.events.pop_front() {
            event_count += 1;
            
            if self.config.verbose && event_count % 1000 == 0 {
                info!("Processing event {}/{}", event_count, total_events);
            }
            
            self.process_event(event).await?;
        }
        
        // Close all open positions at end
        self.close_all_positions().await?;
        
        // Finalize and generate results
        let result = self.generate_results().await?;
        
        info!("Backtest completed. Final equity: {}", result.final_equity);
        info!("Total trades: {}", result.total_trades);
        info!("Win rate: {:.2}%", result.win_rate * dec!(100.0));
        
        Ok(result)
    }

    /// Process a single market event
    async fn process_event(&mut self, event: MarketEvent) -> Result<()> {
        let timestamp = event.timestamp();
        
        // Update current market state
        match &event {
            MarketEvent::Candle(candle) => {
                self.current_prices.insert(candle.symbol.clone(), candle.close);
                self.avg_volumes.insert(candle.symbol.clone(), candle.volume);
            }
            MarketEvent::Trade { symbol, price, .. } => {
                self.current_prices.insert(symbol.clone(), *price);
            }
            MarketEvent::OrderBook { symbol, bids, asks, .. } => {
                if let Some((best_bid, _)) = bids.first() {
                    if let Some((best_ask, _)) = asks.first() {
                        let mid_price = (*best_bid + *best_ask) / dec!(2.0);
                        self.current_prices.insert(symbol.clone(), mid_price);
                    }
                }
            }
        }
        
        // Update portfolio with current prices
        self.portfolio.update_prices(&self.current_prices);
        
        // Check pending orders for fills
        self.check_pending_orders(timestamp).await?;
        
        // Feed event to strategy
        let market_data = match event {
            MarketEvent::Candle(candle) => {
                ea_okx_strategy::traits::MarketDataEvent::Candle {
                    symbol: candle.symbol.clone(),
                    open: candle.open,
                    high: candle.high,
                    low: candle.low,
                    close: candle.close,
                    volume: candle.volume,
                    timestamp: candle.timestamp,
                }
            }
            _ => return Ok(()), // Only process candles for now
        };
        
        self.strategy.on_market_data(market_data).await?;
        
        // Check if strategy generated a signal - strategies now don't have symbols in signals
        // We'll process the first symbol in config for now
        if !self.config.symbols.is_empty() {
            let symbol = self.config.symbols[0].clone();  // Clone to avoid borrow issues
            match self.strategy.generate_signal().await {
                Ok(signal) => {
                    if signal.signal_type != SignalType::Hold {
                        self.execute_signal(signal, &symbol, timestamp).await?;
                    }
                }
                Err(e) => {
                    warn!("Strategy signal generation failed: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Check pending orders for execution
    async fn check_pending_orders(&mut self, timestamp: DateTime<Utc>) -> Result<()> {
        let mut to_fill = Vec::new();
        
        for (order_id, order) in &self.pending_orders {
            if let Some(current_price) = self.current_prices.get(&order.symbol) {
                // Simple fill logic based on order type
                let should_fill = match order.order_type {
                    OrderType::Market => true,
                    OrderType::Limit => {
                        if let Some(order_price) = order.price {
                            match order.side {
                                OrderSide::Buy => current_price <= &order_price.as_decimal(),
                                OrderSide::Sell => current_price >= &order_price.as_decimal(),
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                
                if should_fill {
                    to_fill.push(*order_id);
                }
            }
        }
        
        // Execute fills
        for order_id in to_fill {
            if let Some(order) = self.pending_orders.remove(&order_id) {
                self.fill_order(order, timestamp).await?;
            }
        }
        
        Ok(())
    }

    /// Execute a signal from the strategy
    async fn execute_signal(&mut self, signal: Signal, symbol: &Symbol, timestamp: DateTime<Utc>) -> Result<()> {
        debug!("Executing signal: {:?}", signal);
        
        // Check position limits
        if self.portfolio.position_count() >= self.config.max_positions {
            debug!("Max positions reached, skipping signal");
            return Ok(());
        }
        
        // Calculate position size
        let size = self.calculate_position_size(symbol)?;
        
        if size <= Decimal::ZERO {
            debug!("Position size is zero, skipping signal");
            return Ok(());
        }
        
        // Create order
        let side = match signal.signal_type {
            SignalType::Buy => OrderSide::Buy,
            SignalType::Sell => OrderSide::Sell,
            SignalType::Hold => return Ok(()),
            SignalType::CloseLong | SignalType::CloseShort => {
                // Close existing position
                self.close_position(symbol, timestamp).await?;
                return Ok(());
            }
        };
        
        let price = self.current_prices.get(symbol)
            .copied()
            .ok_or_else(|| Error::ExecutionError("No price available".to_string()))?;
        
        let order = Order::new(
            Uuid::new_v4(),
            symbol.clone(),
            side,
            OrderType::Market,
            Quantity::new(size)?,
            Some(Price::new(price)?),
        );
        
        // Add to pending orders
        self.pending_orders.insert(order.id, order);
        
        Ok(())
    }

    /// Fill an order
    async fn fill_order(&mut self, order: Order, timestamp: DateTime<Utc>) -> Result<()> {
        let symbol = &order.symbol;
        let avg_volume = self.avg_volumes.get(symbol).copied().unwrap_or(dec!(1.0));
        
        // Calculate execution price with costs
        let (execution_price, commission, slippage) = self.config.cost_model.calculate_total_cost(
            order.order_type,
            order.side,
            order.price.unwrap_or(Price::new(dec!(0.0)).unwrap()).as_decimal(),
            order.quantity.as_decimal(),
            avg_volume,
        );
        
        // Create fill
        let fill = Fill {
            order_id: order.id,
            price: execution_price,
            quantity: order.quantity.as_decimal(),
            commission,
            timestamp,
            slippage,
        };
        
        // Update portfolio
        self.portfolio.apply_fill(&order, &fill)?;
        
        // Record execution
        let execution = ExecutionEvent::OrderFilled {
            order_id: order.id,
            symbol: symbol.clone(),
            side: order.side,
            filled_price: execution_price,
            filled_quantity: order.quantity.as_decimal(),
            commission,
            timestamp,
        };
        
        self.executions.push(execution);
        
        // Notify strategy
        self.strategy.on_order_fill(&order).await?;
        
        info!(
            "Order filled: {:?} {} @ {} (comm: {}, slip: {})",
            order.side, symbol.as_str(), execution_price, commission, slippage
        );
        
        Ok(())
    }

    /// Calculate position size based on configuration
    fn calculate_position_size(&self, symbol: &Symbol) -> Result<Decimal> {
        let price = self.current_prices.get(symbol)
            .copied()
            .ok_or_else(|| Error::ExecutionError("No price available".to_string()))?;
        
        let equity = self.portfolio.total_equity();
        
        let size = match &self.config.position_sizing {
            PositionSizing::Fixed(amount) => amount / price,
            PositionSizing::PercentOfEquity(pct) => (equity * pct) / price,
            PositionSizing::Kelly { win_rate, win_loss_ratio } => {
                let kelly = (win_rate * (win_loss_ratio + dec!(1.0)) - dec!(1.0)) / win_loss_ratio;
                let kelly_fraction = kelly.max(Decimal::ZERO).min(dec!(0.25)); // Cap at 25%
                (equity * kelly_fraction) / price
            }
        };
        
        Ok(size)
    }

    /// Close a specific position
    async fn close_position(&mut self, symbol: &Symbol, timestamp: DateTime<Utc>) -> Result<()> {
        if let Some(position) = self.portfolio.get_position(symbol) {
            let quantity = position.quantity.as_decimal().abs();
            let price = self.current_prices.get(symbol)
                .copied()
                .ok_or_else(|| Error::ExecutionError("No price available".to_string()))?;
            
            let side = match position.side {
                PositionSide::Long => OrderSide::Sell,
                PositionSide::Short => OrderSide::Buy,
                PositionSide::Net => OrderSide::Sell,
            };
            
            let order = Order::new(
                Uuid::new_v4(),
                symbol.clone(),
                side,
                OrderType::Market,
                Quantity::new(quantity)?,
                Some(Price::new(price)?),
            );
            
            self.fill_order(order, timestamp).await?;
        }
        
        Ok(())
    }

    /// Close all open positions
    async fn close_all_positions(&mut self) -> Result<()> {
        let symbols: Vec<_> = self.portfolio.positions.keys().cloned().collect();
        let timestamp = self.config.end_time;
        
        for symbol in symbols {
            self.close_position(&symbol, timestamp).await?;
        }
        
        Ok(())
    }

    /// Generate backtest results
    async fn generate_results(&self) -> Result<BacktestResult> {
        BacktestResult::from_portfolio_and_trades(
            &self.portfolio,
            &self.trades,
            self.config.initial_capital,
            self.config.start_time,
            self.config.end_time,
        )
    }
}
