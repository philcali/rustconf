# Basic RESTful RPC Usage Example

This example demonstrates the generated RESTful RPC client API from a YANG module.

## Overview

This example shows:
- The types and functions generated from YANG RPC definitions
- The structure of the RESTful RPC client API
- Input and output type structures
- How the API would be used with a real RESTCONF server

## Running the Example

```bash
cargo run -p restful-basic-example
```

## What Gets Generated

When you enable RESTful RPC generation, rustconf generates:

1. **HTTP Abstractions**
   - `HttpMethod` enum
   - `HttpRequest` and `HttpResponse` structs
   - `HttpTransport` trait for pluggable HTTP clients

2. **Client Infrastructure**
   - `RestconfClient<T>` for managing connections
   - `RequestInterceptor` trait for auth/logging
   - `RpcError` enum for error handling

3. **RPC Functions**
   - Async functions for each YANG RPC
   - Type-safe input/output structures
   - Automatic JSON serialization/deserialization

4. **Transport Adapters** (with feature flags)
   - `reqwest_adapter::ReqwestTransport`
   - `hyper_adapter::HyperTransport`

## Next Steps

- See `restful-interceptor` for authentication patterns
- See `restful-custom-transport` for custom HTTP clients
- See `restful-error-handling` for error handling strategies
