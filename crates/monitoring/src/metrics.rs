use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub checked_at: DateTime<Utc>,
    pub response_time_ms: u64,
}

impl HealthCheck {
    pub fn healthy(
        component: impl Into<String>,
        message: impl Into<String>,
        response_time_ms: u64,
    ) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            message: message.into(),
            checked_at: Utc::now(),
            response_time_ms,
        }
    }

    pub fn degraded(
        component: impl Into<String>,
        message: impl Into<String>,
        response_time_ms: u64,
    ) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Degraded,
            message: message.into(),
            checked_at: Utc::now(),
            response_time_ms,
        }
    }

    pub fn unhealthy(
        component: impl Into<String>,
        message: impl Into<String>,
        response_time_ms: u64,
    ) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            message: message.into(),
            checked_at: Utc::now(),
            response_time_ms,
        }
    }
}

/// System-wide health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub components: Vec<HealthCheck>,
    pub generated_at: DateTime<Utc>,
}

impl HealthReport {
    pub fn new(components: Vec<HealthCheck>) -> Self {
        // Overall status is the worst status among all components
        let overall_status = components
            .iter()
            .map(|c| c.status)
            .max_by_key(|s| match s {
                HealthStatus::Healthy => 0,
                HealthStatus::Degraded => 1,
                HealthStatus::Unhealthy => 2,
            })
            .unwrap_or(HealthStatus::Healthy);

        Self {
            overall_status,
            components,
            generated_at: Utc::now(),
        }
    }
}

/// Metrics collector for trading system
/// This is a placeholder that uses tracing for logging metrics
/// In production, integrate with Prometheus or similar
pub struct MetricsCollector;

impl MetricsCollector {
    pub fn new() -> Self {
        Self
    }

    // Counter methods
    pub fn increment_orders_submitted(&self) {
        tracing::debug!(
            metric = "orders_submitted_total",
            value = 1,
            "Increment counter"
        );
    }

    pub fn increment_orders_filled(&self) {
        tracing::debug!(
            metric = "orders_filled_total",
            value = 1,
            "Increment counter"
        );
    }

    pub fn increment_orders_cancelled(&self) {
        tracing::debug!(
            metric = "orders_cancelled_total",
            value = 1,
            "Increment counter"
        );
    }

    pub fn increment_orders_rejected(&self) {
        tracing::debug!(
            metric = "orders_rejected_total",
            value = 1,
            "Increment counter"
        );
    }

    pub fn increment_trades_executed(&self) {
        tracing::debug!(
            metric = "trades_executed_total",
            value = 1,
            "Increment counter"
        );
    }

    // Gauge methods
    pub fn set_active_positions(&self, count: u64) {
        tracing::debug!(metric = "active_positions", value = count, "Set gauge");
    }

    pub fn set_portfolio_value(&self, value: f64) {
        tracing::debug!(metric = "portfolio_value_usd", value = value, "Set gauge");
    }

    pub fn set_unrealized_pnl(&self, pnl: f64) {
        tracing::debug!(metric = "unrealized_pnl_usd", value = pnl, "Set gauge");
    }

    pub fn set_realized_pnl(&self, pnl: f64) {
        tracing::debug!(metric = "realized_pnl_usd", value = pnl, "Set gauge");
    }

    // Histogram methods
    pub fn record_order_latency(&self, latency_ms: f64) {
        tracing::debug!(
            metric = "order_latency_ms",
            value = latency_ms,
            "Record histogram"
        );
    }

    pub fn record_api_latency(&self, latency_ms: f64) {
        tracing::debug!(
            metric = "api_latency_ms",
            value = latency_ms,
            "Record histogram"
        );
    }

    pub fn record_strategy_execution_time(&self, duration_ms: f64) {
        tracing::debug!(
            metric = "strategy_execution_time_ms",
            value = duration_ms,
            "Record histogram"
        );
    }

    /// Helper to measure and record latency
    pub fn measure_latency<F, R>(&self, metric: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        match metric {
            "order" => self.record_order_latency(duration_ms),
            "api" => self.record_api_latency(duration_ms),
            "strategy" => self.record_strategy_execution_time(duration_ms),
            _ => {}
        }

        result
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance snapshot for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub orders_submitted: u64,
    pub orders_filled: u64,
    pub orders_cancelled: u64,
    pub orders_rejected: u64,
    pub trades_executed: u64,
    pub active_positions: u64,
    pub portfolio_value: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub avg_order_latency_ms: f64,
    pub p95_order_latency_ms: f64,
    pub p99_order_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_creation() {
        let check = HealthCheck::healthy("database", "Connection successful", 15);
        assert_eq!(check.component, "database");
        assert_eq!(check.status, HealthStatus::Healthy);
        assert_eq!(check.response_time_ms, 15);
    }

    #[test]
    fn test_health_report_overall_status() {
        let checks = vec![
            HealthCheck::healthy("database", "OK", 10),
            HealthCheck::degraded("cache", "Slow response", 150),
            HealthCheck::healthy("api", "OK", 20),
        ];

        let report = HealthReport::new(checks);
        assert_eq!(report.overall_status, HealthStatus::Degraded);
        assert_eq!(report.components.len(), 3);
    }

    #[test]
    fn test_health_report_unhealthy() {
        let checks = vec![
            HealthCheck::healthy("database", "OK", 10),
            HealthCheck::unhealthy("exchange", "Connection failed", 0),
        ];

        let report = HealthReport::new(checks);
        assert_eq!(report.overall_status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();

        // Test counter increments
        collector.increment_orders_submitted();
        collector.increment_orders_filled();

        // Test gauge updates
        collector.set_active_positions(5);
        collector.set_portfolio_value(100000.0);

        // Test histogram recordings
        collector.record_order_latency(25.5);
        collector.record_api_latency(50.0);
    }

    #[test]
    fn test_measure_latency() {
        let collector = MetricsCollector::new();

        let result = collector.measure_latency("order", || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
    }
}
