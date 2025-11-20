pub mod cost_model;
pub mod engine;
pub mod error;
pub mod events;
pub mod portfolio;
pub mod results;

pub use cost_model::{CommissionModel, CostModel, SlippageModel};
pub use engine::{
    BacktestConfig, BacktestEngine, HistoricalDataSource, MockDataSource, PositionSizing,
};
pub use error::{Error, Result};
pub use events::{ExecutionEvent, Fill, MarketEvent, Trade};
pub use portfolio::Portfolio;
pub use results::BacktestResult;
