mod commands;
mod services;
mod state;

use state::AppState;
use tauri::Manager;
use commands::{
    strategy::*,
    trading::*,
    data::*,
    risk::*,
    system::*,
    websocket::*,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let app_state = AppState::new();

  tauri::Builder::default()
    .manage(app_state)
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize application state
      let state: tauri::State<AppState> = app.state();
      let state_clone = state.inner().clone();
      tauri::async_runtime::spawn(async move {
          if let Err(e) = state_clone.initialize().await {
              log::error!("Failed to initialize app state: {}", e);
          }
      });

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
      duplicate_strategy,
      // Trading commands
      place_order,
      cancel_order,
      cancel_all_orders,
      get_open_orders,
      get_order_history,
      get_positions,
      close_position,
      get_trades,
      submit_execution_signal,
      get_strategy_execution_stats,
      get_account_balance,
      get_trading_fees,
      get_order_book,
      get_24h_stats,
      get_position_risk,
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
      // WebSocket commands
      subscribe_strategy_updates,
      unsubscribe_strategy_updates,
      get_connected_clients_count,
      get_realtime_strategy_stats,
      simulate_strategy_signal,
      simulate_trade_execution,
      simulate_strategy_error,
      simulate_position_update,
      update_strategy_metrics,
      get_websocket_status,
      get_market_data_status,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
