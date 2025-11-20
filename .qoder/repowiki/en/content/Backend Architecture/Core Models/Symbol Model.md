# Symbol Model

<cite>
**Referenced Files in This Document**
- [types.rs](file://crates/core/src/types.rs)
- [order.rs](file://crates/core/src/models/order.rs)
- [collector.rs](file://crates/data/src/collector.rs)
- [order_manager.rs](file://crates/trading/src/order_manager.rs)
- [error.rs](file://crates/core/src/error.rs)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs)
- [quality.rs](file://crates/data/src/quality.rs)
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
This document provides comprehensive data model documentation for the Symbol entity used to represent trading pairs in the BTC-USDT format. It explains the field definitions, data types, validation rules, and the newtype pattern used to ensure type safety. It also covers the serialization/deserialization implementation with serde, validation logic for format and character constraints, and practical usage patterns across market data collection and order execution. Business rules around symbol naming conventions and their importance in exchange integration are addressed, along with examples of how Symbol instances are created and used throughout the system.

## Project Structure
The Symbol model is defined in the core crate and is consumed by multiple subsystems:
- Core types define the Symbol newtype and validation rules
- Order model uses Symbol to identify trading pairs
- Market data collector uses Symbol to tag incoming market data
- Order manager logs and tracks orders using Symbol
- OKX client models expose instrument identifiers that are converted to Symbol
- Quality control validates market data against Symbol

```mermaid
graph TB
subgraph "Core"
T["types.rs<br/>Symbol, Price, Quantity"]
E["error.rs<br/>Error enum"]
end
subgraph "Models"
M["order.rs<br/>Order struct"]
end
subgraph "Data"
C["collector.rs<br/>MarketDataCollector"]
Q["quality.rs<br/>QualityControl"]
end
subgraph "Trading"
OM["order_manager.rs<br/>OrderManager"]
end
subgraph "OKX Client"
WS["websocket.rs<br/>Ticker/Candle/Trade models"]
end
T --> M
T --> C
T --> OM
T --> Q
WS --> C
E --> C
E --> OM
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)
- [collector.rs](file://crates/data/src/collector.rs#L1-L120)
- [order_manager.rs](file://crates/trading/src/order_manager.rs#L110-L140)
- [error.rs](file://crates/core/src/error.rs#L1-L47)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)
- [collector.rs](file://crates/data/src/collector.rs#L1-L120)
- [order_manager.rs](file://crates/trading/src/order_manager.rs#L110-L140)
- [error.rs](file://crates/core/src/error.rs#L1-L47)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)

## Core Components
- Symbol: A newtype wrapper around String representing trading pairs in BASE-QUOTE format. It enforces validation during construction and exposes convenience methods to access base and quote components. It derives serialization/deserialization support for safe transport across systems.
- Price and Quantity: Newtype wrappers for precise numeric values used in pricing and sizing, complementing Symbol in the order model.

Key characteristics:
- Type safety: Prevents misuse of raw strings as trading symbols
- Validation: Ensures correct format and non-empty parts
- Normalization: Converts to uppercase for consistent representation
- Interoperability: Derives serde traits for JSON serialization/deserialization

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)

## Architecture Overview
Symbol participates in two primary workflows:
- Market data ingestion: Incoming instrument identifiers from OKX are parsed into Symbol, validated, and tagged onto market data records
- Order lifecycle: Orders carry a Symbol to identify the trading pair; logging and state transitions reference the Symbol

```mermaid
sequenceDiagram
participant WS as "OKX WebSocket"
participant COL as "MarketDataCollector"
participant QC as "QualityControl"
participant DB as "Storage"
WS-->>COL : "Ticker/Candle/Trade payload"
COL->>COL : "Parse inst_id to Symbol"
COL->>QC : "validate_market_data(symbol, price, timestamp)"
QC-->>COL : "Result"
COL->>DB : "Store tick/candle"
COL-->>WS : "Continue streaming"
```

**Diagram sources**
- [collector.rs](file://crates/data/src/collector.rs#L198-L220)
- [collector.rs](file://crates/data/src/collector.rs#L222-L269)
- [collector.rs](file://crates/data/src/collector.rs#L272-L319)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)

## Detailed Component Analysis

### Symbol Data Model
Symbol is a newtype struct wrapping a String. It encapsulates:
- Construction with validation
- Accessors for base and quote parts
- Display and parsing from string

Implementation highlights:
- Validation rules enforced on creation:
  - Must contain exactly one hyphen separator
  - Base and quote parts must be non-empty
  - Normalized to uppercase for consistency
- Accessors:
  - base(): returns the base currency part
  - quote(): returns the quote currency part
  - as_str(): returns the normalized symbol string
- Parsing:
  - Implements FromStr to parse from string slices
- Serialization:
  - Derives Serialize/Deserialize for JSON interchange

```mermaid
classDiagram
class Symbol {
+new(input) Result~Symbol~
+base() &str
+quote() &str
+as_str() &str
}
class Price {
+new(value) Result~Price~
+as_decimal() Decimal
}
class Quantity {
+new(value) Result~Quantity~
+as_decimal() Decimal
+is_zero() bool
}
class Order {
+id : Uuid
+symbol : Symbol
+side : OrderSide
+order_type : OrderType
+quantity : Quantity
+price : Option~Price~
+status : OrderStatus
}
Order --> Symbol : "uses"
Order --> Price : "uses"
Order --> Quantity : "uses"
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)

### Validation Logic and Rules
Validation occurs during Symbol::new:
- Format: Must contain exactly one hyphen
- Parts: Both base and quote must be non-empty
- Normalization: Converted to uppercase

Errors are surfaced as InvalidSymbol variants in the Error enum.

```mermaid
flowchart TD
Start(["Symbol::new(input)"]) --> CheckHyphen["Contains exactly one '-'?"]
CheckHyphen --> |No| ErrFormat["Return InvalidSymbol"]
CheckHyphen --> |Yes| Split["Split by '-' into parts"]
Split --> CheckParts["Both parts non-empty?"]
CheckParts --> |No| ErrParts["Return InvalidSymbol"]
CheckParts --> Upper["Convert to uppercase"]
Upper --> Ok(["Return Symbol"])
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L27-L54)
- [error.rs](file://crates/core/src/error.rs#L1-L47)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L27-L54)
- [error.rs](file://crates/core/src/error.rs#L1-L47)

### Serialization and Deserialization with serde
Symbol derives Serialize and Deserialize, enabling safe transport across network boundaries and persistence layers. The OKX client models expose instrument identifiers as strings, which are parsed into Symbol during data ingestion.

Practical implications:
- JSON payloads containing inst_id can be mapped to Symbol
- Consistent uppercase representation prevents case mismatches
- Parsing failures surface as InvalidSymbol errors

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)
- [collector.rs](file://crates/data/src/collector.rs#L198-L220)
- [collector.rs](file://crates/data/src/collector.rs#L272-L319)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)

### Usage Patterns Across the System

#### Market Data Collection
- Collector subscribes to channels for configured symbols
- Incoming WebSocket events include instrument identifiers
- These identifiers are parsed into Symbol and validated before downstream processing
- Quality control validates price/timestamp and deduplicates messages using Symbol

```mermaid
sequenceDiagram
participant COL as "MarketDataCollector"
participant WS as "OKX WebSocket"
participant QC as "QualityControl"
COL->>WS : "Subscribe to channels for symbols"
WS-->>COL : "Ticker payload with inst_id"
COL->>COL : "Symbol : : new(inst_id)"
COL->>QC : "validate_market_data(symbol, price, timestamp)"
QC-->>COL : "Result"
COL-->>COL : "Store tick/candle"
```

**Diagram sources**
- [collector.rs](file://crates/data/src/collector.rs#L90-L110)
- [collector.rs](file://crates/data/src/collector.rs#L198-L220)
- [collector.rs](file://crates/data/src/collector.rs#L222-L269)
- [collector.rs](file://crates/data/src/collector.rs#L272-L319)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)

**Section sources**
- [collector.rs](file://crates/data/src/collector.rs#L90-L110)
- [collector.rs](file://crates/data/src/collector.rs#L198-L220)
- [collector.rs](file://crates/data/src/collector.rs#L222-L269)
- [collector.rs](file://crates/data/src/collector.rs#L272-L319)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)

#### Order Execution
- Orders carry a Symbol to identify the trading pair
- Logging and state machines reference Symbol for context
- Order manager emits events carrying Symbol for visibility

```mermaid
sequenceDiagram
participant OM as "OrderManager"
participant ORD as "Order"
participant EX as "Exchange"
OM->>OM : "submit_order(order)"
OM->>OM : "log side + symbol + price"
OM->>ORD : "store managed order"
OM->>EX : "submit to exchange"
EX-->>OM : "acknowledge"
OM->>OM : "emit OrderSubmitted(symbol)"
```

**Diagram sources**
- [order_manager.rs](file://crates/trading/src/order_manager.rs#L110-L140)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)

**Section sources**
- [order_manager.rs](file://crates/trading/src/order_manager.rs#L110-L140)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)

### Business Rules and Exchange Integration
- Naming convention: BASE-QUOTE format with uppercase base and quote components
- Exchange integration: OKX instruments expose identifiers that are parsed into Symbol; consistent uppercase ensures alignment with exchange conventions
- Operational impact:
  - Prevents runtime ambiguity caused by mixed-case or malformed identifiers
  - Enables reliable routing of market data and orders to the correct trading pair
  - Facilitates deterministic storage and querying by symbol

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L27-L54)
- [collector.rs](file://crates/data/src/collector.rs#L198-L220)
- [collector.rs](file://crates/data/src/collector.rs#L272-L319)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)

## Dependency Analysis
Symbol is consumed by:
- Order model: central to order identity and reporting
- MarketDataCollector: to tag and route market data
- OrderManager: for logging and event emission
- QualityControl: to validate and deduplicate market data keyed by symbol

```mermaid
graph LR
T["types.rs: Symbol"] --> O["order.rs: Order"]
T --> COL["collector.rs: MarketDataCollector"]
T --> OM["order_manager.rs: OrderManager"]
T --> QC["quality.rs: QualityControl"]
WS["websocket.rs: Ticker/Candle/Trade"] --> COL
E["error.rs: Error"] --> COL
E --> OM
```

**Diagram sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)
- [collector.rs](file://crates/data/src/collector.rs#L1-L120)
- [order_manager.rs](file://crates/trading/src/order_manager.rs#L110-L140)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)
- [error.rs](file://crates/core/src/error.rs#L1-L47)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L1-L120)
- [order.rs](file://crates/core/src/models/order.rs#L90-L150)
- [collector.rs](file://crates/data/src/collector.rs#L1-L120)
- [order_manager.rs](file://crates/trading/src/order_manager.rs#L110-L140)
- [quality.rs](file://crates/data/src/quality.rs#L238-L296)
- [websocket.rs](file://crates/okx-client/src/models/websocket.rs#L267-L319)
- [error.rs](file://crates/core/src/error.rs#L1-L47)

## Performance Considerations
- Parsing overhead: Symbol::new performs a single split operation and basic checks; negligible compared to network and database operations
- Memory: Symbol wraps a String; cloning is inexpensive for short identifiers
- Serialization: serde derives minimize boilerplate and reduce risk of misconfiguration
- Recommendations:
  - Prefer constructing Symbol once per message and reuse
  - Avoid repeated parsing of the same instrument identifier
  - Normalize identifiers early to prevent redundant conversions

[No sources needed since this section provides general guidance]

## Troubleshooting Guide
Common issues and resolutions:
- Invalid symbol format:
  - Cause: Missing or multiple separators, empty parts
  - Resolution: Ensure BASE-QUOTE format with non-empty base and quote
  - Reference: [types.rs](file://crates/core/src/types.rs#L27-L54), [error.rs](file://crates/core/src/error.rs#L1-L47)
- Parsing errors in market data:
  - Cause: Unexpected instrument identifier format from exchange
  - Resolution: Validate inst_id before Symbol::new; handle InvalidSymbol gracefully
  - Reference: [collector.rs](file://crates/data/src/collector.rs#L198-L220), [collector.rs](file://crates/data/src/collector.rs#L272-L319)
- Serialization/deserialization failures:
  - Cause: Malformed JSON payloads
  - Resolution: Ensure serde derives are present and payload conforms to expected shape
  - Reference: [types.rs](file://crates/core/src/types.rs#L1-L120), [error.rs](file://crates/core/src/error.rs#L1-L47)

**Section sources**
- [types.rs](file://crates/core/src/types.rs#L27-L54)
- [error.rs](file://crates/core/src/error.rs#L1-L47)
- [collector.rs](file://crates/data/src/collector.rs#L198-L220)
- [collector.rs](file://crates/data/src/collector.rs#L272-L319)

## Conclusion
Symbol is a foundational type that enforces correct trading pair representation and prevents invalid usage across the system. Its newtype pattern, validation rules, and serde support ensure robustness in market data ingestion and order execution. By adhering to the BASE-QUOTE naming convention and leveraging Symbol consistently, the system achieves predictable behavior, improved reliability, and clearer operational semantics.