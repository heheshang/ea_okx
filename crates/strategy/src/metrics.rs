//! Performance metrics calculation

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub total_pnl: Decimal,
    pub total_commission: Decimal,
    pub net_pnl: Decimal,
    pub win_rate: f64,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub profit_factor: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: f64,
    pub total_volume: Decimal,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_pnl: Decimal::ZERO,
            total_commission: Decimal::ZERO,
            net_pnl: Decimal::ZERO,
            win_rate: 0.0,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            profit_factor: 0.0,
            sharpe_ratio: None,
            max_drawdown: 0.0,
            total_volume: Decimal::ZERO,
        }
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_win_rate(&mut self) {
        if self.total_trades > 0 {
            self.win_rate = self.winning_trades as f64 / self.total_trades as f64;
        }
    }

    pub fn calculate_profit_factor(&mut self) {
        let total_wins: f64 = (self.avg_win * Decimal::new(self.winning_trades as i64, 0))
            .to_string()
            .parse()
            .unwrap_or(0.0);
        let total_losses: f64 = (self.avg_loss * Decimal::new(self.losing_trades as i64, 0))
            .to_string()
            .parse()
            .unwrap_or(0.0);

        if total_losses > 0.0 {
            self.profit_factor = total_wins / total_losses;
        }
    }
}
