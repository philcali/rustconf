# Requirements Document: RESTful RPC Generation

## Introduction

This feature enhances the rustconf library to generate functional RESTful HTTP client code from YANG/RESTCONF specifications. Currently, rustconf generates async RPC function stubs that return `Err(RpcError::NotImplemented)`. This enhancement will generate actual HTTP client implementations while maintaining backward compatibility with existing stub generation.

The feature provides a pluggable HTTP transport layer, runtime configuration for managing multiple devices, request interceptors for authentication flows, RESTCONF-compliant URL generation, and configurable error mapping strategies.

## Glossary

- **RPC_Generator**: The rustconf code generation system that produces Rust bindings from YANG specifications
- **HTTP_Transport**: An abstraction layer for executing HTTP requests, implemented by adapters like reqwest or hyper
- **Request_Interceptor**: A hook mechanism that allows modification of requests before sending and responses after receiving
- **RestconfClient**: The generated client struct that manages HTTP communication with RESTCONF servers
- **NamespaceMode**: Configuration option controlling whether YANG module namespaces are included in generated URLs
- **RpcError**: The error type used throughout rustconf for RPC operation failures
- **Feature_Flag**: Cargo feature flags used to conditionally compile transport adapter code
- **Base_URL**: The runtime-configurable root URL for a specific device's RESTCONF endpoint
- **RESTCONF_URL**: A URL formatted according to RESTCONF specification (RFC 8040) including module namespaces
- **Transport_Adapter**: A concrete implementation of the HTTP_Transport trait for a specific HTTP library

## Requirements

### Requirement 1: HTTP Transport Abstraction

**User Story:** As a library user, I want to choose my HTTP client library, so that I can use the HTTP stack that best fits my application's needs.

#### Acceptance Criteria

1. THE RPC_Generator SHALL generate an async HttpTransport trait with an execute method
2. WHEN the HttpTransport trait is generated, THE RPC_Generator SHALL include Send and Sync bounds for thread safety
3. THE RPC_Generator SHALL generate HttpRequest and HttpResponse types that are transport-agnostic
4. THE HttpRequest type SHALL include fields for method, url, headers, and body
5. THE HttpResponse type SHALL include fields for status_code, headers, and body

### Requirement 2: Built-in Transport Adapters

**User Story:** As a library user, I want ready-to-use adapters for popular HTTP libraries, so that I can quickly integrate without writing boilerplate code.

#### Acceptance Criteria

1. WHEN the reqwest-client feature flag is enabled, THE RPC_Generator SHALL generate a ReqwestTransport adapter implementing HttpTransport
2. WHEN the hyper-client feature flag is enabled, THE RPC_Generator SHALL generate a HyperTransport adapter implementing HttpTransport
3. THE RPC_Generator SHALL generate feature-gated module declarations for each transport adapter
4. WHEN a transport adapter is generated, THE RPC_Generator SHALL include proper async/await error handling
5. THE generated transport adapters SHALL convert library-specific errors to RpcError

### Requirement 3: Client Struct with Runtime Configuration

**User Story:** As a developer managing multiple network devices, I want to configure different base URLs at runtime, so that I can use the same generated code for different devices without recompilation.

#### Acceptance Criteria

1. THE RPC_Generator SHALL generate a RestconfClient struct with generic type parameter for HttpTransport
2. THE RestconfClient SHALL include a base_url field of type String
3. THE RestconfClient SHALL include a transport field of the generic HttpTransport type
4. THE RestconfClient SHALL include an optional interceptor field
5. THE RPC_Generator SHALL generate a constructor method accepting base_url and transport parameters

### Requirement 4: Request Interceptor Pattern

**User Story:** As a developer implementing authentication, I want to intercept and modify requests before sending, so that I can add session tokens or other authentication headers dynamically.

#### Acceptance Criteria

1. THE RPC_Generator SHALL generate an async RequestInterceptor trait with before_request and after_response methods
2. THE RequestInterceptor trait SHALL include Send and Sync bounds
3. WHEN before_request is called, THE Request_Interceptor SHALL receive a mutable reference to HttpRequest
4. WHEN after_response is called, THE Request_Interceptor SHALL receive an immutable reference to HttpResponse
5. THE RestconfClient SHALL provide a with_interceptor method for adding interceptors
6. WHEN an interceptor is configured, THE RestconfClient SHALL call before_request before sending each HTTP request
7. WHEN an interceptor is configured, THE RestconfClient SHALL call after_response after receiving each HTTP response

### Requirement 5: RESTCONF URL Generation

**User Story:** As a RESTCONF client developer, I want URLs generated according to RFC 8040 with proper namespace formatting, so that my requests are compatible with standard RESTCONF servers.

#### Acceptance Criteria

1. WHEN NamespaceMode is Enabled, THE RPC_Generator SHALL generate URLs in the format `/restconf/operations/{module}:{operation}`
2. WHEN NamespaceMode is Disabled, THE RPC_Generator SHALL generate URLs in the format `/restconf/operations/{operation}`
3. THE RPC_Generator SHALL extract module names from YANG specifications for namespace generation
4. THE RPC_Generator SHALL URL-encode operation names and module names
5. THE RPC_Generator SHALL combine base_url with generated paths to create complete URLs

### Requirement 6: RESTful RPC Function Generation

**User Story:** As a library user, I want generated RPC functions to make actual HTTP calls, so that I can interact with real RESTCONF servers instead of getting NotImplemented errors.

#### Acceptance Criteria

1. WHEN enable_restful_rpcs is true, THE RPC_Generator SHALL generate async RPC functions that execute HTTP requests
2. WHEN enable_restful_rpcs is false, THE RPC_Generator SHALL generate stub functions returning NotImplemented errors
3. THE generated RPC functions SHALL accept a RestconfClient reference and input parameters
4. THE generated RPC functions SHALL serialize input parameters to JSON
5. THE generated RPC functions SHALL deserialize HTTP response bodies to output types
6. THE generated RPC functions SHALL use POST method for RESTCONF operations
7. THE generated RPC functions SHALL set Content-Type header to application/yang-data+json

### Requirement 7: Error Mapping Strategy

**User Story:** As a developer handling errors, I want HTTP status codes automatically mapped to appropriate error types, so that I can handle different failure scenarios correctly.

#### Acceptance Criteria

1. WHEN an HTTP response has status 400, THE RPC_Generator SHALL map it to RpcError::InvalidInput
2. WHEN an HTTP response has status 401 or 403, THE RPC_Generator SHALL map it to RpcError::Unauthorized
3. WHEN an HTTP response has status 404, THE RPC_Generator SHALL map it to RpcError::NotFound
4. WHEN an HTTP response has status 500-599, THE RPC_Generator SHALL map it to RpcError::ServerError
5. WHEN an HTTP response has status 200-299, THE RPC_Generator SHALL attempt to deserialize the response body
6. THE RPC_Generator SHALL generate an ErrorMapper trait for custom error mapping strategies
7. THE RestconfClient SHALL support optional custom ErrorMapper implementations

### Requirement 8: Feature Flag Management

**User Story:** As a library maintainer, I want transport adapters behind feature flags, so that users only compile dependencies they actually use.

#### Acceptance Criteria

1. THE RPC_Generator SHALL generate #[cfg(feature = "reqwest-client")] attributes for reqwest adapter code
2. THE RPC_Generator SHALL generate #[cfg(feature = "hyper-client")] attributes for hyper adapter code
3. THE RPC_Generator SHALL generate a Cargo.toml template with optional feature dependencies
4. THE generated Cargo.toml SHALL list reqwest as an optional dependency
5. THE generated Cargo.toml SHALL list hyper as an optional dependency
6. WHEN no transport feature flags are enabled, THE generated code SHALL compile without transport adapters

### Requirement 9: Configuration API

**User Story:** As a build script author, I want a clear configuration API, so that I can enable RESTful RPC generation with appropriate options.

#### Acceptance Criteria

1. THE RPC_Generator SHALL provide an enable_restful_rpcs method on the builder
2. THE RPC_Generator SHALL provide a restful_namespace_mode method accepting NamespaceMode enum
3. THE NamespaceMode enum SHALL have Enabled and Disabled variants
4. WHEN enable_restful_rpcs is not called, THE RPC_Generator SHALL default to false
5. WHEN restful_namespace_mode is not called, THE RPC_Generator SHALL default to NamespaceMode::Enabled
6. THE RPC_Generator SHALL validate that enable_restful_rpcs is true when restful_namespace_mode is set

### Requirement 10: Backward Compatibility

**User Story:** As an existing rustconf user, I want the new feature to be additive, so that my existing code continues to work without modifications.

#### Acceptance Criteria

1. WHEN enable_restful_rpcs is false, THE RPC_Generator SHALL generate the same stub functions as before
2. THE RPC_Generator SHALL not modify existing RpcError variants
3. THE RPC_Generator SHALL not change existing function signatures when RESTful generation is disabled
4. THE generated code SHALL compile with the same dependencies as before when RESTful features are disabled
5. THE RPC_Generator SHALL maintain existing module structure and naming conventions

### Requirement 11: Serialization and Deserialization

**User Story:** As a developer calling RPCs, I want input and output types automatically serialized, so that I can work with native Rust types instead of manual JSON handling.

#### Acceptance Criteria

1. THE RPC_Generator SHALL serialize RPC input types to JSON using serde_json
2. THE RPC_Generator SHALL deserialize HTTP response bodies to RPC output types using serde_json
3. WHEN serialization fails, THE RPC_Generator SHALL return RpcError::SerializationError
4. WHEN deserialization fails, THE RPC_Generator SHALL return RpcError::DeserializationError
5. THE RPC_Generator SHALL include proper error context in serialization errors

### Requirement 12: Documentation Generation

**User Story:** As a library user, I want generated code to include documentation, so that I understand how to use the RESTful RPC functions.

#### Acceptance Criteria

1. THE RPC_Generator SHALL generate doc comments for the RestconfClient struct
2. THE RPC_Generator SHALL generate doc comments for the HttpTransport trait
3. THE RPC_Generator SHALL generate doc comments for each RPC function
4. THE RPC_Generator SHALL include usage examples in RestconfClient documentation
5. THE RPC_Generator SHALL document required feature flags in transport adapter modules

### Requirement 13: Configuration Validation

**User Story:** As a build script author, I want clear error messages when configuration is invalid, so that I can quickly fix build issues.

#### Acceptance Criteria

1. WHEN restful_namespace_mode is set but enable_restful_rpcs is false, THE RPC_Generator SHALL return a configuration error
2. WHEN a YANG file cannot be parsed, THE RPC_Generator SHALL return an error with file path and line number
3. WHEN module namespace cannot be extracted, THE RPC_Generator SHALL return an error with module name
4. THE RPC_Generator SHALL validate that base_url is a valid URL format at runtime
5. THE configuration error messages SHALL include actionable guidance for resolution

### Requirement 14: Interceptor Execution Order

**User Story:** As a developer implementing complex authentication flows, I want predictable interceptor execution order, so that I can compose multiple interceptors reliably.

#### Acceptance Criteria

1. WHEN multiple interceptors are configured, THE RestconfClient SHALL execute before_request hooks in registration order
2. WHEN multiple interceptors are configured, THE RestconfClient SHALL execute after_response hooks in reverse registration order
3. WHEN a before_request hook returns an error, THE RestconfClient SHALL abort the request and return the error
4. WHEN an after_response hook returns an error, THE RestconfClient SHALL return the error without calling subsequent hooks
5. THE RestconfClient SHALL execute all before_request hooks before sending the HTTP request

### Requirement 15: Custom Transport Implementation

**User Story:** As an advanced user with specific HTTP requirements, I want to implement custom transports, so that I can integrate with specialized HTTP clients or add custom behavior.

#### Acceptance Criteria

1. THE HttpTransport trait SHALL be public and documented
2. THE HttpRequest and HttpResponse types SHALL be public with all fields accessible
3. THE RPC_Generator SHALL generate examples of custom transport implementation
4. WHEN a custom transport is provided, THE RestconfClient SHALL use it without requiring built-in adapters
5. THE custom transport implementation SHALL have access to all request details including headers and body
