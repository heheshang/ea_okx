use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size: f64,
    pub max_leverage: f64,
    pub daily_loss_limit: f64,
    pub max_concentration: f64,
    pub min_margin_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaRResult {
    pub var_95: f64,
    pub var_99: f64,
    pub cvar: f64,
    pub method: String, // Historical, Parametric, MonteCarlo
}

/// Get current risk limits
#[tauri::command]
pub async fn get_risk_limits() -> Result<RiskLimits, String> {
    log::info!("Fetching risk limits");
    Ok(RiskLimits {
        max_position_size: 100000.0,
        max_leverage: 3.0,
        daily_loss_limit: 5000.0,
        max_concentration: 0.25,
        min_margin_ratio: 0.15,
    })
}

/// Update risk limits
#[tauri::command]
pub async fn update_risk_limits(limits: RiskLimits) -> Result<(), String> {
    log::info!("Updating risk limits: {:?}", limits);
    // TODO: Integrate with risk service
    Ok(())
}

/// Calculate VaR
#[tauri::command]
pub async fn calculate_var(confidence: f64, method: String) -> Result<VaRResult, String> {
    log::info!("Calculating VaR (confidence: {}, method: {})", confidence, method);
    // TODO: Integrate with risk service
    Ok(VaRResult {
        var_95: 2500.0,
        var_99: 4000.0,
        cvar: 5500.0,
        method,
    })
}
