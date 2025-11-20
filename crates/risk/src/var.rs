use crate::error::{Error, Result};
use ea_okx_core::models::Position;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// VaR calculation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarMethod {
    /// Historical simulation
    Historical,
    /// Parametric (variance-covariance)
    Parametric,
    /// Monte Carlo simulation
    MonteCarlo,
}

/// VaR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarConfig {
    /// Confidence level (e.g., 0.95, 0.99)
    pub confidence_level: f64,
    
    /// Time horizon in days
    pub time_horizon_days: u32,
    
    /// Historical lookback period in days
    pub lookback_days: u32,
    
    /// Calculation method
    pub method: VarMethod,
    
    /// Number of Monte Carlo simulations
    pub monte_carlo_simulations: usize,
}

impl Default for VarConfig {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            time_horizon_days: 1,
            lookback_days: 252, // 1 year
            method: VarMethod::Historical,
            monte_carlo_simulations: 10000,
        }
    }
}

/// VaR calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarResult {
    /// Value at Risk amount
    pub var_amount: Decimal,
    
    /// As percentage of portfolio value
    pub var_percentage: Decimal,
    
    /// Confidence level used
    pub confidence_level: f64,
    
    /// Method used
    pub method: VarMethod,
    
    /// Component VaRs by position
    pub component_vars: HashMap<String, Decimal>,
}

/// VaR calculator
pub struct VarCalculator {
    config: VarConfig,
}

impl VarCalculator {
    pub fn new(config: VarConfig) -> Self {
        Self { config }
    }

    /// Calculate VaR for a portfolio
    pub fn calculate_var(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>], // Returns for each position
        portfolio_value: Decimal,
    ) -> Result<VarResult> {
        match self.config.method {
            VarMethod::Historical => {
                self.calculate_historical_var(positions, historical_returns, portfolio_value)
            }
            VarMethod::Parametric => {
                self.calculate_parametric_var(positions, historical_returns, portfolio_value)
            }
            VarMethod::MonteCarlo => {
                self.calculate_monte_carlo_var(positions, historical_returns, portfolio_value)
            }
        }
    }

    /// Historical simulation VaR
    fn calculate_historical_var(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>],
        portfolio_value: Decimal,
    ) -> Result<VarResult> {
        if historical_returns.is_empty() {
            return Err(Error::CalculationError("No historical data".to_string()));
        }

        // Calculate portfolio returns for each historical period
        let portfolio_returns = self.calculate_portfolio_returns(positions, historical_returns)?;

        // Sort returns
        let mut sorted_returns = portfolio_returns.clone();
        sorted_returns.sort();

        // Find VaR at confidence level
        let index = ((1.0 - self.config.confidence_level) * sorted_returns.len() as f64) as usize;
        let var_return = sorted_returns.get(index).copied().unwrap_or(Decimal::ZERO);
        
        let var_amount = (var_return.abs()) * portfolio_value;
        let var_percentage = var_return.abs() * dec!(100.0);

        // Calculate component VaRs
        let component_vars = self.calculate_component_vars(positions, historical_returns)?;

        Ok(VarResult {
            var_amount,
            var_percentage,
            confidence_level: self.config.confidence_level,
            method: VarMethod::Historical,
            component_vars,
        })
    }

    /// Parametric VaR (variance-covariance method)
    fn calculate_parametric_var(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>],
        portfolio_value: Decimal,
    ) -> Result<VarResult> {
        // Calculate mean and std deviation of portfolio returns
        let portfolio_returns = self.calculate_portfolio_returns(positions, historical_returns)?;

        let mean = portfolio_returns.iter().sum::<Decimal>() 
            / Decimal::from(portfolio_returns.len());

        let variance = portfolio_returns.iter()
            .map(|r| {
                let diff = r - mean;
                diff * diff
            })
            .sum::<Decimal>() / Decimal::from(portfolio_returns.len());

        // Calculate standard deviation using f64 conversion
        let std_dev = {
            let var_f64 = variance.to_string().parse::<f64>().unwrap_or(0.0);
            Decimal::from_f64_retain(var_f64.sqrt()).unwrap_or(Decimal::ZERO)
        };

        // Z-score for confidence level
        let z_score = match self.config.confidence_level {
            cl if cl >= 0.99 => dec!(2.33),  // 99%
            cl if cl >= 0.95 => dec!(1.65),  // 95%
            cl if cl >= 0.90 => dec!(1.28),  // 90%
            _ => dec!(1.65),
        };

        let var_return = std_dev * z_score;
        let var_amount = var_return * portfolio_value;
        let var_percentage = var_return * dec!(100.0);

        let component_vars = self.calculate_component_vars(positions, historical_returns)?;

        Ok(VarResult {
            var_amount,
            var_percentage,
            confidence_level: self.config.confidence_level,
            method: VarMethod::Parametric,
            component_vars,
        })
    }

    /// Monte Carlo VaR
    fn calculate_monte_carlo_var(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>],
        portfolio_value: Decimal,
    ) -> Result<VarResult> {
        // Simplified Monte Carlo - in production would use proper random number generation
        // For now, use historical simulation as approximation
        self.calculate_historical_var(positions, historical_returns, portfolio_value)
    }

    /// Calculate portfolio returns from position returns
    fn calculate_portfolio_returns(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>],
    ) -> Result<Vec<Decimal>> {
        if positions.is_empty() {
            return Ok(vec![Decimal::ZERO]);
        }

        let num_periods = historical_returns.iter()
            .map(|r| r.len())
            .min()
            .unwrap_or(0);

        let mut portfolio_returns = Vec::with_capacity(num_periods);

        for period in 0..num_periods {
            let mut period_return = Decimal::ZERO;
            let total_value: Decimal = positions.iter()
                .map(|p| p.quantity.as_decimal() * p.current_price.as_decimal())
                .sum();

            for (pos_idx, position) in positions.iter().enumerate() {
                if let Some(returns) = historical_returns.get(pos_idx) {
                    if let Some(ret) = returns.get(period) {
                        let weight = if total_value > Decimal::ZERO {
                            (position.quantity.as_decimal() * position.current_price.as_decimal()) / total_value
                        } else {
                            Decimal::ZERO
                        };
                        period_return += ret * weight;
                    }
                }
            }

            portfolio_returns.push(period_return);
        }

        Ok(portfolio_returns)
    }

    /// Calculate component VaRs for each position
    fn calculate_component_vars(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>],
    ) -> Result<HashMap<String, Decimal>> {
        let mut component_vars = HashMap::new();

        for (idx, position) in positions.iter().enumerate() {
            if let Some(returns) = historical_returns.get(idx) {
                let mut sorted_returns = returns.clone();
                sorted_returns.sort();

                let index = ((1.0 - self.config.confidence_level) * sorted_returns.len() as f64) as usize;
                let var_return = sorted_returns.get(index).copied().unwrap_or(Decimal::ZERO);
                
                let position_value = position.quantity.as_decimal() * position.current_price.as_decimal();
                let component_var = var_return.abs() * position_value;

                component_vars.insert(
                    position.symbol.as_str().to_string(),
                    component_var,
                );
            }
        }

        Ok(component_vars)
    }

    /// Calculate Expected Shortfall (CVaR) - average loss beyond VaR
    pub fn calculate_expected_shortfall(
        &self,
        positions: &[Position],
        historical_returns: &[Vec<Decimal>],
        portfolio_value: Decimal,
    ) -> Result<Decimal> {
        let portfolio_returns = self.calculate_portfolio_returns(positions, historical_returns)?;

        let mut sorted_returns = portfolio_returns.clone();
        sorted_returns.sort();

        // Find VaR threshold
        let var_index = ((1.0 - self.config.confidence_level) * sorted_returns.len() as f64) as usize;

        // Calculate average of losses beyond VaR
        let tail_losses: Vec<Decimal> = sorted_returns.iter()
            .take(var_index)
            .copied()
            .collect();

        if tail_losses.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let avg_tail_loss = tail_losses.iter().sum::<Decimal>() / Decimal::from(tail_losses.len());
        let expected_shortfall = avg_tail_loss.abs() * portfolio_value;

        Ok(expected_shortfall)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_var_calculation() {
        let config = VarConfig::default();
        let calculator = VarCalculator::new(config);

        // Simplified test with mock data
        let positions = vec![];
        let historical_returns = vec![vec![
            dec!(-0.02), dec!(0.01), dec!(-0.01), dec!(0.03), dec!(-0.015)
        ]];
        let portfolio_value = dec!(100000.0);

        let result = calculator.calculate_var(
            &positions,
            &historical_returns,
            portfolio_value,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_parametric_var() {
        let config = VarConfig {
            method: VarMethod::Parametric,
            ..Default::default()
        };
        let calculator = VarCalculator::new(config);

        let positions = vec![];
        let historical_returns = vec![vec![
            dec!(-0.02), dec!(0.01), dec!(-0.01), dec!(0.03), dec!(-0.015)
        ]];
        let portfolio_value = dec!(100000.0);

        let result = calculator.calculate_var(
            &positions,
            &historical_returns,
            portfolio_value,
        );

        assert!(result.is_ok());
        if let Ok(var_result) = result {
            assert_eq!(var_result.method, VarMethod::Parametric);
        }
    }
}
