# Configuration Examples

## Development Configuration

```toml
# config/development.toml

[environment]
name = "development"
log_level = "debug"

[okx]
# Use testnet for development
api_url = "https://www.okx.com"  # Change to testnet URL
ws_url = "wss://ws.okx.com:8443/ws/v5/public"
api_key = "${OKX_API_KEY}"
secret_key = "${OKX_SECRET_KEY}"
passphrase = "${OKX_PASSPHRASE}"

[database]
host = "localhost"
port = 5432
database = "ea_okx_dev"
username = "postgres"
password = "${DB_PASSWORD}"
max_connections = 10
ssl_mode = "prefer"

[redis]
url = "redis://localhost:6379"
pool_size = 10

[trading]
# Conservative settings for development
max_leverage = 2.0
default_capital = 10000.0
max_position_size_pct = 0.10  # 10%
max_daily_loss_pct = 0.02      # 2%

[risk]
enable_pre_trade_checks = true
enable_position_limits = true
max_positions = 5

[monitoring]
metrics_port = 9090
enable_prometheus = true
alert_email = "dev@example.com"
```

## Production Configuration

```toml
# config/production.toml

[environment]
name = "production"
log_level = "info"

[okx]
api_url = "https://www.okx.com"
ws_url = "wss://ws.okx.com:8443/ws/v5/public"
# Load from environment variables
api_key = "${OKX_API_KEY}"
secret_key = "${OKX_SECRET_KEY}"
passphrase = "${OKX_PASSPHRASE}"

[database]
host = "${DB_HOST}"
port = 5432
database = "ea_okx_prod"
username = "${DB_USER}"
password = "${DB_PASSWORD}"
max_connections = 50
ssl_mode = "require"

[redis]
url = "${REDIS_URL}"
pool_size = 50

[trading]
max_leverage = 3.0
default_capital = 100000.0
max_position_size_pct = 0.20
max_daily_loss_pct = 0.05

[risk]
enable_pre_trade_checks = true
enable_position_limits = true
enable_circuit_breaker = true
max_positions = 20
var_confidence = 0.95

[monitoring]
metrics_port = 9090
enable_prometheus = true
alert_email = "alerts@example.com"
pagerduty_key = "${PAGERDUTY_KEY}"
```

## Symbol Configuration

```toml
# config/symbols.toml

[[symbols]]
name = "BTC-USDT"
enabled = true
min_order_size = 0.0001
max_order_size = 10.0
tick_size = 0.1
maker_fee = 0.0008
taker_fee = 0.001

[[symbols]]
name = "ETH-USDT"
enabled = true
min_order_size = 0.001
max_order_size = 100.0
tick_size = 0.01
maker_fee = 0.0008
taker_fee = 0.001

[[symbols]]
name = "SOL-USDT"
enabled = true
min_order_size = 0.1
max_order_size = 1000.0
tick_size = 0.001
maker_fee = 0.0008
taker_fee = 0.001
```

## Environment Variables

Create a `.env` file (never commit this to version control):

```bash
# .env.example

# OKX API Credentials
OKX_API_KEY=your-api-key-here
OKX_SECRET_KEY=your-secret-key-here
OKX_PASSPHRASE=your-passphrase-here

# Database
DB_HOST=localhost
DB_USER=postgres
DB_PASSWORD=your-db-password
DB_NAME=ea_okx

# Redis
REDIS_URL=redis://localhost:6379

# Monitoring
PAGERDUTY_KEY=your-pagerduty-key

# Application
RUST_LOG=info
ENVIRONMENT=development
```

## Docker Compose Configuration

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: timescale/timescaledb:latest-pg15
    environment:
      POSTGRES_DB: ea_okx
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

## Loading Configuration

```rust
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub environment: EnvironmentConfig,
    pub okx: OkxConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub monitoring: MonitoringConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        let config = Config::builder()
            .add_source(File::with_name(&format!("config/{}", env)))
            .add_source(Environment::with_prefix("APP"))
            .build()?;
        
        config.try_deserialize().map_err(Into::into)
    }
}
```
