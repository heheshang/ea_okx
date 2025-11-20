# REST API Client

<cite>
**Referenced Files in This Document**   
- [lib.rs](file://crates/okx-client/src/lib.rs)
- [rest.rs](file://crates/okx-client/src/rest.rs)
- [auth.rs](file://crates/okx-client/src/auth.rs)
- [error.rs](file://crates/okx-client/src/error.rs)
- [models/request.rs](file://crates/okx-client/src/models/request.rs)
- [models/response.rs](file://crates/okx-client/src/models/response.rs)
- [websocket.rs](file://crates/okx-client/src/websocket.rs)
- [models/websocket.rs](file://crates/okx-client/src/models/websocket.rs)
- [Cargo.toml](file://crates/okx-client/Cargo.toml)
- [README.md](file://examples/README.md)
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
The OKX REST API client provides a comprehensive interface for interacting with the OKX cryptocurrency exchange. This documentation details the implementation of the REST client for account management, trading operations, and market data retrieval. The client is built using reqwest with connection pooling and timeout configuration, featuring robust error handling, retry mechanisms with exponential backoff, and rate limit handling. It integrates seamlessly with trading execution and data collection systems within the quantitative trading platform.

## Project Structure
The OKX client is implemented as a Rust crate within the larger trading system. It follows a modular structure with separate components for REST and WebSocket functionality, authentication, error handling, and data models.

```mermaid
graph TB
subgraph "okx-client"
lib[lib.rs]
rest[rest.rs]
auth[auth.rs]
error[error.rs]
models[models/]
websocket[websocket.rs]
end
lib --> rest
lib --> auth
lib --> error
lib --> models
lib --> websocket
models --> request[request.rs]
models --> response[response.rs]
models --> websocket_models[websocket.rs]
```

**Diagram sources**
- [lib.rs](file://crates/okx-client/src/lib.rs)
- [rest.rs](file://crates/okx-client/src/rest.rs)
- [auth.rs](file://crates/okx-client/src/auth.rs)
- [error.rs](file://crates/okx-client/src/error.rs)
- [models/request.rs](file://crates/okx-client/src/models/request.rs)
- [models/response.rs](file://crates/okx-client/src/models/response.rs)
- [websocket.rs](file://crates/okx-client/src/websocket.rs)
- [models/websocket.rs](file://crates/okx-client/src/models/websocket.rs)

**Section sources**
- [lib.rs](file://crates/okx-client/src/lib.rs)

## Core Components
The OKX client consists of several core components that work together to provide a robust API interface. These include the REST client implementation, authentication utilities, error handling system, data models for requests and responses, and WebSocket client for real-time data streaming. The client is designed to handle both public market data endpoints and private trading/account endpoints with proper authentication.

**Section sources**
- [lib.rs](file://crates/okx-client/src/lib.rs)
- [rest.rs](file://crates/okx-client/src/rest.rs)
- [auth.rs](file://crates/okx-client/src/auth.rs)
- [error.rs](file://crates/okx-client/src/error.rs)

## Architecture Overview
The OKX client follows a modular architecture with clear separation of concerns. The REST client handles HTTP requests to the OKX API, while the WebSocket client manages real-time data streaming. Authentication is handled through dedicated utilities that generate the required signatures for API requests. Data models provide type-safe representations of API requests and responses.

```mermaid
graph TD
subgraph "Client Interface"
App[Trading Application]
end
subgraph "OKX Client"
REST[REST Client]
WS[WebSocket Client]
Auth[Authentication]
Models[Data Models]
Error[Error Handling]
end
subgraph "External Services"
OKX[OKX Exchange API]
end
App --> REST
App --> WS
REST --> Auth
REST --> Models
REST --> Error
WS --> Auth
WS --> Models
WS --> Error
REST --> OKX
WS --> OKX
```

**Diagram sources**
- [lib.rs](file://crates/okx-client/src/lib.rs)
- [rest.rs](file://crates/okx-client/src/rest.rs)
- [websocket.rs](file://crates/okx-client/src/websocket.rs)
- [auth.rs](file://crates/okx-client/src/auth.rs)

## Detailed Component Analysis

### REST Client Implementation
The REST client provides synchronous and asynchronous methods for interacting with the OKX API. It handles HTTP requests, authentication, error handling, and response parsing.

```mermaid
classDiagram
class OkxRestClient {
+new(credentials : Credentials, testnet : bool) Result<Self>
}
class Credentials {
-api_key : String
-secret_key : String
-passphrase : String
+new(api_key, secret_key, passphrase) Self
+api_key() &str
+passphrase() &str
+sign(timestamp, method, request_path, body) Result<String>
+timestamp() String
}
class RequestSigner {
-credentials : Credentials
+new(credentials) Self
+sign_request(method, request_path, body) Result<(String, String)>
+api_key() &str
+passphrase() &str
}
class ApiResponse~T~ {
+code : String
+msg : String
+data : Vec~T~
+is_success() bool
}
class PlaceOrderRequest {
+inst_id : String
+td_mode : String
+side : String
+ord_type : String
+sz : String
+px : Option~String~
+cl_ord_id : Option~String~
}
class OrderResponse {
+ord_id : String
+cl_ord_id : String
+state : String
}
OkxRestClient --> Credentials : "uses"
RequestSigner --> Credentials : "contains"
ApiResponse~T~ --> OrderResponse : "specialization"
```

**Diagram sources**
- [rest.rs](file://crates/okx-client/src/rest.rs)
- [auth.rs](file://crates/okx-client/src/auth.rs)
- [models/request.rs](file://crates/okx-client/src/models/request.rs)
- [models/response.rs](file://crates/okx-client/src/models/response.rs)

**Section sources**
- [rest.rs](file://crates/okx-client/src/rest.rs)
- [auth.rs](file://crates/okx-client/src/auth.rs)
- [models/request.rs](file://crates/okx-client/src/models/request.rs)
- [models/response.rs](file://crates/okx-client/src/models/response.rs)

### WebSocket Client Implementation
The WebSocket client provides real-time market data streaming from OKX. It handles connection management, subscription to various channels, authentication for private channels, and message parsing.

```mermaid
sequenceDiagram
participant App as "Trading Application"
participant Client as "OkxWebSocketClient"
participant PublicWS as "Public WebSocket"
participant PrivateWS as "Private WebSocket"
participant OKX as "OKX Exchange"
App->>Client : new(credentials, is_testnet)
App->>Client : connect()
Client->>PublicWS : connect to public URL
PublicWS-->>Client : connected
Client->>PrivateWS : connect to private URL
PrivateWS-->>Client : connected
Client->>Client : authenticate()
Client->>PrivateWS : send login request
PrivateWS-->>Client : login confirmation
Client-->>App : connected
App->>Client : subscribe(requests)
Client->>Client : partition public/private
Client->>PublicWS : send subscription
Client->>PrivateWS : send subscription
PublicWS-->>Client : subscription confirmation
PrivateWS-->>Client : subscription confirmation
Client-->>App : subscribed
loop Message Processing
PublicWS->>Client : ticker/candle data
PrivateWS->>Client : account/position data
Client->>Client : parse message
Client->>App : next_message()
end
App->>Client : disconnect()
Client->>PublicWS : close connection
Client->>PrivateWS : close connection
Client-->>App : disconnected
```

**Diagram sources**
- [websocket.rs](file://crates/okx-client/src/websocket.rs)
- [models/websocket.rs](file://crates/okx-client/src/models/websocket.rs)

**Section sources**
- [websocket.rs](file://crates/okx-client/src/websocket.rs)
- [models/websocket.rs](file://crates/okx-client/src/models/websocket.rs)

### Error Handling Strategy
The client implements a comprehensive error handling system with specific error types for different failure modes, including HTTP errors, WebSocket errors, authentication issues, rate limiting, and API-specific errors.

```mermaid
graph TD
Error[Error] --> HttpError["HttpError(reqwest::Error)"]
Error --> WebSocketError["WebSocketError(String)"]
Error --> WebSocketConnection["WebSocketConnection(String)"]
Error --> WebSocketSend["WebSocketSend(String)"]
Error --> ParseError["ParseError(String)"]
Error --> AuthError["AuthError(String)"]
Error --> RateLimitExceeded["RateLimitExceeded(String)"]
Error --> InvalidResponse["InvalidResponse(String)"]
Error --> ApiError["ApiError { code: String, message: String }"]
Error --> SerializationError["SerializationError(serde_json::Error)"]
Error --> UrlError["UrlError(url::ParseError)"]
Error --> Timeout["Timeout(String)"]
Error --> ConnectionError["ConnectionError(String)"]
Error --> Internal["Internal(String)"]
style Error fill:#f9f,stroke:#333,stroke-width:2px
```

**Diagram sources**
- [error.rs](file://crates/okx-client/src/error.rs)

**Section sources**
- [error.rs](file://crates/okx-client/src/error.rs)

## Dependency Analysis
The OKX client has dependencies on several external crates for HTTP communication, serialization, cryptography, and async runtime functionality. These dependencies are managed through Cargo and are specified in the Cargo.toml file.

```mermaid
graph LR
okx-client --> reqwest
okx-client --> serde
okx-client --> serde_json
okx-client --> tokio
okx-client --> hmac
okx-client --> sha2
okx-client --> base64
okx-client --> chrono
okx-client --> tracing
okx-client --> url
okx-client --> thiserror
okx-client --> anyhow
reqwest --> hyper
reqwest --> tokio
serde_json --> serde
hmac --> crypto-mac
sha2 --> digest
base64 --> base64ct
tracing --> log
```

**Diagram sources**
- [Cargo.toml](file://crates/okx-client/Cargo.toml)

**Section sources**
- [Cargo.toml](file://crates/okx-client/Cargo.toml)

## Performance Considerations
The OKX client is designed with performance in mind, utilizing connection pooling through reqwest, efficient data serialization with serde, and non-blocking async operations with tokio. The WebSocket client maintains persistent connections for real-time data streaming, reducing latency compared to polling REST endpoints. The client also implements exponential backoff for retry mechanisms to handle transient failures without overwhelming the API servers.

## Troubleshooting Guide
When encountering issues with the OKX client, consider the following common problems and solutions:

1. **Authentication failures**: Verify that API key, secret key, and passphrase are correct and have the necessary permissions.
2. **Rate limiting**: Implement proper rate limiting in your application and handle RateLimitExceeded errors appropriately.
3. **WebSocket disconnections**: The client has auto-reconnection logic, but monitor connection state and subscription status.
4. **Parsing errors**: Ensure that data models match the API response format, especially after API updates.
5. **Network issues**: Check network connectivity and firewall settings, especially for WebSocket connections.

For debugging, enable tracing logs to see detailed information about API requests and responses.

**Section sources**
- [auth.rs](file://crates/okx-client/src/auth.rs)
- [error.rs](file://crates/okx-client/src/error.rs)
- [websocket.rs](file://crates/okx-client/src/websocket.rs)

## Conclusion
The OKX REST API client provides a robust and feature-complete interface for interacting with the OKX exchange. It handles authentication, error management, and data serialization while providing both REST and WebSocket interfaces for different use cases. The client is designed to integrate seamlessly with trading systems, supporting both market data collection and trade execution workflows. With proper error handling and retry mechanisms, it provides reliable access to exchange functionality for algorithmic trading applications.