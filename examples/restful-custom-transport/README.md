# Custom Transport Implementation Example

This example demonstrates how to implement a custom HTTP transport for the RESTful RPC client.

## Overview

This example shows how to:
- Implement the `HttpTransport` trait
- Add custom behavior (retry logic, custom headers, timeouts)
- Use the custom transport with `RestconfClient`

## Why Custom Transports?

Custom transports are useful when you need:
- Retry logic with exponential backoff
- Custom timeout configurations
- Special header injection
- Request/response logging
- Integration with existing HTTP client infrastructure
- Mock transports for testing

## Running the Example

```bash
cargo run --example restful-custom-transport
```

## Code Walkthrough

The example demonstrates:

1. **Implementing HttpTransport**: Creating a struct that implements the `HttpTransport` trait
2. **Adding retry logic**: Implementing exponential backoff for failed requests
3. **Custom headers**: Injecting application-specific headers
4. **Timeout configuration**: Setting custom timeouts for requests
5. **Using the custom transport**: Creating a `RestconfClient` with the custom implementation

## Key Concepts

### HttpTransport Trait

The trait requires implementing a single async method:

```rust
async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>
```

This gives you complete control over how HTTP requests are executed.

### Retry Logic

The example shows exponential backoff retry logic, which is useful for handling transient network errors.

### Thread Safety

Custom transports must be `Send + Sync` to work with async Rust, allowing them to be used across threads.

## Next Steps

- See `restful-interceptor` for authentication patterns
- See `restful-error-handling` for advanced error handling
- Implement your own transport for your specific HTTP client or requirements
