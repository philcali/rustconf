# Request Interceptor Example

This example demonstrates how to use request interceptors for authentication, logging, and request/response modification.

## Overview

This example shows how to:
- Implement the `RequestInterceptor` trait
- Add authentication headers to requests
- Log request and response details
- Validate responses before processing
- Chain multiple interceptors

## Why Interceptors?

Interceptors are useful for:
- **Authentication**: Adding tokens, API keys, or signing requests
- **Logging**: Recording request/response details for debugging
- **Monitoring**: Tracking API usage and performance
- **Validation**: Checking response headers or status codes
- **Error handling**: Custom error detection and reporting

## Running the Example

```bash
cargo run --example restful-interceptor
```

## Code Walkthrough

The example demonstrates:

1. **Authentication Interceptor**: Adds Bearer token to all requests
2. **Logging Interceptor**: Logs request and response details
3. **Validation Interceptor**: Checks response headers and status
4. **Chaining Interceptors**: Using multiple interceptors together

## Key Concepts

### RequestInterceptor Trait

The trait provides two hooks:

```rust
async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError>
async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError>
```

### Execution Order

- `before_request` hooks are called in registration order
- `after_response` hooks are called in reverse registration order
- If any hook returns an error, the request is aborted

### Error Handling

Interceptors can abort requests by returning errors:
- From `before_request`: Request is not sent
- From `after_response`: Response is discarded

## Next Steps

- See `restful-error-handling` for advanced error handling patterns
- Implement your own interceptors for specific authentication schemes
- Combine interceptors with custom transports for complete control
