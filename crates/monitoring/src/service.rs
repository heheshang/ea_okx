use crate::alerts::{Alert, AlertRule};
use crate::error::Result;
use crate::metrics::{HealthCheck, HealthReport, MetricsCollector, PerformanceSnapshot};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Monitoring service that coordinates metrics collection, health checks, and alerting
pub struct MonitoringService {
    metrics: Arc<MetricsCollector>,
    alert_rules: Arc<RwLock<HashMap<Uuid, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    health_checks: Arc<RwLock<Vec<Box<dyn HealthChecker>>>>,
}

/// Trait for components that can perform health checks
#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check(&self) -> HealthCheck;
    fn name(&self) -> &str;
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            health_checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get reference to metrics collector
    pub fn metrics(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    /// Register a new alert rule
    pub async fn register_alert_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        rules.insert(rule.id, rule);
        Ok(())
    }

    /// Remove an alert rule
    pub async fn remove_alert_rule(&self, rule_id: Uuid) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        rules.remove(&rule_id);
        Ok(())
    }

    /// Get all alert rules
    pub async fn get_alert_rules(&self) -> Vec<AlertRule> {
        let rules = self.alert_rules.read().await;
        rules.values().cloned().collect()
    }

    /// Evaluate all alert rules against a metric
    pub async fn evaluate_metric(&self, metric_name: &str, value: f64) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        let mut alerts = self.active_alerts.write().await;

        for rule in rules.values_mut() {
            if rule.condition.metric_name == metric_name && rule.evaluate(value) {
                let message = format!(
                    "{}: {} (threshold: {})",
                    rule.name, value, rule.condition.threshold
                );
                
                let alert = Alert::new(rule, value, message);
                alerts.insert(alert.id, alert.clone());
                
                // Update last triggered time
                rule.last_triggered = Some(Utc::now());
                
                tracing::warn!(
                    rule_name = %rule.name,
                    metric_name = %metric_name,
                    value = %value,
                    threshold = %rule.condition.threshold,
                    "Alert triggered"
                );
            }
        }

        Ok(())
    }

    /// Get all active (unacknowledged) alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.values()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    /// Get all alerts (including acknowledged)
    pub async fn get_all_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, user: impl Into<String>) -> Result<()> {
        let mut alerts = self.active_alerts.write().await;
        
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.acknowledge(user);
            tracing::info!(
                alert_id = %alert_id,
                rule_name = %alert.rule_name,
                "Alert acknowledged"
            );
        }

        Ok(())
    }

    /// Register a health checker
    pub async fn register_health_checker(&self, checker: Box<dyn HealthChecker>) -> Result<()> {
        let mut checkers = self.health_checks.write().await;
        checkers.push(checker);
        Ok(())
    }

    /// Perform health check on all registered components
    pub async fn perform_health_check(&self) -> HealthReport {
        let checkers = self.health_checks.read().await;
        
        let mut checks = Vec::new();
        for checker in checkers.iter() {
            let check = checker.check().await;
            checks.push(check);
        }

        HealthReport::new(checks)
    }

    /// Get performance snapshot
    pub fn get_performance_snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            timestamp: Utc::now(),
            orders_submitted: 0, // These would be tracked separately
            orders_filled: 0,
            orders_cancelled: 0,
            orders_rejected: 0,
            trades_executed: 0,
            active_positions: 0,
            portfolio_value: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            avg_order_latency_ms: 0.0,
            p95_order_latency_ms: 0.0,
            p99_order_latency_ms: 0.0,
        }
    }

    /// Start monitoring background tasks
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Monitoring service started");
        Ok(())
    }

    /// Stop monitoring service
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Monitoring service stopped");
        Ok(())
    }
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new()
    }
}

/// Example health checker for database
pub struct DatabaseHealthChecker {
    name: String,
}

impl DatabaseHealthChecker {
    pub fn new() -> Self {
        Self {
            name: "database".to_string(),
        }
    }
}

#[async_trait]
impl HealthChecker for DatabaseHealthChecker {
    async fn check(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        
        // Simulate database check
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        // Example logic: healthy if response < 50ms
        if elapsed < 50 {
            HealthCheck::healthy(&self.name, "Database connection OK", elapsed)
        } else if elapsed < 100 {
            HealthCheck::degraded(&self.name, "Database response slow", elapsed)
        } else {
            HealthCheck::unhealthy(&self.name, "Database timeout", elapsed)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Example health checker for exchange API
pub struct ExchangeHealthChecker {
    name: String,
}

impl ExchangeHealthChecker {
    pub fn new() -> Self {
        Self {
            name: "exchange".to_string(),
        }
    }
}

#[async_trait]
impl HealthChecker for ExchangeHealthChecker {
    async fn check(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        
        // Simulate exchange API check
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        if elapsed < 100 {
            HealthCheck::healthy(&self.name, "Exchange API responsive", elapsed)
        } else {
            HealthCheck::degraded(&self.name, "Exchange API slow", elapsed)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertCondition, AlertSeverity, ComparisonOperator};
    use crate::metrics::HealthStatus;

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let service = MonitoringService::new();
        let rules = service.get_alert_rules().await;
        assert_eq!(rules.len(), 0);
    }

    #[tokio::test]
    async fn test_register_alert_rule() {
        let service = MonitoringService::new();
        
        let condition = AlertCondition {
            metric_name: "latency".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 100.0,
            duration_seconds: 60,
        };

        let rule = AlertRule::new(
            "High Latency",
            "Latency exceeded threshold",
            condition,
            AlertSeverity::Warning,
        );

        service.register_alert_rule(rule.clone()).await.unwrap();
        
        let rules = service.get_alert_rules().await;
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "High Latency");
    }

    #[tokio::test]
    async fn test_evaluate_metric() {
        let service = MonitoringService::new();
        
        let condition = AlertCondition {
            metric_name: "error_rate".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 0.01,
            duration_seconds: 60,
        };

        let rule = AlertRule::new(
            "High Error Rate",
            "Error rate exceeded 1%",
            condition,
            AlertSeverity::Critical,
        );

        service.register_alert_rule(rule).await.unwrap();
        
        // Trigger the alert
        service.evaluate_metric("error_rate", 0.05).await.unwrap();
        
        let active_alerts = service.get_active_alerts().await;
        assert_eq!(active_alerts.len(), 1);
        assert_eq!(active_alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_acknowledge_alert() {
        let service = MonitoringService::new();
        
        let condition = AlertCondition {
            metric_name: "cpu_usage".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 80.0,
            duration_seconds: 30,
        };

        let rule = AlertRule::new(
            "High CPU",
            "CPU usage high",
            condition,
            AlertSeverity::Warning,
        );

        service.register_alert_rule(rule).await.unwrap();
        service.evaluate_metric("cpu_usage", 95.0).await.unwrap();
        
        let active_alerts = service.get_active_alerts().await;
        assert_eq!(active_alerts.len(), 1);
        
        let alert_id = active_alerts[0].id;
        service.acknowledge_alert(alert_id, "admin").await.unwrap();
        
        let active_alerts_after = service.get_active_alerts().await;
        assert_eq!(active_alerts_after.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let service = MonitoringService::new();
        
        let db_checker = Box::new(DatabaseHealthChecker::new());
        let exchange_checker = Box::new(ExchangeHealthChecker::new());
        
        service.register_health_checker(db_checker).await.unwrap();
        service.register_health_checker(exchange_checker).await.unwrap();
        
        let report = service.perform_health_check().await;
        
        assert_eq!(report.components.len(), 2);
        assert!(matches!(report.overall_status, HealthStatus::Healthy | HealthStatus::Degraded));
    }
}
