# Price Model

<cite>
**Referenced Files in This Document**
- [types.rs](file://crates/core/src/types.rs)
- [order.rs](file://crates/core/src/models/order.rs)
- [position.rs](file://crates/core/src/models/position.rs)
- [trade.rs](file://crates/core/src/models/trade.rs)
- [error.rs](file://crates/core/src/error.rs)
- [Cargo.toml](file://crates/core/Cargo.toml)
- [simple_ma_crossover.rs](file://examples/simple_ma_crossover.rs)
- [grid_trading.rs](file://examples/grid_trading.rs)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Project Structure](#project-structure)
3. [Core Components](#core-components)
4. [Architecture Overview](#architecture-overview)
5. [Detailed Component Analysis](#detailed-component-analysis)
6. [Dependency Analysis](#dependency-analysis)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting Guide](#troubleshooting-guide)
9. [Conclusion](#conclusion)

## Introduction
This document provides comprehensive data model documentation for the Price entity used throughout the trading system. It explains the field definitions, data types, validation rules, and the implementation of the newtype pattern using rust_decimal::Decimal to ensure type safety and precision in financial calculations. It also covers validation logic for positive values, serialization/deserialization with serde, and practical usage in order pricing, position valuation, and PnL calculations. Finally, it addresses business rules around price precision in cryptocurrency trading and the importance of maintaining 8-decimal accuracy.

## Project Structure
The Price model is defined in the core crate and is consumed by multiple domain models:
- Price definition and validation live in the core types module.
- Order, Position, and Trade models use Price for pricing.
- Example strategies demonstrate real-world usage patterns.

```mermaid
graph TB
Types["crates/core/src/types.rs<br/>Defines Price, Quantity, Decimal"] --> OrderModel["crates/core/src/models/order.rs<br/>Order uses Price"]
Types --> PositionModel["crates/core/src/models/position.rs<br/>Position uses Price"]
Types --> TradeModel["crates/core/src/models/trade.rs<br/>Trade uses Price"]
Examples["examples/*.rs<br/>Usage examples"] --> OrderModel
Examples --> PositionModel
Examples --> TradeModel
Cargo["crates/core/Cargo.toml<br/>Dependencies: serde, rust_decimal, rust_decimal_macros"] --> Types
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L86-L123)
- [order.rs](file://crates/core/src/models/order.rs#L95-L151)
- [position.rs](file://crates/core/src/models/position.rs#L32-L76)
- [trade.rs](file://crates/core/src/models/trade.rs#L10-L57)
- [Cargo.toml](file://crates/core/Cargo.toml#L8-L17)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L86-L123)
- [order.rs](file://crates/core/src/models/order.rs#L95-L151)
- [position.rs](file://crates/core/src/models/position.rs#L32-L76)
- [trade.rs](file://crates/core/src/models/trade.rs#L10-L57)
- [Cargo.toml](file://crates/core/Cargo.toml#L8-L17)

## Core Components
- Price: A newtype wrapper around rust_decimal::Decimal with strict positivity validation and serde support.
- Decimal: A type alias for rust_decimal::Decimal to centralize decimal usage across the system.
- Validation rules:
  - Price must be strictly greater than zero.
  - Quantity can be zero or positive.
  - Negative values are rejected for both types.

Key implementation highlights:
- Price::new validates positivity and returns a typed Price instance.
- Price::from_f64 enables controlled construction from floating-point values for testing.
- Price implements Display and derives serde Serialize/Deserialize for JSON interchange.
- Tests confirm ordering, invalid inputs, and conversions.

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L86-L123)
- [types.rs](file://crates/core/src/types.rs#L181-L291)
- [error.rs](file://crates/core/src/error.rs#L11-L16)

## Architecture Overview
Price participates in three primary domains:
- Orders: Limit price and average fill price are stored as Price.
- Positions: Average entry price and current price are stored as Price.
- Trades: Execution price is stored as Price.

```mermaid
classDiagram
class Price {
+new(value) Result~Price~
+as_decimal() Decimal
+from_f64(value) Result~Price~
+Display
+Serialize
+Deserialize
}
class Order {
+price : Option~Price~
+avg_fill_price : Option~Price~
+quantity : Quantity
+side : OrderSide
+order_type : OrderType
}
class Position {
+avg_entry_price : Price
+current_price : Price
+quantity : Quantity
+unrealized_pnl : Decimal
}
class Trade {
+price : Price
+quantity : Quantity
+commission : Decimal
+realized_pnl : Option~Decimal~
}
Order --> Price : "uses"
Position --> Price : "uses"
Trade --> Price : "uses"
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L86-L123)
- [order.rs](file://crates/core/src/models/order.rs#L118-L126)
- [position.rs](file://crates/core/src/models/position.rs#L47-L54)
- [trade.rs](file://crates/core/src/models/trade.rs#L33-L37)

## Detailed Component Analysis

### Price Definition and Newtype Pattern
- Newtype pattern: Price wraps rust_decimal::Decimal to prevent accidental mixing with other numeric types.
- Serde integration: Price derives Serialize and Deserialize, enabling JSON serialization without custom logic.
- Validation: Positive-only constraint enforced in Price::new.
- Precision: Uses rust_decimal::Decimal for exact decimal arithmetic and string representation.

```mermaid
flowchart TD
Start(["Construct Price"]) --> Validate["Validate value > 0"]
Validate --> Valid{"Valid?"}
Valid --> |No| Err["Return InvalidPrice error"]
Valid --> |Yes| Wrap["Wrap into Price(newtype)"]
Wrap --> Done(["Return Ok(Price)"])
Err --> End(["Exit"])
Done --> End
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L102-L110)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L86-L123)
- [error.rs](file://crates/core/src/error.rs#L11-L16)

### Usage in Orders
- Limit price: Optional Price used for limit orders.
- Average fill price: Tracks the weighted average execution price after fills.
- Lifecycle updates: Order.update_fill sets avg_fill_price and transitions status accordingly.

```mermaid
sequenceDiagram
participant Strategy as "Strategy"
participant Order as "Order"
participant Engine as "Execution Engine"
Strategy->>Order : "Create order with price=None (market)"
Strategy->>Order : "Create order with price=Some(Price) (limit)"
Engine->>Order : "update_fill(filled_qty, avg_price)"
Order->>Order : "Set avg_fill_price = avg_price"
Order->>Order : "Transition status to Partial/Filled"
```

**Diagram sources**
- [order.rs](file://crates/core/src/models/order.rs#L118-L126)
- [order.rs](file://crates/core/src/models/order.rs#L243-L264)

**Section sources**
- [order.rs](file://crates/core/src/models/order.rs#L95-L151)
- [order.rs](file://crates/core/src/models/order.rs#L243-L264)

### Usage in Positions
- Average entry price: The Price at which a position was opened.
- Current price: The latest market Price used to compute unrealized PnL.
- Unrealized PnL calculation: Depends on side (Long/Short/Net), price difference, and quantity.

```mermaid
flowchart TD
Enter(["Position update_price(current_price)"]) --> ComputeDiff["Compute price_diff = current_price - avg_entry_price"]
ComputeDiff --> SideCheck{"Side"}
SideCheck --> |Long| LongCalc["unrealized_pnl = price_diff * quantity"]
SideCheck --> |Short| ShortCalc["unrealized_pnl = -price_diff * quantity"]
SideCheck --> |Net| NetCalc["unrealized_pnl = price_diff * quantity"]
LongCalc --> Save["Save updated unrealized_pnl"]
ShortCalc --> Save
NetCalc --> Save
Save --> Exit(["Done"])
```

**Diagram sources**
- [position.rs](file://crates/core/src/models/position.rs#L107-L124)

**Section sources**
- [position.rs](file://crates/core/src/models/position.rs#L32-L76)
- [position.rs](file://crates/core/src/models/position.rs#L107-L124)

### Usage in Trades
- Execution price: The Price at which a trade was executed.
- Trade value and net value: Computed from quantity and price, adjusted by commission depending on side.
- Effective price: Net value divided by quantity.

```mermaid
flowchart TD
Start(["Trade created"]) --> Value["trade_value = quantity * price"]
Value --> Net{"Side"}
Net --> |Buy| BuyNet["net_value = gross + commission"]
Net --> |Sell| SellNet["net_value = gross - commission"]
BuyNet --> Eff["effective_price = net_value / quantity"]
SellNet --> Eff
Eff --> End(["Done"])
```

**Diagram sources**
- [trade.rs](file://crates/core/src/models/trade.rs#L89-L106)

**Section sources**
- [trade.rs](file://crates/core/src/models/trade.rs#L10-L57)
- [trade.rs](file://crates/core/src/models/trade.rs#L89-L106)

### Business Rules Around Price Precision
- Cryptocurrency markets often require high precision (e.g., 8 decimals) to avoid rounding errors and ensure fair execution.
- The rust_decimal::Decimal type preserves exact decimal representation and avoids floating-point rounding issues.
- The Price model enforces positivity to prevent invalid market states and ensures downstream calculations remain meaningful.
- Serialization/deserialization with serde preserves exact decimal values, preventing precision loss during transport.

**Section sources**
- [Cargo.toml](file://crates/core/Cargo.toml#L8-L17)
- [types.rs](file://crates/core/src/types.rs#L86-L123)

### Examples of Price Usage
- Moving Average Crossover Strategy:
  - Uses Price for entry/exit conditions and computes stop-loss/take-profit targets as Price values.
  - Computes unrealized PnL using Price differences and position quantities.
- Grid Trading Strategy:
  - Generates grid levels by stepping between lower and upper bound Prices.
  - Places limit orders at grid Prices.

```mermaid
sequenceDiagram
participant Strategy as "MACrossoverStrategy"
participant Order as "Order"
participant Position as "Position"
Strategy->>Strategy : "on_price(Price)"
Strategy->>Order : "create_order(signal, current_price)"
Strategy->>Position : "on_order_filled(order, fill_price)"
Strategy->>Strategy : "check_exit_conditions(current_price)"
Strategy->>Strategy : "unrealized_pnl(current_price)"
```

**Diagram sources**
- [simple_ma_crossover.rs](file://examples/simple_ma_crossover.rs#L142-L279)
- [grid_trading.rs](file://examples/grid_trading.rs#L55-L115)

**Section sources**
- [simple_ma_crossover.rs](file://examples/simple_ma_crossover.rs#L142-L279)
- [grid_trading.rs](file://examples/grid_trading.rs#L55-L115)

## Dependency Analysis
- Dependencies:
  - serde and serde_json for serialization/deserialization.
  - rust_decimal and rust_decimal_macros for precise decimal arithmetic and macro support.
  - uuid and chrono for identifiers and timestamps.
- Coupling:
  - Price is widely used across Order, Position, and Trade, increasing cohesion within the financial domain.
  - No circular dependencies detected among core modules.

```mermaid
graph TB
Serde["serde / serde_json"] --> Types["types.rs"]
Decimal["rust_decimal / macros"] --> Types
Uuid["uuid"] --> Order["order.rs"]
Chrono["chrono"] --> Order
Chrono --> Position["position.rs"]
Chrono --> Trade["trade.rs"]
```

**Diagram sources**
- [Cargo.toml](file://crates/core/Cargo.toml#L8-L17)
- [order.rs](file://crates/core/src/models/order.rs#L1-L15)
- [position.rs](file://crates/core/src/models/position.rs#L1-L10)
- [trade.rs](file://crates/core/src/models/trade.rs#L1-L9)

**Section sources**
- [Cargo.toml](file://crates/core/Cargo.toml#L8-L17)

## Performance Considerations
- Using rust_decimal::Decimal ensures exact decimal arithmetic and avoids floating-point errors, which is critical for financial computations.
- The newtype pattern adds negligible overhead while providing strong type safety.
- Serde’s default Decimal serialization preserves precision without requiring custom logic.

[No sources needed since this section provides general guidance]

## Troubleshooting Guide
Common issues and resolutions:
- InvalidPrice errors occur when constructing Price with zero or negative values. Ensure all prices are strictly positive.
- DecimalError occurs when converting invalid f64 values to Decimal. Prefer constructing from Decimal literals or strings.
- Serialization errors indicate malformed JSON. Verify that Price values are serialized as numeric strings or numbers compatible with rust_decimal.

**Section sources**
- [error.rs](file://crates/core/src/error.rs#L11-L16)
- [error.rs](file://crates/core/src/error.rs#L29-L33)

## Conclusion
The Price model is a robust, type-safe representation of financial prices with 8-decimal precision. Its newtype design, strict validation, and serde integration ensure correctness and reliability across order pricing, position valuation, and PnL calculations. The examples demonstrate practical usage in trading strategies, reinforcing the importance of precision and positivity in cryptocurrency markets.