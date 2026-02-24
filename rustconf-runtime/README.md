# rustconf-runtime

Runtime components for rustconf-generated RESTCONF clients.

This crate provides the core runtime types and traits needed by code generated from rustconf. It separates static runtime components from generated code, reducing code duplication and simplifying maintenance.

## Features

- **Core abstractions**: `HttpTransport` trait, `RestconfClient`, `RpcError`
- **Transport adapters**: Optional implementations for reqwest and hyper
- **Extensibility**: Implement custom transports and interceptors
- **Zero build-time dependencies**: Pure runtime library, no code generation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustconf-runtime = { version = "0.1", features = ["reqwest"] }
```

### Available Features

- `reqwest`: Enable the reqwest-based HTTP transport adapter (recommended for most users)
- `hyper`: Enable the hyper-based HTTP transport adapter (for advanced use cases)
- `default`: No features enabled by default - choose your transport explicitly

### Basic Example

```rust
use rustconf_runtime::{RestconfClient, RpcError};
use rustconf_runtime::reqwest_adapter::ReqwestTransport;

#[tokio::main]
async fn main() -> Result<(), RpcError> {
    let transport = ReqwestTransport::new();
    let client = RestconfClient::new("https://device.example.com", transport)?;
    
    // Use the client with generated operations...
    Ok(())
}
```

## Core Types and Traits

### HttpTransport Trait

The `HttpTransport` trait defines the interface for executing HTTP requests. All transport implementations must implement this trait.

```rust
#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>;
}
```

**Implementations:**
- `ReqwestTransport` (feature: `reqwest`) - Uses the reqwest HTTP client
- `HyperTransport` (feature: `hyper`) - Uses the hyper HTTP client
- Custom implementations - Implement your own for specialized needs

### RestconfClient

The main client type that manages RESTCONF communication. It wraps an `HttpTransport` and provides interceptor support.

```rust
pub struct RestconfClient<T: HttpTransport> {
    // fields are private
}

impl<T: HttpTransport> RestconfClient<T> {
    /// Create a new RESTCONF client with the given base URL and transport
    pub fn new(base_url: impl Into<String>, transport: T) -> Result<Self, RpcError>;
    
    /// Add a request interceptor to the client
    pub fn with_interceptor(self, interceptor: impl RequestInterceptor + 'static) -> Self;
}
```

**Example:**
```rust
let transport = ReqwestTransport::new();
let client = RestconfClient::new("https://device.example.com", transport)?
    .with_interceptor(AuthInterceptor { token: "...".to_string() });
```

### HttpRequest and HttpResponse

Request and response types used by the transport layer.

```rust
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub enum HttpMethod {
    GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD,
}
```

### RpcError

Error type for all RESTCONF operations.

```rust
pub enum RpcError {
    TransportError(String),
    SerializationError(String),
    DeserializationError(String),
    ValidationError(String),
    HttpError { status_code: u16, message: String },
    NotImplemented,
}
```

**Error Handling Example:**
```rust
match some_operation(&client).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(RpcError::HttpError { status_code, message }) => {
        eprintln!("HTTP error {}: {}", status_code, message);
    }
    Err(RpcError::ValidationError(msg)) => {
        eprintln!("Validation failed: {}", msg);
    }
    Err(e) => eprintln!("Other error: {:?}", e),
}
```

## Transport Adapters

### Reqwest Adapter

The reqwest adapter is recommended for most use cases. It provides a high-level, ergonomic HTTP client.

**Enable the feature:**
```toml
[dependencies]
rustconf-runtime = { version = "0.1", features = ["reqwest"] }
```

**Usage:**
```rust
use rustconf_runtime::reqwest_adapter::ReqwestTransport;

let transport = ReqwestTransport::new();
let client = RestconfClient::new("https://device.example.com", transport)?;
```

### Hyper Adapter

The hyper adapter provides lower-level control and is suitable for advanced use cases.

**Enable the feature:**
```toml
[dependencies]
rustconf-runtime = { version = "0.1", features = ["hyper"] }
```

**Usage:**
```rust
use rustconf_runtime::hyper_adapter::HyperTransport;

let transport = HyperTransport::new();
let client = RestconfClient::new("https://device.example.com", transport)?;
```

### Custom Transport

You can implement your own transport by implementing the `HttpTransport` trait:

```rust
use async_trait::async_trait;
use rustconf_runtime::{HttpTransport, HttpRequest, HttpResponse, RpcError};

struct MyCustomTransport {
    // your fields
}

#[async_trait]
impl HttpTransport for MyCustomTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Your custom implementation
        // - Handle authentication
        // - Add custom headers
        // - Implement retry logic
        // - Use a different HTTP client
        todo!()
    }
}

// Use it
let transport = MyCustomTransport { /* ... */ };
let client = RestconfClient::new("https://device.example.com", transport)?;
```

## Request Interceptors

Interceptors allow you to modify requests before they are sent. Common use cases include authentication, logging, and adding custom headers.

### RequestInterceptor Trait

```rust
pub trait RequestInterceptor: Send + Sync {
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError>;
}
```

### Authentication Example

```rust
use rustconf_runtime::{RequestInterceptor, HttpRequest, RpcError};

struct AuthInterceptor {
    token: String,
}

impl RequestInterceptor for AuthInterceptor {
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
        request.headers.push((
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        ));
        Ok(())
    }
}

// Use with client
let client = RestconfClient::new("https://device.example.com", transport)?
    .with_interceptor(AuthInterceptor { token: "my-token".to_string() });
```

### Logging Example

```rust
struct LoggingInterceptor;

impl RequestInterceptor for LoggingInterceptor {
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
        println!("Request: {} {}", request.method, request.url);
        Ok(())
    }
}

let client = RestconfClient::new("https://device.example.com", transport)?
    .with_interceptor(LoggingInterceptor);
```

### Chaining Interceptors

You can chain multiple interceptors - they will be executed in the order they are added:

```rust
let client = RestconfClient::new("https://device.example.com", transport)?
    .with_interceptor(LoggingInterceptor)
    .with_interceptor(AuthInterceptor { token: "...".to_string() })
    .with_interceptor(CustomHeaderInterceptor);
```

## Error Mapping

Customize how HTTP responses are mapped to errors:

```rust
pub trait ErrorMapper: Send + Sync {
    fn map_error(&self, response: &HttpResponse) -> RpcError;
}

pub struct DefaultErrorMapper;
```

The default error mapper handles standard HTTP error codes. You can implement custom error mapping for API-specific error formats.

## Integration with Generated Code

This crate is designed to work seamlessly with code generated by rustconf. Generated code will:

1. Import types from `rustconf_runtime`
2. Use `RestconfClient<T: HttpTransport>` for all operations
3. Return `Result<T, RpcError>` from all operations
4. Work with any transport implementation

**Example generated operation:**
```rust
pub async fn get_system_info<T: HttpTransport>(
    client: &RestconfClient<T>,
) -> Result<GetSystemInfoOutput, RpcError> {
    // Generated implementation using client.transport
}
```

## Feature Flags Summary

| Feature | Description | Dependencies Added |
|---------|-------------|-------------------|
| `reqwest` | Reqwest HTTP client adapter | reqwest |
| `hyper` | Hyper HTTP client adapter | hyper, hyper-tls |
| (none) | Core types only, no adapters | async-trait, serde |

Choose features based on your needs:
- Most users: `features = ["reqwest"]`
- Advanced/low-level: `features = ["hyper"]`
- Custom transport: no features, implement `HttpTransport` yourself

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
