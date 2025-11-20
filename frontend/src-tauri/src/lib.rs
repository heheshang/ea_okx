mod commands;

use commands::{
    strategy::*,
    trading::*,
    data::*,
    risk::*,
    system::*,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Strategy commands
      get_strategies,
      get_strategy,
      create_strategy,
      update_strategy,
      delete_strategy,
      start_strategy,
      stop_strategy,
      pause_strategy,
      get_strategy_metrics,
      // Trading commands
      place_order,
      cancel_order,
      get_open_orders,
      get_order_history,
      get_positions,
      close_position,
      // Data commands
      subscribe_market_data,
      get_latest_price,
      get_candles,
      // Risk commands
      get_risk_limits,
      update_risk_limits,
      calculate_var,
      // System commands
      get_system_metrics,
      get_alerts,
      run_backtest,
      get_backtest_results,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
