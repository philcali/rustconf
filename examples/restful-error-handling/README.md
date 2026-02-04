# Error Handling Example

This example demonstrates comprehensive error handling patterns for RESTful RPC operations.

## Overview

This example shows how to:
- Handle different `RpcError` variants
- Implement custom error mappers
- Recover from transient errors
- Log and report errors effectively
- Use error context for debugging

## Why Error Handling Matters

Proper error handling is crucial for:
- **User experience**: Providing clear error messages
- **Debugging**: Understanding what went wrong and where
- **Reliability**: Recovering from transient failures
- **Monitoring**: Tracking error rates and patterns
- **Security**: Avoiding information leakage in error messages

## Running the Example

```bash
cargo run --example restful-error-handling
```

## Code Walkthrough

The example demonstrates:

1. **Handling RpcError variants**: Pattern matching on different error types
2. **Custom error mapper**: Implementing application-specific error mapping
3. **Error recovery**: Retrying operations and fallback strategies
4. **Error logging**: Recording error details for debugging
5. **User-friendly messages**: Converting technical errors to user messages

## Key Concepts

### RpcError Variants

The `RpcError` enum provides detailed error information:

- `TransportError`: Network or HTTP client failures
- `SerializationError`: Failed to serialize request data
- `DeserializationError`: Failed to deserialize response data
- `InvalidInput`: HTTP 400 - Bad request
- `Unauthorized`: HTTP 401/403 - Authentication failed
- `NotFound`: HTTP 404 - Resource not found
- `ServerError`: HTTP 500-599 - Server-side error
- `UnknownError`: Unexpected error condition

### Custom Error Mappers

Implement the `ErrorMapper` trait to customize how HTTP responses are converted to errors:

```rust
impl ErrorMapper for CustomErrorMapper {
    fn map_error(&self, response: &HttpResponse) -> RpcError {
        // Custom mapping logic
    }
}
```

### Error Context

Include relevant context in error messages:
- Request URL
- HTTP status code
- Response body (when safe)
- Operation being performed

## Next Steps

- Implement your own error mapper for your server's error format
- Add structured logging for error tracking
- Integrate with monitoring systems
- Implement circuit breaker patterns for failing services
