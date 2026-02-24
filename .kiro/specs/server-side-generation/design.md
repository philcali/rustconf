# Design Document: Server-Side Code Generation

## Overview

This design extends rustconf to generate server-side RESTCONF implementations from YANG schemas. The architecture mirrors the existing client-side generation, maintaining consistency in type definitions, validation, and transport abstractions.

The server generation produces:
1. **Handler traits**: Define the server-side API surface from YANG operations
2. **Stub implementations**: Provide default handlers that return sensible values
3. **Validation wrappers**: Enforce YANG constraints automatically
4. **Request router**: Parse and dispatch incoming requests

Developers can use the generated code in two ways:
- **As-is for testing**: Use stub handlers to test clients without hardware
- **Override for production**: Implement specific trait methods with real logic

Key design principles:
- Reuse existing type generation and validation infrastructure
- Mirror client-side transport abstraction pattern for server frameworks
- Maintain round-trip serialization compatibility between client and server
- Provide clear separation between generated scaffolding and developer implementation

## Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                    YANG Schema                               │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Code Generator                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Type Gen     │  │ Client Gen   │  │ Server Gen   │     │
│  │ (shared)     │  │ (existing)   │  │ (new)        │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└──────────────────────┬──────────────────┬───────────────────┘
                       │                  │
                       ▼                  ▼
        ┌──────────────────┐  ┌──────────────────┐
        │  Client Code     │  │  Server Code     │
        │  - Operations    │  │  - Handlers      │
        │  - Types         │  │  - Stubs         │
        │  - Validation    │  │  - Router        │
        └──────────────────┘  │  - Validation    │
                               └──────────────────┘
```

### Server Code Structure

Generated server code follows modular organization:

```
src/generated/
├── mod.rs              # Module declarations and re-exports
├── types.rs            # YANG-derived types (shared with client)
├── validation.rs       # Validation logic (shared with client)
└── server/
    ├── mod.rs          # Server module exports
    ├── handlers.rs     # Handler trait definitions
    ├── stubs.rs        # Stub handler implementations
    ├── router.rs       # Request routing logic
    └── transport.rs    # Server transport abstractions
```

## Components and Interfaces

### 1. Server Handler Traits

Handler traits define the interface for implementing RESTCONF operations. Each YANG module generates one handler trait.

```rust
/// Handler trait for device-management YANG module operations.
#[async_trait]
pub trait DeviceManagementHandler: Send + Sync {
    /// Execute restart-device RPC operation.
    async fn restart_device(
        &self,
        input: RestartDeviceInput,
    ) -> Result<RestartDeviceOutput, ServerError>;
    
    /// Execute get-system-info RPC operation.
    async fn get_system_info(&self) -> Result<GetSystemInfoOutput, ServerError>;
    
    /// GET operation for /interfaces container.
    async fn get_interfaces(&self) -> Result<Interfaces, ServerError>;
    
    /// PUT operation for /interfaces container.
    async fn put_interfaces(&self, data: Interfaces) -> Result<(), ServerError>;
    
    /// PATCH operation for /interfaces container.
    async fn patch_interfaces(&self, data: Interfaces) -> Result<(), ServerError>;
    
    /// DELETE operation for /interfaces container.
    async fn delete_interfaces(&self) -> Result<(), ServerError>;
}
```

### 2. Request Router

The router maps incoming HTTP requests to handler methods based on URL path and HTTP method.

```rust
pub struct RestconfRouter<H: DeviceManagementHandler> {
    handler: Arc<H>,
    base_path: String,
}

impl<H: DeviceManagementHandler> RestconfRouter<H> {
    /// Create a new router with the given handler.
    pub fn new(handler: H, base_path: impl Into<String>) -> Self {
        Self {
            handler: Arc::new(handler),
            base_path: base_path.into(),
        }
    }
    
    /// Route an incoming request to the appropriate handler.
    pub async fn route(&self, request: ServerRequest) -> ServerResponse {
        // Parse path and method
        // Match against YANG-defined paths
        // Deserialize request body if needed
        // Invoke handler method
        // Serialize response
        // Handle errors
    }
}
```

### 3. Stub Handler Implementation

Stub handlers provide default implementations that can be used as-is for testing or overridden for production.

```rust
/// Stub implementation of DeviceManagementHandler.
/// 
/// This implementation returns sensible default values for all operations.
/// Use as-is for testing, or override specific methods for production.
pub struct StubDeviceManagementHandler {
    call_log: Arc<Mutex<Vec<String>>>,
}

impl StubDeviceManagementHandler {
    pub fn new() -> Self {
        Self {
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get the log of handler calls for debugging and testing.
    pub fn get_call_log(&self) -> Vec<String> {
        self.call_log.lock().unwrap().clone()
    }
}

#[async_trait]
impl DeviceManagementHandler for StubDeviceManagementHandler {
    async fn restart_device(
        &self,
        input: RestartDeviceInput,
    ) -> Result<RestartDeviceOutput, ServerError> {
        self.call_log.lock().unwrap().push(
            format!("restart_device(delay={})", input.delay_seconds.unwrap_or(0))
        );
        
        // Return default success response
        Ok(RestartDeviceOutput {
            success: Some(true),
            message: Some("Stub: restart initiated".to_string()),
        })
    }
    
    async fn get_interfaces(&self) -> Result<Interfaces, ServerError> {
        self.call_log.lock().unwrap().push("get_interfaces()".to_string());
        
        // Return empty default
        Ok(Interfaces {
            interface: Vec::new(),
        })
    }
    
    // ... other stub implementations
}

/// Example: Override specific methods for production
pub struct ProductionHandler {
    stub: StubDeviceManagementHandler,
    // ... production-specific fields
}

#[async_trait]
impl DeviceManagementHandler for ProductionHandler {
    async fn restart_device(
        &self,
        input: RestartDeviceInput,
    ) -> Result<RestartDeviceOutput, ServerError> {
        // Real implementation
        // ... actual device restart logic ...
    }
    
    // Delegate unimplemented operations to stub
    async fn get_interfaces(&self) -> Result<Interfaces, ServerError> {
        self.stub.get_interfaces().await
    }
}
```

### 4. Server Transport Abstraction

Server transport trait abstracts over different server frameworks.

```rust
/// Trait for server-side HTTP transport implementations.
#[async_trait]
pub trait ServerTransport: Send + Sync {
    /// Start the server and begin accepting requests.
    async fn serve<H: DeviceManagementHandler + 'static>(
        &self,
        router: RestconfRouter<H>,
        bind_addr: impl Into<String> + Send,
    ) -> Result<(), ServerError>;
}

/// Server request type (parsed from HTTP).
pub struct ServerRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

/// Server response type (to be sent as HTTP).
pub struct ServerResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
```

### 5. Server Error Type

```rust
/// Error type for server-side operations.
#[derive(Debug, Clone)]
pub enum ServerError {
    /// Request validation failed.
    ValidationError(String),
    
    /// Request deserialization failed.
    DeserializationError(String),
    
    /// Response serialization failed.
    SerializationError(String),
    
    /// Handler implementation error.
    HandlerError(String),
    
    /// Resource not found.
    NotFound(String),
    
    /// Internal server error.
    InternalError(String),
}

impl ServerError {
    /// Map error to HTTP status code.
    pub fn status_code(&self) -> u16 {
        match self {
            ServerError::ValidationError(_) => 400,
            ServerError::DeserializationError(_) => 400,
            ServerError::NotFound(_) => 404,
            ServerError::SerializationError(_) => 500,
            ServerError::HandlerError(_) => 500,
            ServerError::InternalError(_) => 500,
        }
    }
    
    /// Format as RESTCONF error response.
    pub fn to_restconf_error(&self) -> RestconfErrorResponse {
        // Format according to RFC 8040 error response structure
    }
}
```

## Data Models

### Generator Configuration Extension

Extend the existing `GeneratorConfig` with server-specific options:

```rust
pub struct GeneratorConfig {
    // ... existing client fields ...
    
    /// Enable server-side code generation.
    pub enable_server_generation: bool,
    
    /// Server code output directory (relative to main output_dir).
    pub server_output_subdir: String,
}
```

### Path Matching Structure

```rust
/// Represents a RESTCONF path pattern for routing.
struct PathPattern {
    /// Path segments (e.g., ["data", "interfaces", "interface"]).
    segments: Vec<PathSegment>,
    
    /// HTTP methods supported for this path.
    methods: Vec<HttpMethod>,
    
    /// Handler method name to invoke.
    handler_method: String,
}

enum PathSegment {
    /// Static path component (e.g., "data", "interfaces").
    Static(String),
    
    /// Dynamic key parameter (e.g., "{name}").
    Key { name: String, type_name: String },
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property 1: Handler Method Generation Completeness

*For any* YANG RPC or data node definition, the generated handler trait SHALL contain a method with signature matching the operation's input parameters and output types, using Result types for error handling.

**Validates: Requirements 1.1, 1.4, 1.5, 1.6**

### Property 2: CRUD Operations Based on Config Flag

*For any* YANG data node, if config is true then the generated handler SHALL include GET, PUT, PATCH, POST, and DELETE methods, and if config is false then the handler SHALL include only GET methods.

**Validates: Requirements 1.2, 1.3**

### Property 3: Stub Handler Completeness

*For any* handler trait generated from a YANG schema, the stub implementation SHALL provide implementations for all trait methods that return valid default values conforming to YANG type constraints and log each invocation.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 4: Handler Validation Wrapper

*For any* incoming request, the server scaffolding SHALL validate request data against YANG constraints before invoking the handler, returning a 400 error if validation fails without calling the handler.

**Validates: Requirements 3.1, 3.2**

### Property 5: Response Validation Before Serialization

*For any* handler output, the server SHALL validate the response data against YANG constraints (including range, pattern, and mandatory field constraints) and return a 500 error if validation fails.

**Validates: Requirements 3.3, 3.4, 7.3, 7.4, 7.5, 7.6, 7.7, 5.5**

### Property 6: Path Routing Correctness

*For any* valid RESTCONF path corresponding to a YANG definition, the request router SHALL correctly parse the path, extract any list keys with percent-decoding, and route to the appropriate handler method.

**Validates: Requirements 4.1, 4.5, 4.6**

### Property 7: Request Deserialization

*For any* valid request body conforming to the YANG schema, the router SHALL successfully deserialize it into the appropriate input type.

**Validates: Requirements 4.2**

### Property 8: Invalid Request Handling

*For any* malformed request body or unmatched path, the router SHALL return appropriate error responses (400 for deserialization failures, 404 for unmatched paths).

**Validates: Requirements 4.3, 4.4, 7.2**

### Property 9: Response Serialization Round-Trip

*For any* valid response data type, serializing on the server and deserializing on the client SHALL produce an equivalent value, and vice versa for request data.

**Validates: Requirements 5.1, 13.2, 13.3**

### Property 10: Error Response Formatting

*For any* server error type, the formatted RESTCONF error response SHALL include the correct HTTP status code and follow RFC 8040 error structure, with all validation errors included when multiple occur.

**Validates: Requirements 5.2, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6**

### Property 11: Content-Type Header Presence

*For any* server response, the response SHALL include an appropriate Content-Type header.

**Validates: Requirements 5.3**

### Property 12: Content Negotiation

*For any* request with an Accept header specifying JSON or XML, the server SHALL serialize the response in the requested format.

**Validates: Requirements 5.4**

### Property 13: Transport-Agnostic Handler Interfaces

*For any* generated handler trait, the trait methods SHALL not reference transport-specific types, ensuring handlers work with any transport adapter.

**Validates: Requirements 6.1, 6.5**

### Property 14: Transport Adapter Capability

*For any* transport adapter implementation, it SHALL support HTTP request parsing and response writing operations.

**Validates: Requirements 6.4**

### Property 15: Notification Type Generation

*For any* YANG notification definition, the generator SHALL create a type-safe notification data structure and corresponding publisher method.

**Validates: Requirements 8.1, 8.4**

### Property 16: Notification Serialization Round-Trip

*For any* notification data, serializing on the server and deserializing on the client SHALL produce an equivalent value.

**Validates: Requirements 8.2**

### Property 17: Concurrent Notification Delivery

*For any* published notification with multiple subscribers, all subscribers SHALL receive the notification.

**Validates: Requirements 8.3**

### Property 18: Notification Transport Delivery

*For any* transport adapter, the notification publisher SHALL successfully deliver notifications through that transport's delivery mechanism.

**Validates: Requirements 8.5**

### Property 19: Configuration Validation

*For any* invalid generator configuration, the builder SHALL return a validation error before generation begins.

**Validates: Requirements 10.4**

### Property 20: Handler Registry Lookup

*For any* registered handler path pattern, the registry SHALL return the correct handler implementation when queried with a matching path.

**Validates: Requirements 11.2, 11.3**

### Property 21: Module Compilation

*For any* generated server module, the module SHALL include all necessary use statements and compile without errors.

**Validates: Requirements 12.5**

### Property 22: Client-Server Type Compatibility

*For any* YANG schema, the types generated for client and server SHALL be structurally identical with the same serde attributes.

**Validates: Requirements 13.1, 13.4, 13.5**

## Error Handling

### Error Type Hierarchy

The server-side error handling follows a similar pattern to the client-side `RpcError`, but adapted for server concerns:

```rust
pub enum ServerError {
    ValidationError(String),
    DeserializationError(String),
    SerializationError(String),
    HandlerError(String),
    NotFound(String),
    InternalError(String),
}
```

### Error Response Format

All errors are formatted according to RFC 8040 RESTCONF error response structure:

```json
{
  "ietf-restconf:errors": {
    "error": [
      {
        "error-type": "application",
        "error-tag": "invalid-value",
        "error-message": "Validation failed: port must be between 1 and 65535"
      }
    ]
  }
}
```

### Error Handling Flow

1. **Request Phase**: Validation errors → 400 Bad Request
2. **Routing Phase**: Path not found → 404 Not Found
3. **Handler Phase**: Handler errors → 500 Internal Server Error
4. **Response Phase**: Serialization errors → 500 Internal Server Error

### Multiple Error Aggregation

When multiple validation errors occur (e.g., multiple fields violate constraints), all errors are collected and returned in a single response:

```json
{
  "ietf-restconf:errors": {
    "error": [
      {
        "error-type": "application",
        "error-tag": "invalid-value",
        "error-message": "port must be between 1 and 65535"
      },
      {
        "error-type": "application",
        "error-tag": "missing-element",
        "error-message": "hostname is mandatory"
      }
    ]
  }
}
```

## Testing Strategy

### Dual Testing Approach

The server-side generation will be validated through both unit tests and property-based tests:

**Unit Tests**:
- Specific examples of handler generation for known YANG schemas
- Edge cases like empty modules, modules with only notifications
- Error conditions like invalid configuration
- Integration between router and handlers
- Specific transport adapter implementations

**Property-Based Tests**:
- Universal properties across all YANG schemas
- Round-trip serialization between client and server
- Validation enforcement for all constraint types
- Path routing for all valid RESTCONF paths
- Error mapping for all error types

### Property-Based Testing Configuration

- **Library**: Use `proptest` (already in workspace dependencies)
- **Iterations**: Minimum 100 iterations per property test
- **Tagging**: Each property test references its design document property
- **Tag Format**: `// Feature: server-side-generation, Property N: <property text>`

### Test Organization

```
rustconf/src/generator/tests/
├── server_generation.rs      # Unit tests for server code generation
├── server_handlers.rs        # Property tests for handler generation
├── server_routing.rs         # Property tests for routing logic
├── server_validation.rs      # Property tests for validation
└── server_roundtrip.rs       # Property tests for client-server compatibility
```

### Key Testing Scenarios

1. **Handler Generation**: Verify correct handler traits are generated for various YANG schemas
2. **Mock Behavior**: Verify mock handlers return valid defaults and log calls
3. **Routing**: Verify path parsing and handler dispatch for all valid paths
4. **Validation**: Verify constraint enforcement on requests and responses
5. **Round-Trip**: Verify client-serialized requests can be server-deserialized and vice versa
6. **Error Handling**: Verify all error types map to correct HTTP status codes
7. **Notifications**: Verify notification types and publisher methods are generated
8. **Transport Abstraction**: Verify handlers work with different transport adapters

### Integration Testing

Integration tests will verify end-to-end flows:
- Generate server code from YANG schema
- Implement a simple handler
- Start server with a transport adapter
- Use generated client to make requests
- Verify responses match expectations

This ensures the generated server code integrates correctly with the existing client generation and runtime components.
