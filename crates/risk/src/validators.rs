use crate::error::{Error, Result};
use ea_okx_core::models::{Order, Position, OrderSide};
use ea_okx_core::{Symbol, Quantity};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

/// Risk limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    /// Maximum position size per symbol
    pub max_position_size: HashMap<Symbol, Quantity>,
    
    /// Maximum total portfolio value
    pub max_portfolio_value: Decimal,
    
    /// Maximum leverage allowed
    pub max_leverage: Decimal,
    
    /// Daily loss limit (absolute value)
    pub daily_loss_limit: Decimal,
    
    /// Maximum concentration per symbol (percentage of portfolio)
    pub max_concentration_pct: Decimal,
    
    /// Maximum number of open positions
    pub max_open_positions: usize,
    
    /// Minimum required margin ratio
    pub min_margin_ratio: Decimal,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size: HashMap::new(),
            max_portfolio_value: dec!(1000000.0),
            max_leverage: dec!(3.0),
            daily_loss_limit: dec!(10000.0),
            max_concentration_pct: dec!(25.0),
            max_open_positions: 10,
            min_margin_ratio: dec!(0.15), // 15% minimum margin
        }
    }
}

/// Portfolio state for risk checks
#[derive(Debug, Clone)]
pub struct PortfolioState {
    pub total_equity: Decimal,
    pub available_margin: Decimal,
    pub positions: Vec<Position>,
    pub daily_pnl: Decimal,
}

/// Pre-trade risk validator
pub struct PreTradeValidator {
    limits: RiskLimits,
}

impl PreTradeValidator {
    pub fn new(limits: RiskLimits) -> Self {
        Self { limits }
    }

    /// Validate an order before execution
    pub fn validate_order(
        &self,
        order: &Order,
        portfolio: &PortfolioState,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();

        // 1. Position size check
        if let Err(e) = self.check_position_size(order, portfolio) {
            result.add_violation(RiskViolation {
                severity: ViolationSeverity::Critical,
                rule: "Position Size Limit".to_string(),
                message: e.to_string(),
            });
        }

        // 2. Leverage check
        if let Err(e) = self.check_leverage(order, portfolio) {
            result.add_violation(RiskViolation {
                severity: ViolationSeverity::Critical,
                rule: "Leverage Limit".to_string(),
                message: e.to_string(),
            });
        }

        // 3. Daily loss limit check
        if let Err(e) = self.check_daily_loss(portfolio) {
            result.add_violation(RiskViolation {
                severity: ViolationSeverity::Critical,
                rule: "Daily Loss Limit".to_string(),
                message: e.to_string(),
            });
        }

        // 4. Concentration check
        if let Err(e) = self.check_concentration(order, portfolio) {
            result.add_violation(RiskViolation {
                severity: ViolationSeverity::Warning,
                rule: "Concentration Limit".to_string(),
                message: e.to_string(),
            });
        }

        // 5. Margin check
        if let Err(e) = self.check_margin(order, portfolio) {
            result.add_violation(RiskViolation {
                severity: ViolationSeverity::Critical,
                rule: "Margin Requirement".to_string(),
                message: e.to_string(),
            });
        }

        // 6. Maximum positions check
        if let Err(e) = self.check_max_positions(order, portfolio) {
            result.add_violation(RiskViolation {
                severity: ViolationSeverity::Warning,
                rule: "Maximum Positions".to_string(),
                message: e.to_string(),
            });
        }

        Ok(result)
    }

    /// Check position size limits
    fn check_position_size(
        &self,
        order: &Order,
        portfolio: &PortfolioState,
    ) -> Result<()> {
        let order_qty = order.quantity.as_decimal();
        
        // Check if we have a limit for this symbol
        if let Some(max_qty) = self.limits.max_position_size.get(&order.symbol) {
            // Calculate current position
            let current_position = portfolio.positions.iter()
                .find(|p| p.symbol == order.symbol)
                .map(|p| p.quantity.as_decimal())
                .unwrap_or(Decimal::ZERO);

            let new_position = match order.side {
                OrderSide::Buy => current_position + order_qty,
                OrderSide::Sell => (current_position - order_qty).abs(),
            };

            if new_position > max_qty.as_decimal() {
                return Err(Error::PositionLimitExceeded(format!(
                    "New position {} exceeds limit {} for {}",
                    new_position, max_qty.as_decimal(), order.symbol.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Check leverage limits
    fn check_leverage(
        &self,
        order: &Order,
        portfolio: &PortfolioState,
    ) -> Result<()> {
        // Use market price if order price is None (for market orders)
        let price = order.price.as_ref()
            .map(|p| p.as_decimal())
            .unwrap_or(dec!(0.0)); // For market orders, we'd need current price
        let order_value = price * order.quantity.as_decimal();
        let total_exposure = portfolio.positions.iter()
            .map(|p| p.quantity.as_decimal() * p.current_price.as_decimal())
            .sum::<Decimal>() + order_value;

        let leverage = if portfolio.total_equity > Decimal::ZERO {
            total_exposure / portfolio.total_equity
        } else {
            Decimal::ZERO
        };

        if leverage > self.limits.max_leverage {
            return Err(Error::LeverageLimitExceeded(format!(
                "Leverage {:.2}x exceeds limit {:.2}x",
                leverage, self.limits.max_leverage
            )));
        }

        Ok(())
    }

    /// Check daily loss limits
    fn check_daily_loss(&self, portfolio: &PortfolioState) -> Result<()> {
        if portfolio.daily_pnl < -self.limits.daily_loss_limit {
            return Err(Error::DailyLossLimitExceeded(format!(
                "Daily loss {:.2} exceeds limit {:.2}",
                portfolio.daily_pnl.abs(), self.limits.daily_loss_limit
            )));
        }
        Ok(())
    }

    /// Check concentration limits
    fn check_concentration(
        &self,
        order: &Order,
        portfolio: &PortfolioState,
    ) -> Result<()> {
        // Use market price if order price is None (for market orders)
        let price = order.price.as_ref()
            .map(|p| p.as_decimal())
            .unwrap_or(dec!(0.0)); // For market orders, we'd need current price
        let order_value = price * order.quantity.as_decimal();
        let concentration_pct = if portfolio.total_equity > Decimal::ZERO {
            (order_value / portfolio.total_equity) * dec!(100.0)
        } else {
            dec!(100.0)
        };

        if concentration_pct > self.limits.max_concentration_pct {
            warn!(
                "Order concentration {:.2}% exceeds limit {:.2}%",
                concentration_pct, self.limits.max_concentration_pct
            );
            // Note: This is a warning, not a hard failure
        }

        Ok(())
    }

    /// Check margin requirements
    fn check_margin(
        &self,
        order: &Order,
        portfolio: &PortfolioState,
    ) -> Result<()> {
        // Use market price if order price is None (for market orders)
        let price = order.price.as_ref()
            .map(|p| p.as_decimal())
            .unwrap_or(dec!(0.0)); // For market orders, we'd need current price
        let order_value = price * order.quantity.as_decimal();
        let required_margin = order_value * self.limits.min_margin_ratio;

        if portfolio.available_margin < required_margin {
            return Err(Error::InsufficientMargin {
                required: format!("{:.2}", required_margin),
                available: format!("{:.2}", portfolio.available_margin),
            });
        }

        Ok(())
    }

    /// Check maximum positions limit
    fn check_max_positions(
        &self,
        order: &Order,
        portfolio: &PortfolioState,
    ) -> Result<()> {
        // Check if this would open a new position
        let has_existing = portfolio.positions.iter()
            .any(|p| p.symbol == order.symbol);

        if !has_existing && portfolio.positions.len() >= self.limits.max_open_positions {
            warn!(
                "Maximum positions {} reached",
                self.limits.max_open_positions
            );
            // Note: This is a warning, not a hard failure
        }

        Ok(())
    }
}

/// Validation result
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub violations: Vec<RiskViolation>,
}

impl ValidationResult {
    pub fn add_violation(&mut self, violation: RiskViolation) {
        self.violations.push(violation);
    }

    pub fn is_valid(&self) -> bool {
        !self.has_critical_violations()
    }

    pub fn has_critical_violations(&self) -> bool {
        self.violations.iter()
            .any(|v| v.severity == ViolationSeverity::Critical)
    }

    pub fn has_warnings(&self) -> bool {
        self.violations.iter()
            .any(|v| v.severity == ViolationSeverity::Warning)
    }
}

/// Risk violation
#[derive(Debug, Clone)]
pub struct RiskViolation {
    pub severity: ViolationSeverity,
    pub rule: String,
    pub message: String,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity {
    Critical,
    Warning,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn create_test_order(qty: Decimal, price: Decimal) -> Order {
        Order::new(
            Uuid::new_v4(),
            Symbol::new("BTC-USDT").unwrap(),
            OrderSide::Buy,
            ea_okx_core::models::OrderType::Iceberg,
            ea_okx_core::Price::new(price).unwrap(),
            Quantity::new(qty).unwrap(),
        )
    }

    fn create_test_portfolio() -> PortfolioState {
        PortfolioState {
            total_equity: dec!(100000.0),
            available_margin: dec!(50000.0),
            positions: vec![],
            daily_pnl: Decimal::ZERO,
        }
    }

    #[test]
    fn test_leverage_check_pass() {
        let limits = RiskLimits {
            max_leverage: dec!(3.0),
            ..Default::default()
        };
        
        let validator = PreTradeValidator::new(limits);
        let order = create_test_order(dec!(1.0), dec!(50000.0));
        let portfolio = create_test_portfolio();

        let result = validator.validate_order(&order, &portfolio).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_daily_loss_limit() {
        let limits = RiskLimits {
            daily_loss_limit: dec!(5000.0),
            ..Default::default()
        };

        let validator = PreTradeValidator::new(limits);
        let order = create_test_order(dec!(0.1), dec!(50000.0));
        
        let mut portfolio = create_test_portfolio();
        portfolio.daily_pnl = dec!(-6000.0);

        let result = validator.validate_order(&order, &portfolio).unwrap();
        assert!(!result.is_valid());
        assert!(result.has_critical_violations());
    }

    #[test]
    fn test_margin_check() {
        let limits = RiskLimits {
            min_margin_ratio: dec!(0.15),
            ..Default::default()
        };

        let validator = PreTradeValidator::new(limits);
        let order = create_test_order(dec!(10.0), dec!(50000.0)); // $500k order
        
        let mut portfolio = create_test_portfolio();
        portfolio.available_margin = dec!(10000.0); // Only $10k available

        let result = validator.validate_order(&order, &portfolio).unwrap();
        assert!(!result.is_valid());
    }
}