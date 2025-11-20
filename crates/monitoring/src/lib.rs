//! # EA OKX Monitoring
//!
//! Monitoring, metrics collection, and alerting for the EA OKX trading system.
//!
//! ## Features
//!
//! - **Metrics Collection**: Track trading performance, system health, and operational metrics
//! - **Health Checks**: Monitor component health (database, exchange API, cache)
//! - **Alerting**: Configurable alert rules with severity levels and cooldown periods
//! - **Performance Tracking**: Real-time performance snapshots and historical data
//!
//! ## Usage
//!
//! ```no_run
//! use ea_okx_monitoring::{MonitoringService, alerts::{AlertRule, AlertCondition, ComparisonOperator, AlertSeverity}};
//!
//! #[tokio::main]
//! async fn main() {
//!     let service = MonitoringService::new();
//!     
//!     // Register an alert rule
//!     let condition = AlertCondition {
//!         metric_name: "order_latency".to_string(),
//!         operator: ComparisonOperator::GreaterThan,
//!         threshold: 100.0,
//!         duration_seconds: 60,
//!     };
//!     
//!     let rule = AlertRule::new(
//!         "High Order Latency",
//!         "Order latency exceeded 100ms",
//!         condition,
//!         AlertSeverity::Warning,
//!     );
//!     
//!     service.register_alert_rule(rule).await.unwrap();
//!     
//!     // Evaluate metrics
//!     service.evaluate_metric("order_latency", 125.0).await.unwrap();
//!     
//!     // Check for alerts
//!     let alerts = service.get_active_alerts().await;
//!     println!("Active alerts: {}", alerts.len());
//! }
//! ```

pub mod alerts;
pub mod error;
pub mod metrics;
pub mod service;

pub use alerts::{Alert, AlertCondition, AlertRule, AlertSeverity, ComparisonOperator};
pub use error::{Error, Result};
pub use metrics::{HealthCheck, HealthReport, HealthStatus, MetricsCollector, PerformanceSnapshot};
pub use service::{DatabaseHealthChecker, ExchangeHealthChecker, HealthChecker, MonitoringService};
