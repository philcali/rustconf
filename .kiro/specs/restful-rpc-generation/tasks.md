# Implementation Plan: RESTful RPC Generation

## Overview

This implementation plan breaks down the RESTful RPC generation feature into discrete, incremental coding tasks. The approach is additive—extending the existing rustconf generator without breaking existing functionality. Tasks are organized to deliver core functionality first, followed by transport adapters, interceptors, and comprehensive testing.

## Tasks

- [x] 1. Extend configuration API and data models
  - Add `enable_restful_rpcs`, `restful_namespace_mode`, and related fields to `GeneratorConfig`
  - Implement `NamespaceMode` enum with `Enabled` and `Disabled` variants
  - Add builder methods: `enable_restful_rpcs()` and `restful_namespace_mode()`
  - Implement configuration validation logic (error when namespace_mode is set but enable_restful_rpcs is false)
  - Extend `RpcError` enum with new variants: `TransportError`, `SerializationError`, `DeserializationError`, `InvalidInput`, `Unauthorized`, `NotFound`, `ServerError`, `UnknownError`
  - Implement `Display` and `Error` traits for extended `RpcError`
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 13.1, 10.2_

- [ ]* 1.1 Write property test for configuration validation
  - **Property 10: Configuration Validation**
  - **Validates: Requirements 9.6**

- [ ] 2. Generate core HTTP abstractions
  - [x] 2.1 Generate `HttpMethod` enum with variants: GET, POST, PUT, DELETE, PATCH
    - _Requirements: 1.3_
  
  - [x] 2.2 Generate `HttpRequest` struct with fields: method, url, headers, body
    - Make all fields public for custom transport access
    - _Requirements: 1.3, 1.4, 15.2_
  
  - [x] 2.3 Generate `HttpResponse` struct with fields: status_code, headers, body
    - Make all fields public for custom transport access
    - _Requirements: 1.3, 1.5, 15.2_
  
  - [x] 2.4 Generate `HttpTransport` trait with async execute method
    - Include `Send + Sync` bounds
    - Use `async_trait` macro
    - Add comprehensive documentation with examples
    - _Requirements: 1.1, 1.2, 15.1_
  
  - [x] 2.5 Generate `RequestInterceptor` trait with before_request and after_response methods
    - Include `Send + Sync` bounds
    - `before_request` takes `&mut HttpRequest`
    - `after_response` takes `&HttpResponse`
    - Add comprehensive documentation
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ]* 2.6 Write unit tests for HTTP abstraction types
  - Test struct construction and field access
  - Test enum variant creation
  - _Requirements: 1.3, 1.4, 1.5_

- [ ] 3. Generate RestconfClient struct
  - [ ] 3.1 Generate `RestconfClient<T: HttpTransport>` struct
    - Include fields: base_url, transport, interceptor (optional)
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  
  - [ ] 3.2 Generate constructor `new(base_url, transport)` method
    - Validate base_url format at runtime
    - Return error for invalid URLs
    - _Requirements: 3.5, 13.4_
  
  - [ ] 3.3 Generate `with_interceptor()` builder method
    - Accept `impl RequestInterceptor + 'static`
    - Store as `Box<dyn RequestInterceptor>`
    - _Requirements: 4.5_
  
  - [ ] 3.4 Generate `execute_request()` internal method
    - Call `before_request` hook if interceptor is configured
    - Execute request through transport
    - Call `after_response` hook if interceptor is configured
    - Handle errors from interceptors (abort on error)
    - Mark as `pub(crate)` for use by RPC functions
    - _Requirements: 4.6, 4.7, 14.3, 14.4, 14.5_
  
  - [ ] 3.5 Generate `base_url()` getter method
    - Mark as `pub(crate)`
    - _Requirements: 3.2_
  
  - [ ] 3.6 Add comprehensive documentation to RestconfClient
    - Include usage examples
    - Document interceptor pattern
    - _Requirements: 12.1, 12.4_

- [ ]* 3.7 Write property test for base URL validation
  - **Property 11: Base URL Validation**
  - **Validates: Requirements 13.4**

- [ ]* 3.8 Write property tests for interceptor execution order
  - **Property 12: Interceptor Before-Request Execution Order**
  - **Validates: Requirements 4.6, 14.1, 14.5**
  - **Property 13: Interceptor After-Response Execution Order**
  - **Validates: Requirements 4.7, 14.2**

- [ ]* 3.9 Write property tests for interceptor error handling
  - **Property 14: Interceptor Before-Request Error Handling**
  - **Validates: Requirements 14.3**
  - **Property 15: Interceptor After-Response Error Handling**
  - **Validates: Requirements 14.4**

- [ ] 4. Implement URL generation logic
  - [ ] 4.1 Create `UrlBuilder` helper struct
    - Store `namespace_mode` configuration
    - _Requirements: 5.1, 5.2_
  
  - [ ] 4.2 Implement `build_operation_url()` method
    - Handle `NamespaceMode::Enabled` format: `/restconf/operations/{module}:{operation}`
    - Handle `NamespaceMode::Disabled` format: `/restconf/operations/{operation}`
    - URL-encode module and operation names using `urlencoding` crate
    - Normalize base_url trailing slashes
    - _Requirements: 5.1, 5.2, 5.4, 5.5_

- [ ]* 4.3 Write property tests for URL generation
  - **Property 2: URL Generation with Namespace Enabled**
  - **Validates: Requirements 5.1, 5.4, 5.5**
  - **Property 3: URL Generation with Namespace Disabled**
  - **Validates: Requirements 5.2, 5.4, 5.5**

- [ ]* 4.4 Write unit tests for URL edge cases
  - Test empty module/operation names
  - Test base URLs with/without trailing slashes
  - Test Unicode characters in names
  - Test special characters requiring encoding
  - _Requirements: 5.4, 5.5_

- [ ] 5. Generate RESTful RPC functions
  - [ ] 5.1 Modify RPC function generator to check `enable_restful_rpcs` flag
    - When false: generate stub functions (existing behavior)
    - When true: generate RESTful implementations
    - _Requirements: 6.1, 6.2, 10.1_
  
  - [ ] 5.2 Generate RESTful RPC function implementations
    - Accept `client: &RestconfClient<T>` and input parameters
    - Serialize input to JSON using `serde_json::to_vec()`
    - Construct RESTCONF URL using `UrlBuilder`
    - Build `HttpRequest` with POST method
    - Set Content-Type header to "application/yang-data+json"
    - Set Accept header to "application/yang-data+json"
    - Call `client.execute_request()`
    - Map HTTP status codes to `RpcError` variants
    - Deserialize response body for 2xx status codes
    - Return appropriate errors for serialization/deserialization failures
    - _Requirements: 6.3, 6.4, 6.5, 6.6, 6.7, 11.1, 11.2, 11.3, 11.4, 11.5_
  
  - [ ] 5.3 Implement error mapping logic
    - 200-299: Attempt deserialization
    - 400: Map to `RpcError::InvalidInput`
    - 401, 403: Map to `RpcError::Unauthorized`
    - 404: Map to `RpcError::NotFound`
    - 500-599: Map to `RpcError::ServerError`
    - Other: Map to `RpcError::UnknownError`
    - Include response body in error messages
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [ ] 5.4 Generate documentation for RPC functions
    - Include function description from YANG
    - Document parameters and return types
    - Include usage examples
    - _Requirements: 12.3_

- [ ]* 5.5 Write property test for JSON serialization round-trip
  - **Property 1: JSON Serialization Round-Trip**
  - **Validates: Requirements 6.4, 6.5, 11.1, 11.2**

- [ ]* 5.6 Write property tests for HTTP method and headers
  - **Property 4: HTTP Method Consistency**
  - **Validates: Requirements 6.6**
  - **Property 5: Content-Type Header Presence**
  - **Validates: Requirements 6.7**

- [ ]* 5.7 Write property tests for error mapping
  - **Property 6: Server Error Status Code Mapping**
  - **Validates: Requirements 7.4**
  - **Property 7: Success Status Code Deserialization Attempt**
  - **Validates: Requirements 7.5**
  - **Property 8: Serialization Error Type**
  - **Validates: Requirements 11.3, 11.5**
  - **Property 9: Deserialization Error Type**
  - **Validates: Requirements 11.4, 11.5**

- [ ]* 5.8 Write unit tests for specific status code mappings
  - Test 400 → InvalidInput
  - Test 401 → Unauthorized
  - Test 403 → Unauthorized
  - Test 404 → NotFound
  - _Requirements: 7.1, 7.2, 7.3_

- [ ]* 5.9 Write property test for request completeness
  - **Property 17: Transport Request Information Completeness**
  - **Validates: Requirements 15.5**

- [ ] 6. Checkpoint - Ensure core functionality works
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 7. Generate reqwest transport adapter
  - [ ] 7.1 Generate `reqwest_adapter` module with feature gate
    - Add `#[cfg(feature = "reqwest-client")]` attribute
    - _Requirements: 2.1, 8.1_
  
  - [ ] 7.2 Generate `ReqwestTransport` struct
    - Include `reqwest::Client` field
    - _Requirements: 2.1_
  
  - [ ] 7.3 Implement constructor methods
    - `new()`: Create with default client
    - `with_client()`: Accept custom reqwest::Client
    - _Requirements: 2.1_
  
  - [ ] 7.4 Implement `HttpTransport` trait for `ReqwestTransport`
    - Convert `HttpMethod` to `reqwest::Method`
    - Build reqwest request with method, URL, headers, body
    - Execute request and await response
    - Convert reqwest errors to `RpcError::TransportError`
    - Extract status code, headers, and body from response
    - Return `HttpResponse`
    - _Requirements: 2.1, 2.4, 2.5, 18_
  
  - [ ] 7.5 Add documentation to reqwest adapter
    - Include usage examples
    - Document feature flag requirement
    - _Requirements: 12.5_

- [ ]* 7.6 Write property test for reqwest error conversion
  - **Property 18: Transport Error Conversion**
  - **Validates: Requirements 2.5**

- [ ]* 7.7 Write integration tests for reqwest adapter
  - Test with mock HTTP server
  - Test successful requests
  - Test error conditions (network errors, timeouts)
  - _Requirements: 2.1, 2.4_

- [ ] 8. Generate hyper transport adapter
  - [ ] 8.1 Generate `hyper_adapter` module with feature gate
    - Add `#[cfg(feature = "hyper-client")]` attribute
    - _Requirements: 2.2, 8.2_
  
  - [ ] 8.2 Generate `HyperTransport` struct
    - Include `hyper::Client` field
    - _Requirements: 2.2_
  
  - [ ] 8.3 Implement constructor method
    - `new()`: Create with default client
    - _Requirements: 2.2_
  
  - [ ] 8.4 Implement `HttpTransport` trait for `HyperTransport`
    - Convert `HttpMethod` to `hyper::Method`
    - Build hyper request with method, URI, headers, body
    - Execute request and await response
    - Convert hyper errors to `RpcError::TransportError`
    - Extract status code, headers, and body from response
    - Return `HttpResponse`
    - _Requirements: 2.2, 2.4, 2.5, 18_
  
  - [ ] 8.5 Add documentation to hyper adapter
    - Include usage examples
    - Document feature flag requirement
    - _Requirements: 12.5_

- [ ]* 8.6 Write integration tests for hyper adapter
  - Test with mock HTTP server
  - Test successful requests
  - Test error conditions
  - _Requirements: 2.2, 2.4_

- [ ] 9. Generate Cargo.toml template
  - [ ] 9.1 Create Cargo.toml template for generated code
    - Include core dependencies: serde, serde_json, async-trait, urlencoding
    - Add optional dependencies: reqwest, hyper
    - Define feature flags: reqwest-client, hyper-client
    - _Requirements: 8.3, 8.4, 8.5_
  
  - [ ] 9.2 Ensure generated code compiles without transport features
    - Test compilation with no features enabled
    - Verify only core traits and types are available
    - _Requirements: 8.6_

- [ ]* 9.3 Write unit tests for feature flag combinations
  - Test compilation with reqwest-client only
  - Test compilation with hyper-client only
  - Test compilation with both features
  - Test compilation with no features
  - _Requirements: 8.6_

- [ ] 10. Generate ErrorMapper trait and default implementation
  - [ ] 10.1 Generate `ErrorMapper` trait
    - Include `Send + Sync` bounds
    - Define `map_error(&self, response: &HttpResponse) -> RpcError` method
    - _Requirements: 7.6_
  
  - [ ] 10.2 Generate `DefaultErrorMapper` struct
    - Implement `ErrorMapper` trait
    - Map status codes to RpcError variants
    - Include response body in error messages
    - _Requirements: 7.6_
  
  - [ ] 10.3 Add optional error_mapper field to RestconfClient
    - Allow custom error mapper injection
    - Use DefaultErrorMapper when not provided
    - _Requirements: 7.7_

- [ ]* 10.4 Write unit tests for custom error mapper
  - Test custom mapper overriding default behavior
  - Test error mapper receiving correct response data
  - _Requirements: 7.7_

- [ ] 11. Add custom transport implementation examples
  - [ ] 11.1 Generate example custom transport in documentation
    - Show struct definition
    - Show HttpTransport trait implementation
    - Show usage with RestconfClient
    - _Requirements: 15.3_

- [ ]* 11.2 Write property test for custom transport compatibility
  - **Property 16: Custom Transport Compatibility**
  - **Validates: Requirements 15.4**

- [ ] 12. Implement backward compatibility checks
  - [ ] 12.1 Verify stub generation when enable_restful_rpcs is false
    - Generate code with flag disabled
    - Verify functions return NotImplemented
    - Verify function signatures unchanged
    - _Requirements: 10.1, 10.3_
  
  - [ ] 12.2 Verify dependency compatibility
    - Generate code with RESTful features disabled
    - Verify no new dependencies required
    - Verify compilation succeeds
    - _Requirements: 10.4_
  
  - [ ] 12.3 Verify module structure preservation
    - Check generated module names
    - Check type names and visibility
    - Ensure no breaking changes
    - _Requirements: 10.5_

- [ ]* 12.4 Write integration tests for backward compatibility
  - Test existing code continues to work
  - Test with enable_restful_rpcs=false
  - Verify no behavioral changes
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

- [ ] 13. Final checkpoint - Comprehensive testing
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Generate comprehensive documentation
  - [ ] 14.1 Add module-level documentation
    - Explain RESTful RPC generation feature
    - Document configuration options
    - Include getting started guide
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_
  
  - [ ] 14.2 Generate examples directory
    - Basic usage example with reqwest
    - Custom transport example
    - Interceptor example for authentication
    - Error handling example
    - _Requirements: 12.4, 15.3_
  
  - [ ] 14.3 Add inline code examples to generated documentation
    - RestconfClient usage
    - Transport adapter usage
    - Interceptor implementation
    - _Requirements: 12.1, 12.2, 12.4, 12.5_

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties (minimum 100 iterations each)
- Unit tests validate specific examples and edge cases
- The implementation is additive and maintains backward compatibility
- Feature flags allow users to opt-in to specific transport adapters
- **IMPORTANT**: Always read files before editing them, as they may have been reformatted or modified between tasks. Test files in particular require careful review before modifications.
