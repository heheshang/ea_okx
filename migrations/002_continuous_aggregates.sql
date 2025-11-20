-- Continuous aggregates and additional optimizations

-- Continuous aggregate for 5-minute OHLCV from ticks
CREATE MATERIALIZED VIEW market_ohlcv_5m
WITH (timescaledb.continuous) AS
SELECT
    symbol,
    time_bucket('5 minutes', timestamp) AS timestamp,
    first(price, timestamp) AS open,
    max(price) AS high,
    min(price) AS low,
    last(price, timestamp) AS close,
    sum(quantity) AS volume,
    sum(price * quantity) AS quote_volume,
    count(*) AS trade_count,
    sum(price * quantity) / NULLIF(sum(quantity), 0) AS vwap
FROM market_ticks
GROUP BY symbol, time_bucket('5 minutes', timestamp);

-- Refresh policy for 5-minute aggregate
SELECT add_continuous_aggregate_policy('market_ohlcv_5m',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes');

-- Continuous aggregate for hourly OHLCV
CREATE MATERIALIZED VIEW market_ohlcv_1h
WITH (timescaledb.continuous) AS
SELECT
    symbol,
    time_bucket('1 hour', timestamp) AS timestamp,
    first(open, timestamp) AS open,
    max(high) AS high,
    min(low) AS low,
    last(close, timestamp) AS close,
    sum(volume) AS volume,
    sum(quote_volume) AS quote_volume,
    sum(trade_count) AS trade_count,
    sum(quote_volume) / NULLIF(sum(volume), 0) AS vwap
FROM market_ohlcv_5m
GROUP BY symbol, time_bucket('1 hour', timestamp);

SELECT add_continuous_aggregate_policy('market_ohlcv_1h',
    start_offset => INTERVAL '1 day',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');

-- Continuous aggregate for daily OHLCV
CREATE MATERIALIZED VIEW market_ohlcv_1d
WITH (timescaledb.continuous) AS
SELECT
    symbol,
    time_bucket('1 day', timestamp) AS timestamp,
    first(open, timestamp) AS open,
    max(high) AS high,
    min(low) AS low,
    last(close, timestamp) AS close,
    sum(volume) AS volume,
    sum(quote_volume) AS quote_volume,
    sum(trade_count) AS trade_count,
    sum(quote_volume) / NULLIF(sum(volume), 0) AS vwap
FROM market_ohlcv_1h
GROUP BY symbol, time_bucket('1 day', timestamp);

SELECT add_continuous_aggregate_policy('market_ohlcv_1d',
    start_offset => INTERVAL '7 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day');

-- Continuous aggregate for daily strategy performance
CREATE MATERIALIZED VIEW strategy_performance_daily
WITH (timescaledb.continuous) AS
SELECT
    strategy_id,
    time_bucket('1 day', created_at) AS date,
    count(*) FILTER (WHERE status = 'filled') AS total_trades,
    count(*) FILTER (WHERE status = 'filled' AND realized_pnl > 0) AS winning_trades,
    count(*) FILTER (WHERE status = 'filled' AND realized_pnl < 0) AS losing_trades,
    sum(COALESCE(realized_pnl, 0)) AS total_pnl,
    sum(commission) AS total_commission,
    sum(COALESCE(realized_pnl, 0)) - sum(commission) AS net_pnl,
    sum(COALESCE(realized_pnl, 0)) * 100.0 / NULLIF(sum(avg_fill_price * filled_quantity), 0) AS return_pct,
    sum(quantity * avg_fill_price) AS total_volume
FROM trades
WHERE status = 'filled'
GROUP BY strategy_id, time_bucket('1 day', created_at);

SELECT add_continuous_aggregate_policy('strategy_performance_daily',
    start_offset => INTERVAL '7 days',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');

-- Additional indexes for query optimization
CREATE INDEX idx_trades_okx_order_id ON trades(okx_order_id) WHERE okx_order_id IS NOT NULL;
CREATE INDEX idx_trades_filled ON trades(strategy_id, completed_at DESC) WHERE status = 'filled';
CREATE INDEX idx_positions_active ON positions(strategy_id) WHERE quantity > 0;

-- Partial index for active strategies
CREATE INDEX idx_strategies_active ON strategies(updated_at DESC) WHERE status = 'active';

-- GIN index for JSONB columns
CREATE INDEX idx_strategies_parameters ON strategies USING GIN (parameters);
CREATE INDEX idx_strategies_risk_limits ON strategies USING GIN (risk_limits);
CREATE INDEX idx_system_metrics_tags ON system_metrics USING GIN (tags);
