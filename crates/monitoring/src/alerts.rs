use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Severity levels for alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric_name: String,
    pub operator: ComparisonOperator,
    pub threshold: f64,
    pub duration_seconds: u64,
}

/// Comparison operators for alert conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub last_triggered: Option<DateTime<Utc>>,
}

impl AlertRule {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        condition: AlertCondition,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            condition,
            severity,
            enabled: true,
            cooldown_seconds: 300, // 5 minutes default
            last_triggered: None,
        }
    }

    /// Check if the alert is in cooldown period
    pub fn is_in_cooldown(&self) -> bool {
        if let Some(last_triggered) = self.last_triggered {
            let elapsed = Utc::now().signed_duration_since(last_triggered);
            elapsed.num_seconds() < self.cooldown_seconds as i64
        } else {
            false
        }
    }

    /// Evaluate the alert condition against a metric value
    pub fn evaluate(&self, metric_value: f64) -> bool {
        if !self.enabled || self.is_in_cooldown() {
            return false;
        }

        match self.condition.operator {
            ComparisonOperator::GreaterThan => metric_value > self.condition.threshold,
            ComparisonOperator::LessThan => metric_value < self.condition.threshold,
            ComparisonOperator::Equals => (metric_value - self.condition.threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEquals => (metric_value - self.condition.threshold).abs() >= f64::EPSILON,
            ComparisonOperator::GreaterThanOrEqual => metric_value >= self.condition.threshold,
            ComparisonOperator::LessThanOrEqual => metric_value <= self.condition.threshold,
        }
    }
}

/// Triggered alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Alert {
    pub fn new(
        rule: &AlertRule,
        metric_value: f64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            severity: rule.severity,
            message: message.into(),
            metric_name: rule.condition.metric_name.clone(),
            metric_value,
            threshold: rule.condition.threshold,
            triggered_at: Utc::now(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
            metadata: HashMap::new(),
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self, user: impl Into<String>) {
        self.acknowledged = true;
        self.acknowledged_at = Some(Utc::now());
        self.acknowledged_by = Some(user.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_creation() {
        let condition = AlertCondition {
            metric_name: "cpu_usage".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 80.0,
            duration_seconds: 60,
        };

        let rule = AlertRule::new(
            "High CPU Usage",
            "CPU usage exceeded 80%",
            condition,
            AlertSeverity::Warning,
        );

        assert_eq!(rule.name, "High CPU Usage");
        assert_eq!(rule.severity, AlertSeverity::Warning);
        assert!(rule.enabled);
        assert_eq!(rule.cooldown_seconds, 300);
    }

    #[test]
    fn test_alert_evaluation() {
        let condition = AlertCondition {
            metric_name: "latency".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 100.0,
            duration_seconds: 30,
        };

        let rule = AlertRule::new(
            "High Latency",
            "Latency exceeded 100ms",
            condition,
            AlertSeverity::Critical,
        );

        assert!(rule.evaluate(150.0));
        assert!(!rule.evaluate(50.0));
        assert!(!rule.evaluate(100.0));
    }

    #[test]
    fn test_alert_acknowledgment() {
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

        let mut alert = Alert::new(&rule, 0.05, "Error rate: 5%");
        assert!(!alert.acknowledged);

        alert.acknowledge("admin");
        assert!(alert.acknowledged);
        assert_eq!(alert.acknowledged_by, Some("admin".to_string()));
        assert!(alert.acknowledged_at.is_some());
    }
}
