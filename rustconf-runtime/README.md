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

### Example

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

### Custom Transport

You can implement your own transport by implementing the `HttpTransport` trait:

```rust
use async_trait::async_trait;
use rustconf_runtime::{HttpTransport, HttpRequest, HttpResponse, RpcError};

struct MyCustomTransport;

#[async_trait]
impl HttpTransport for MyCustomTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Your custom implementation
        todo!()
    }
}
```

### Request Interceptors

Add cross-cutting concerns like authentication or logging:

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
    .with_interceptor(AuthInterceptor { token: "...".to_string() });
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
