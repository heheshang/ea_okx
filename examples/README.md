# Example Strategies and Demos

This directory contains example trading strategies and demonstration programs for the OKX Quantitative Trading System.

## Real-Time Market Data Examples

### WebSocket Ticker Example (`websocket_ticker.rs`)

Demonstrates real-time market data streaming using OKX WebSocket API.

**Features:**
- Connects to OKX WebSocket (public and private channels)
- Subscribes to multiple ticker streams (BTC, ETH, SOL)
- Subscribes to 1-minute candlestick data
- Authentication for private channels
- Auto-reconnection with exponential backoff
- Heartbeat/ping-pong mechanism

**Usage:**
```bash
# Set environment variables
export OKX_API_KEY="your-api-key"
export OKX_SECRET_KEY="your-secret-key" 
export OKX_PASSPHRASE="your-passphrase"
export OKX_TESTNET="true"  # Optional, default false

# Run the example (from examples directory)
cd examples
cargo run --bin websocket_ticker
```

**Output Example:**
```
INFO  OKX WebSocket Ticker Example
INFO  Connecting to OKX WebSocket...
INFO  Successfully connected to OKX WebSocket
INFO  Subscribing to tickers: ["BTC-USDT", "ETH-USDT", "SOL-USDT"]
INFO  ✓ Subscription confirmed
INFO  📊 Ticker BTC-USDT - Last: 50123.45, Bid: 50120.10, Ask: 50125.80
INFO  🕯️ Candle - O: 50100, H: 50200, L: 50050, C: 50123, V: 123.45
```

## Available Strategies

### 1. Simple Moving Average Crossover (`simple_ma_crossover.rs`)

A trend-following strategy using two moving averages.

**Parameters:**
- Fast MA: 20 periods (default)
- Slow MA: 50 periods (default)
- Stop Loss: 2%
- Take Profit: 5%
- Position Size: 20% of capital

**Signals:**
- **BUY**: Fast MA crosses above Slow MA (Golden Cross)
- **SELL**: Fast MA crosses below Slow MA (Death Cross)

**Example:**
```rust
use simple_ma_crossover::{MACrossoverStrategy, MACrossoverParams};

let params = MACrossoverParams {
    fast_period: 20,
    slow_period: 50,
    stop_loss_pct: dec!(0.02),
    take_profit_pct: dec!(0.05),
    position_size_pct: dec!(0.20),
};

let mut strategy = MACrossoverStrategy::new(
    strategy_id,
    Symbol::new("BTC-USDT")?,
    params,
    dec!(10000), // $10,000 capital
);
```

### 2. Grid Trading (`grid_trading.rs`)

Range-bound strategy profiting from price oscillations.

**Parameters:**
- Price Range: Upper and lower bounds
- Grid Levels: Number of price intervals
- Order Size: Fixed quantity per level

**How it Works:**
1. Creates grid of buy/sell orders across price range
2. Buys at lower levels, sells at higher levels
3. Profits from volatility within the range

**Example:**
```rust
use grid_trading::GridTradingStrategy;

let strategy = GridTradingStrategy::new(
    strategy_id,
    Symbol::new("BTC-USDT")?,
    Price::new(dec!(38000))?, // Lower bound
    Price::new(dec!(42000))?, // Upper bound
    10,                        // 10 grid levels
    Quantity::new(dec!(0.01))?, // 0.01 BTC per order
);
```

### 3. RSI Mean Reversion (`rsi_strategy.rs`)

Contrarian strategy using Relative Strength Index.

**Parameters:**
- RSI Period: 14 (default)
- Oversold: RSI < 30
- Overbought: RSI > 70
- Position Size: 10% of capital

**Signals:**
- **BUY**: RSI < 30 (oversold condition)
- **SELL**: RSI > 70 (overbought condition)

**Example:**
```rust
use rsi_strategy::RSIStrategy;

let mut strategy = RSIStrategy::new(
    strategy_id,
    Symbol::new("BTC-USDT")?,
    14,            // RSI period
    dec!(10000),   // Capital
);

// Update with price
strategy.on_price(current_price);

// Get signal
if let Some(signal) = strategy.generate_signal() {
    // Execute trade
}
```

## Running Examples

```bash
# Run MA Crossover strategy
cargo run --example simple_ma_crossover

# Run Grid Trading strategy
cargo run --example grid_trading

# Run RSI strategy
cargo run --example rsi_strategy
```

## Testing Strategies

All strategies include comprehensive unit tests:

```bash
# Test all examples
cargo test --examples

# Test specific strategy
cargo test --example simple_ma_crossover
```

## Strategy Development Guidelines

### 1. Signal Generation
- Clearly define entry and exit conditions
- Use statistical/technical indicators
- Avoid look-ahead bias

### 2. Risk Management
- Always implement stop losses
- Define maximum position sizes
- Limit leverage appropriately

### 3. Parameter Optimization
- Use walk-forward analysis
- Avoid overfitting to historical data
- Test across multiple market conditions

### 4. Performance Metrics
- Track Sharpe ratio
- Monitor maximum drawdown
- Calculate win rate and profit factor

## Backtesting

These strategies are designed to work with the backtesting engine:

```rust
use ea_okx_backtest::BacktestEngine;

let engine = BacktestEngine::new(strategy, historical_data);
let results = engine.run()?;

println!("Sharpe Ratio: {}", results.sharpe_ratio);
println!("Max Drawdown: {}%", results.max_drawdown * 100);
println!("Win Rate: {}%", results.win_rate * 100);
```

## Paper Trading

Test strategies in real-time without risking capital:

```bash
# Enable paper trading mode
cargo run --features paper-trading
```

## Production Deployment

⚠️ **WARNING**: Test thoroughly in paper trading mode before deploying to production!

1. Backtest with at least 5 years of data
2. Run 30 days of paper trading
3. Start with minimal capital
4. Monitor performance closely
5. Have emergency stop mechanisms ready

## Contributing

When adding new strategies:

1. Include comprehensive documentation
2. Add unit tests (target 90%+ coverage)
3. Provide example usage
4. Document parameters and signals
5. Include risk management

## Disclaimer

These strategies are for educational purposes only. Past performance does not guarantee future results. Trading cryptocurrencies involves substantial risk of loss.
