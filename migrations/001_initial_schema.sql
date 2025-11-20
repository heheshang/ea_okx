-- Initial database schema for EA OKX quantitative trading system
-- PostgreSQL 15+ with TimescaleDB extension

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "timescaledb";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('admin', 'manager', 'trader', 'analyst', 'auditor')),
    two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    two_factor_secret VARCHAR(255),
    api_key_hash VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role) WHERE is_active = TRUE;

-- Strategies table
CREATE TABLE strategies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    strategy_type VARCHAR(50) NOT NULL,
    version VARCHAR(20) NOT NULL,
    parameters JSONB NOT NULL,
    risk_limits JSONB NOT NULL,
    symbols TEXT[] NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('draft', 'validating', 'backtesting', 'papertrading', 'active', 'paused', 'stopped', 'archived')),
    allocated_capital DECIMAL(20,8) NOT NULL CHECK (allocated_capital > 0),
    max_position_size DECIMAL(20,8) NOT NULL CHECK (max_position_size > 0),
    max_leverage DECIMAL(10,2) NOT NULL CHECK (max_leverage > 0 AND max_leverage <= 10),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deployed_at TIMESTAMPTZ,
    stopped_at TIMESTAMPTZ
);

CREATE INDEX idx_strategies_status ON strategies(status);
CREATE INDEX idx_strategies_type ON strategies(strategy_type);
CREATE INDEX idx_strategies_created_by ON strategies(created_by);

-- Market OHLCV data (TimescaleDB hypertable)
CREATE TABLE market_ohlcv (
    id BIGSERIAL,
    symbol VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    interval VARCHAR(10) NOT NULL CHECK (interval IN ('1m', '5m', '15m', '1h', '4h', '1d')),
    open DECIMAL(20,8) NOT NULL CHECK (open > 0),
    high DECIMAL(20,8) NOT NULL CHECK (high >= open AND high >= low),
    low DECIMAL(20,8) NOT NULL CHECK (low <= close AND low <= high),
    close DECIMAL(20,8) NOT NULL CHECK (close > 0),
    volume DECIMAL(20,8) NOT NULL CHECK (volume >= 0),
    quote_volume DECIMAL(20,8) NOT NULL CHECK (quote_volume >= 0),
    trade_count INTEGER NOT NULL CHECK (trade_count >= 0),
    vwap DECIMAL(20,8),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (symbol, interval, timestamp)
);

-- Convert to hypertable
SELECT create_hypertable('market_ohlcv', 'timestamp', chunk_time_interval => INTERVAL '7 days');

-- Create indexes
CREATE INDEX idx_market_ohlcv_symbol_time ON market_ohlcv(symbol, timestamp DESC);

-- Compression policy (after 30 days)
SELECT add_compression_policy('market_ohlcv', INTERVAL '30 days');

-- Retention policy
SELECT add_retention_policy('market_ohlcv', INTERVAL '5 years');

-- Market ticks table
CREATE TABLE market_ticks (
    id BIGSERIAL,
    symbol VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    trade_id VARCHAR(50) NOT NULL UNIQUE,
    price DECIMAL(20,8) NOT NULL CHECK (price > 0),
    quantity DECIMAL(20,8) NOT NULL CHECK (quantity > 0),
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    is_block_trade BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (symbol, timestamp, id)
);

SELECT create_hypertable('market_ticks', 'timestamp', chunk_time_interval => INTERVAL '1 day');
CREATE INDEX idx_market_ticks_symbol_time ON market_ticks(symbol, timestamp DESC);

-- Compression and retention
SELECT add_compression_policy('market_ticks', INTERVAL '7 days');
SELECT add_retention_policy('market_ticks', INTERVAL '90 days');

-- Order book snapshots
CREATE TABLE order_book_snapshots (
    id BIGSERIAL,
    symbol VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    bids JSONB NOT NULL,
    asks JSONB NOT NULL,
    checksum INTEGER,
    depth_level VARCHAR(10) NOT NULL,
    PRIMARY KEY (symbol, timestamp, id)
);

SELECT create_hypertable('order_book_snapshots', 'timestamp', chunk_time_interval => INTERVAL '1 day');

-- Compression and retention
SELECT add_compression_policy('order_book_snapshots', INTERVAL '3 days');
SELECT add_retention_policy('order_book_snapshots', INTERVAL '7 days');

-- Trades table
CREATE TABLE trades (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    okx_order_id VARCHAR(50) UNIQUE,
    client_order_id VARCHAR(50) NOT NULL UNIQUE,
    strategy_id UUID NOT NULL REFERENCES strategies(id),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    order_type VARCHAR(20) NOT NULL CHECK (order_type IN ('market', 'limit', 'stop_loss', 'take_profit', 'post_only')),
    position_side VARCHAR(10) NOT NULL CHECK (position_side IN ('long', 'short', 'net')),
    quantity DECIMAL(20,8) NOT NULL CHECK (quantity > 0),
    price DECIMAL(20,8) CHECK (price > 0),
    avg_fill_price DECIMAL(20,8),
    filled_quantity DECIMAL(20,8) NOT NULL CHECK (filled_quantity >= 0 AND filled_quantity <= quantity),
    commission DECIMAL(20,8) NOT NULL CHECK (commission >= 0),
    commission_asset VARCHAR(10) NOT NULL,
    realized_pnl DECIMAL(20,8),
    slippage_bps INTEGER,
    status VARCHAR(20) NOT NULL CHECK (status IN ('created', 'submitted', 'partial', 'filled', 'cancelled', 'rejected', 'failed')),
    reject_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_at TIMESTAMPTZ,
    first_fill_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    latency_ms INTEGER,
    total_latency_ms INTEGER
);

CREATE INDEX idx_trades_strategy_id ON trades(strategy_id, created_at DESC);
CREATE INDEX idx_trades_symbol ON trades(symbol, created_at DESC);
CREATE INDEX idx_trades_status ON trades(status) WHERE status IN ('submitted', 'partial');
CREATE INDEX idx_trades_created_at ON trades(created_at DESC);

-- Positions table
CREATE TABLE positions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    strategy_id UUID NOT NULL REFERENCES strategies(id),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL CHECK (side IN ('long', 'short', 'net')),
    quantity DECIMAL(20,8) NOT NULL CHECK (quantity >= 0),
    avg_entry_price DECIMAL(20,8) NOT NULL CHECK (avg_entry_price > 0),
    current_price DECIMAL(20,8) NOT NULL CHECK (current_price > 0),
    unrealized_pnl DECIMAL(20,8) NOT NULL,
    realized_pnl DECIMAL(20,8) NOT NULL,
    margin DECIMAL(20,8),
    leverage DECIMAL(10,2),
    liquidation_price DECIMAL(20,8),
    opened_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(strategy_id, symbol)
);

CREATE INDEX idx_positions_strategy_id ON positions(strategy_id);
CREATE INDEX idx_positions_symbol ON positions(symbol);

-- Strategy performance table
CREATE TABLE strategy_performance (
    id BIGSERIAL PRIMARY KEY,
    strategy_id UUID NOT NULL REFERENCES strategies(id),
    date DATE NOT NULL,
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    total_pnl DECIMAL(20,8) NOT NULL DEFAULT 0,
    total_commission DECIMAL(20,8) NOT NULL DEFAULT 0,
    net_pnl DECIMAL(20,8) NOT NULL DEFAULT 0,
    return_pct DECIMAL(10,4) NOT NULL DEFAULT 0,
    sharpe_ratio DECIMAL(10,4),
    max_drawdown DECIMAL(10,4) NOT NULL DEFAULT 0,
    win_rate DECIMAL(10,4) NOT NULL DEFAULT 0,
    avg_win DECIMAL(20,8),
    avg_loss DECIMAL(20,8),
    profit_factor DECIMAL(10,4),
    total_volume DECIMAL(20,8) NOT NULL DEFAULT 0,
    UNIQUE(strategy_id, date)
);

CREATE INDEX idx_strategy_performance_strategy ON strategy_performance(strategy_id, date DESC);

-- Risk events table
CREATE TABLE risk_events (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('info', 'warning', 'critical', 'emergency')),
    strategy_id UUID REFERENCES strategies(id),
    symbol VARCHAR(20),
    description TEXT NOT NULL,
    metric_name VARCHAR(100),
    metric_value DECIMAL(20,8),
    threshold_value DECIMAL(20,8),
    action_taken VARCHAR(50),
    acknowledged BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_risk_events_severity ON risk_events(severity, created_at DESC) WHERE NOT acknowledged;
CREATE INDEX idx_risk_events_strategy ON risk_events(strategy_id, created_at DESC);

-- System metrics table
CREATE TABLE system_metrics (
    id BIGSERIAL,
    timestamp TIMESTAMPTZ NOT NULL,
    metric_category VARCHAR(50) NOT NULL CHECK (metric_category IN ('system', 'data', 'trading', 'risk')),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(20,8) NOT NULL,
    unit VARCHAR(20),
    tags JSONB,
    PRIMARY KEY (timestamp, metric_category, metric_name)
);

SELECT create_hypertable('system_metrics', 'timestamp', chunk_time_interval => INTERVAL '1 day');
SELECT add_compression_policy('system_metrics', INTERVAL '14 days');
SELECT add_retention_policy('system_metrics', INTERVAL '180 days');

-- Audit logs table
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(100),
    old_value JSONB,
    new_value JSONB,
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user ON audit_logs(user_id, created_at DESC);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Update triggers for updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_strategies_updated_at BEFORE UPDATE ON strategies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_positions_last_updated BEFORE UPDATE ON positions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
