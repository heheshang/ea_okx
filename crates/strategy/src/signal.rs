//! Trading signal types

use ea_okx_core::types::{Price, Quantity};
use serde::{Deserialize, Serialize};

/// Signal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    CloseLong,
    CloseShort,
}

/// Trading signal with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_type: SignalType,
    pub confidence: f64,
    pub target_price: Option<Price>,
    pub stop_loss: Option<Price>,
    pub take_profit: Option<Price>,
    pub suggested_quantity: Option<Quantity>,
    pub metadata: serde_json::Value,
}

impl Signal {
    pub fn buy(confidence: f64) -> Self {
        Self {
            signal_type: SignalType::Buy,
            confidence,
            target_price: None,
            stop_loss: None,
            take_profit: None,
            suggested_quantity: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn sell(confidence: f64) -> Self {
        Self {
            signal_type: SignalType::Sell,
            confidence,
            target_price: None,
            stop_loss: None,
            take_profit: None,
            suggested_quantity: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn hold() -> Self {
        Self {
            signal_type: SignalType::Hold,
            confidence: 1.0,
            target_price: None,
            stop_loss: None,
            take_profit: None,
            suggested_quantity: None,
            metadata: serde_json::json!({}),
        }
    }
}
