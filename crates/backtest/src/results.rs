use crate::error::Result;
use crate::events::Trade;
use crate::portfolio::Portfolio;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Complete backtest results with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// Start and end times
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    
    /// Capital metrics
    pub initial_capital: Decimal,
    pub final_equity: Decimal,
    pub total_pnl: Decimal,
    pub total_return_pct: Decimal,
    
    /// Trade statistics
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: Decimal,
    
    /// P&L metrics
    pub gross_profit: Decimal,
    pub gross_loss: Decimal,
    pub profit_factor: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    
    /// Risk metrics
    pub max_drawdown: Decimal,
    pub max_drawdown_pct: Decimal,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub calmar_ratio: Decimal,
    
    /// Cost analysis
    pub total_commission: Decimal,
    pub total_slippage: Decimal,
    pub total_costs: Decimal,
    
    /// Time metrics
    pub avg_trade_duration_hours: Decimal,
    pub max_trade_duration_hours: Decimal,
    pub min_trade_duration_hours: Decimal,
    
    /// Equity curve
    pub equity_curve: Vec<(DateTime<Utc>, Decimal)>,
    
    /// Drawdown curve
    pub drawdown_curve: Vec<(DateTime<Utc>, Decimal)>,
}

impl BacktestResult {
    pub fn from_portfolio_and_trades(
        portfolio: &Portfolio,
        trades: &[Trade],
        initial_capital: Decimal,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Self> {
        let final_equity = portfolio.total_equity();
        let total_pnl = final_equity - initial_capital;
        let total_return_pct = if initial_capital > Decimal::ZERO {
            total_pnl / initial_capital
        } else {
            Decimal::ZERO
        };
        
        // Calculate trade statistics
        let total_trades = trades.len();
        let winning_trades = trades.iter().filter(|t| t.pnl > Decimal::ZERO).count();
        let losing_trades = trades.iter().filter(|t| t.pnl < Decimal::ZERO).count();
        let win_rate = if total_trades > 0 {
            Decimal::from(winning_trades) / Decimal::from(total_trades)
        } else {
            Decimal::ZERO
        };
        
        // Calculate P&L metrics
        let gross_profit: Decimal = trades.iter()
            .filter(|t| t.pnl > Decimal::ZERO)
            .map(|t| t.pnl)
            .sum();
        
        let gross_loss: Decimal = trades.iter()
            .filter(|t| t.pnl < Decimal::ZERO)
            .map(|t| t.pnl.abs())
            .sum();
        
        let profit_factor = if gross_loss > Decimal::ZERO {
            gross_profit / gross_loss
        } else if gross_profit > Decimal::ZERO {
            Decimal::MAX
        } else {
            Decimal::ZERO
        };
        
        let average_win = if winning_trades > 0 {
            gross_profit / Decimal::from(winning_trades)
        } else {
            Decimal::ZERO
        };
        
        let average_loss = if losing_trades > 0 {
            gross_loss / Decimal::from(losing_trades)
        } else {
            Decimal::ZERO
        };
        
        let largest_win = trades.iter()
            .map(|t| t.pnl)
            .max()
            .unwrap_or(Decimal::ZERO);
        
        let largest_loss = trades.iter()
            .map(|t| t.pnl)
            .min()
            .unwrap_or(Decimal::ZERO);
        
        // Calculate drawdown
        let (max_drawdown, max_drawdown_pct, drawdown_curve) = 
            Self::calculate_drawdown(&portfolio.equity_curve);
        
        // Calculate risk metrics
        let sharpe_ratio = Self::calculate_sharpe_ratio(&portfolio.equity_curve);
        let sortino_ratio = Self::calculate_sortino_ratio(&portfolio.equity_curve);
        
        let calmar_ratio = if max_drawdown_pct.abs() > dec!(0.0001) {
            total_return_pct / max_drawdown_pct.abs()
        } else {
            Decimal::ZERO
        };
        
        // Calculate trade duration metrics
        let durations: Vec<Decimal> = trades.iter()
            .filter_map(|t| t.duration())
            .map(|d| Decimal::from(d.num_hours()))
            .collect();
        
        let avg_trade_duration_hours = if !durations.is_empty() {
            durations.iter().sum::<Decimal>() / Decimal::from(durations.len())
        } else {
            Decimal::ZERO
        };
        
        let max_trade_duration_hours = durations.iter()
            .copied()
            .max()
            .unwrap_or(Decimal::ZERO);
        
        let min_trade_duration_hours = durations.iter()
            .copied()
            .min()
            .unwrap_or(Decimal::ZERO);
        
        Ok(Self {
            start_time,
            end_time,
            initial_capital,
            final_equity,
            total_pnl,
            total_return_pct,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            gross_profit,
            gross_loss,
            profit_factor,
            average_win,
            average_loss,
            largest_win,
            largest_loss,
            max_drawdown,
            max_drawdown_pct,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            total_commission: portfolio.total_commission,
            total_slippage: portfolio.total_slippage,
            total_costs: portfolio.total_commission + portfolio.total_slippage,
            avg_trade_duration_hours,
            max_trade_duration_hours,
            min_trade_duration_hours,
            equity_curve: portfolio.equity_curve.clone(),
            drawdown_curve,
        })
    }

    /// Calculate maximum drawdown
    fn calculate_drawdown(
        equity_curve: &[(DateTime<Utc>, Decimal)]
    ) -> (Decimal, Decimal, Vec<(DateTime<Utc>, Decimal)>) {
        let mut peak = Decimal::ZERO;
        let mut max_dd = Decimal::ZERO;
        let mut max_dd_pct = Decimal::ZERO;
        let mut dd_curve = Vec::new();
        
        for (timestamp, equity) in equity_curve {
            if *equity > peak {
                peak = *equity;
            }
            
            let dd = peak - equity;
            let dd_pct = if peak > Decimal::ZERO {
                dd / peak
            } else {
                Decimal::ZERO
            };
            
            if dd > max_dd {
                max_dd = dd;
                max_dd_pct = dd_pct;
            }
            
            dd_curve.push((*timestamp, dd_pct));
        }
        
        (max_dd, max_dd_pct, dd_curve)
    }

    /// Calculate Sharpe ratio (annualized)
    fn calculate_sharpe_ratio(equity_curve: &[(DateTime<Utc>, Decimal)]) -> Decimal {
        if equity_curve.len() < 2 {
            return Decimal::ZERO;
        }
        
        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..equity_curve.len() {
            let prev_equity = equity_curve[i - 1].1;
            let curr_equity = equity_curve[i].1;
            
            if prev_equity > Decimal::ZERO {
                let ret = (curr_equity - prev_equity) / prev_equity;
                returns.push(ret);
            }
        }
        
        if returns.is_empty() {
            return Decimal::ZERO;
        }
        
        // Calculate mean return
        let mean: Decimal = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());
        
        // Calculate standard deviation
        let variance: Decimal = returns.iter()
            .map(|r| {
                let diff = r - mean;
                diff * diff
            })
            .sum::<Decimal>() / Decimal::from(returns.len());
        
        let std_dev = if variance > Decimal::ZERO {
            let variance_f64 = variance.to_string().parse::<f64>().unwrap_or(0.0);
            Decimal::from_f64_retain(variance_f64.sqrt()).unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };
        
        if std_dev > Decimal::ZERO {
            // Assume 252 trading days per year
            let annualized_return = mean * dec!(252.0);
            let sqrt_252 = Decimal::from_f64_retain(252.0_f64.sqrt()).unwrap_or(Decimal::ONE);
            let annualized_std = std_dev * sqrt_252;
            
            annualized_return / annualized_std
        } else {
            Decimal::ZERO
        }
    }

    /// Calculate Sortino ratio (annualized, using downside deviation)
    fn calculate_sortino_ratio(equity_curve: &[(DateTime<Utc>, Decimal)]) -> Decimal {
        if equity_curve.len() < 2 {
            return Decimal::ZERO;
        }
        
        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..equity_curve.len() {
            let prev_equity = equity_curve[i - 1].1;
            let curr_equity = equity_curve[i].1;
            
            if prev_equity > Decimal::ZERO {
                let ret = (curr_equity - prev_equity) / prev_equity;
                returns.push(ret);
            }
        }
        
        if returns.is_empty() {
            return Decimal::ZERO;
        }
        
        // Calculate mean return
        let mean: Decimal = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());
        
        // Calculate downside deviation (only negative returns)
        let downside_returns: Vec<Decimal> = returns.iter()
            .filter(|r| **r < Decimal::ZERO)
            .copied()
            .collect();
        
        if downside_returns.is_empty() {
            return Decimal::MAX;
        }
        
        let downside_variance: Decimal = downside_returns.iter()
            .map(|r| r * r)
            .sum::<Decimal>() / Decimal::from(downside_returns.len());
        
        let downside_dev = if downside_variance > Decimal::ZERO {
            let variance_f64 = downside_variance.to_string().parse::<f64>().unwrap_or(0.0);
            Decimal::from_f64_retain(variance_f64.sqrt()).unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };
        
        if downside_dev > Decimal::ZERO {
            // Assume 252 trading days per year
            let annualized_return = mean * dec!(252.0);
            let sqrt_252 = Decimal::from_f64_retain(252.0_f64.sqrt()).unwrap_or(Decimal::ONE);
            let annualized_dd = downside_dev * sqrt_252;
            
            annualized_return / annualized_dd
        } else {
            Decimal::ZERO
        }
    }

    /// Generate a summary report
    pub fn summary(&self) -> String {
        format!(
            r#"
=== Backtest Results ===

Period: {} to {}
Duration: {} days

Capital:
  Initial: ${:.2}
  Final: ${:.2}
  Total P&L: ${:.2}
  Return: {:.2}%

Trades:
  Total: {}
  Winners: {} ({:.2}%)
  Losers: {} ({:.2}%)
  Win Rate: {:.2}%

P&L Analysis:
  Gross Profit: ${:.2}
  Gross Loss: ${:.2}
  Profit Factor: {:.2}
  Average Win: ${:.2}
  Average Loss: ${:.2}
  Largest Win: ${:.2}
  Largest Loss: ${:.2}

Risk Metrics:
  Max Drawdown: ${:.2} ({:.2}%)
  Sharpe Ratio: {:.2}
  Sortino Ratio: {:.2}
  Calmar Ratio: {:.2}

Costs:
  Commission: ${:.2}
  Slippage: ${:.2}
  Total Costs: ${:.2}

Trade Duration:
  Average: {:.2} hours
  Max: {:.2} hours
  Min: {:.2} hours
"#,
            self.start_time.format("%Y-%m-%d"),
            self.end_time.format("%Y-%m-%d"),
            (self.end_time - self.start_time).num_days(),
            self.initial_capital,
            self.final_equity,
            self.total_pnl,
            self.total_return_pct * dec!(100.0),
            self.total_trades,
            self.winning_trades,
            (self.win_rate * dec!(100.0)),
            self.losing_trades,
            ((Decimal::ONE - self.win_rate) * dec!(100.0)),
            self.win_rate * dec!(100.0),
            self.gross_profit,
            self.gross_loss,
            self.profit_factor,
            self.average_win,
            self.average_loss,
            self.largest_win,
            self.largest_loss,
            self.max_drawdown,
            self.max_drawdown_pct * dec!(100.0),
            self.sharpe_ratio,
            self.sortino_ratio,
            self.calmar_ratio,
            self.total_commission,
            self.total_slippage,
            self.total_costs,
            self.avg_trade_duration_hours,
            self.max_trade_duration_hours,
            self.min_trade_duration_hours,
        )
    }
}
