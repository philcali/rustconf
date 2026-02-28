# Implementation Plan: Server-Side Code Generation

## Overview

This plan implements server-side code generation for rustconf, enabling developers to generate RESTCONF server scaffolding from YANG schemas. The implementation follows the existing client-side architecture patterns and reuses type generation and validation infrastructure.

## Tasks

- [x] 1. Create server-side runtime types in rustconf-runtime
  - [x] 1.1 Add ServerError type with HTTP status code mapping
    - Define error variants (ValidationError, DeserializationError, HandlerError, NotFound, InternalError)
    - Implement status_code() method for HTTP mapping
    - Implement to_restconf_error() for RFC 8040 formatting
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
  
  - [ ]* 1.2 Write property test for error status code mapping
    - **Property 10: Error Response Formatting**
    - **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6**
  
  - [x] 1.3 Add ServerTransport trait for framework abstraction
    - Define trait with serve() method
    - Add ServerRequest and ServerResponse types
    - Mirror client-side HttpTransport pattern
    - _Requirements: 6.1, 6.2, 6.4_
  
  - [ ]* 1.4 Write property test for transport abstraction
    - **Property 13: Transport-Agnostic Handler Interfaces**
    - **Property 14: Transport Adapter Capability**
    - **Validates: Requirements 6.1, 6.4, 6.5**

- [x] 2. Extend GeneratorConfig for server generation
  - [x] 2.1 Add enable_server_generation flag to GeneratorConfig
    - Add boolean field with default false
    - Add server_output_subdir field with default "server"
    - Update builder methods
    - _Requirements: 10.1, 10.2_
  
  - [x] 2.2 Add configuration validation
    - Validate output directories are valid paths
    - Validate server generation is compatible with other flags
    - Return clear error messages for invalid configs
    - _Requirements: 10.4_
  
  - [ ]* 2.3 Write property test for configuration validation
    - **Property 19: Configuration Validation**
    - **Validates: Requirements 10.4**

- [x] 3. Implement server handler trait generation
  - [x] 3.1 Create server/handlers.rs generator module
    - Add ServerHandlerGenerator struct
    - Implement generate_handler_trait() method
    - Generate trait with async methods for each YANG operation
    - Include doc comments from YANG descriptions
    - _Requirements: 1.1, 1.4, 1.5, 1.6, 2.5_
  
  - [x] 3.2 Generate handler methods for YANG RPCs
    - Parse RPC input/output nodes
    - Generate method signature with input parameters
    - Use Result<Output, ServerError> return type
    - _Requirements: 1.1, 1.5, 1.6_
  
  - [x] 3.3 Generate handler methods for YANG data nodes
    - Generate GET method for all data nodes
    - Generate PUT, PATCH, POST, DELETE only if config=true
    - Use appropriate parameter and return types
    - _Requirements: 1.2, 1.3_
  
  - [ ]* 3.4 Write property test for handler generation
    - **Property 1: Handler Method Generation Completeness**
    - **Property 2: CRUD Operations Based on Config Flag**
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6**

- [x] 4. Implement stub handler generation
  - [x] 4.1 Create server/stubs.rs generator module
    - Add StubHandlerGenerator struct
    - Implement generate_stub_impl() method
    - Generate struct implementing handler trait
    - Add call_log field for debugging
    - _Requirements: 2.1, 2.4_
  
  - [x] 4.2 Generate default values for YANG types
    - Implement default_value_for_type() helper
    - Handle all YANG built-in types (uint8-64, int8-64, string, boolean, etc.)
    - Return sensible defaults (0 for numbers, "" for strings, false for booleans)
    - _Requirements: 2.3_
  
  - [x] 4.3 Add call logging to stub methods
    - Log method name and parameters
    - Store in Arc<Mutex<Vec<String>>>
    - Provide get_call_log() accessor
    - _Requirements: 2.4_
  
  - [ ]* 4.4 Write property test for stub handler completeness
    - **Property 3: Stub Handler Completeness**
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

- [x] 5. Checkpoint - Ensure handler generation compiles
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement request router
  - [x] 6.1 Create server/router.rs generator module
    - Add RouterGenerator struct
    - Generate RestconfRouter struct with handler field
    - Implement route() method skeleton
    - _Requirements: 4.1, 4.5_
  
  - [x] 6.2 Implement path parsing and matching
    - Parse RESTCONF URL paths into segments
    - Match against YANG-defined paths
    - Extract list keys from path segments
    - Handle percent-encoded key values
    - Return 404 for unmatched paths
    - _Requirements: 4.1, 4.4, 4.6_
  
  - [x] 6.3 Implement request deserialization
    - Deserialize request body based on matched operation
    - Use serde_json for JSON parsing
    - Return 400 for deserialization failures with error details
    - _Requirements: 4.2, 4.3_
  
  - [x] 6.4 Implement handler method dispatch
    - Call appropriate handler trait method based on matched path
    - Pass deserialized input parameters
    - Handle async handler invocation
    - _Requirements: 4.5_
  
  - [ ]* 6.5 Write property tests for routing
    - **Property 6: Path Routing Correctness**
    - **Property 7: Request Deserialization**
    - **Property 8: Invalid Request Handling**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6**

- [x] 7. Implement validation wrappers
  - [x] 7.1 Add request validation before handler invocation
    - Reuse existing validation infrastructure from validation.rs
    - Validate all input fields against YANG constraints
    - Return 400 with constraint details if validation fails
    - Skip handler invocation on validation failure
    - _Requirements: 3.1, 3.2, 7.1, 7.2_
  
  - [x] 7.2 Add response validation before serialization
    - Validate handler output against YANG constraints
    - Check range, pattern, and mandatory field constraints
    - Return 500 if output validation fails
    - _Requirements: 3.3, 3.4, 7.3, 7.4, 7.5, 7.6, 7.7_
  
  - [ ]* 7.3 Write property tests for validation
    - **Property 4: Handler Validation Wrapper**
    - **Property 5: Response Validation Before Serialization**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 7.1-7.7**

- [x] 8. Implement response serialization
  - [x] 8.1 Add response serialization in router
    - Serialize handler output to JSON using serde
    - Add Content-Type: application/json header
    - Handle serialization errors with 500 response
    - _Requirements: 5.1, 5.3_
  
  - [x] 8.2 Add content negotiation support
    - Parse Accept header from request
    - Support application/json and application/xml
    - Serialize response in requested format
    - Default to JSON if no Accept header
    - _Requirements: 5.4_
  
  - [x] 8.3 Implement RESTCONF error response formatting
    - Format ServerError as RFC 8040 error structure
    - Include error-type, error-tag, error-message fields
    - Support multiple errors in single response
    - _Requirements: 5.2, 9.5, 9.6_
  
  - [x] 8.4 Write property tests for serialization
    - **Property 9: Response Serialization Round-Trip**
    - **Property 11: Content-Type Header Presence**
    - **Property 12: Content Negotiation**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 13.2, 13.3**

- [ ] 9. Checkpoint - Ensure routing and serialization work end-to-end
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement notification support
  - [ ] 10.1 Generate notification publisher types
    - Create NotificationPublisher struct
    - Add publish() method for each YANG notification
    - Generate type-safe notification data structures
    - _Requirements: 8.1, 8.4_
  
  - [ ] 10.2 Add subscriber management
    - Implement subscriber registration
    - Support multiple concurrent subscribers
    - Handle subscriber lifecycle
    - _Requirements: 8.3_
  
  - [ ] 10.3 Implement transport-specific notification delivery
    - Integrate with ServerTransport for delivery
    - Serialize notifications according to YANG schema
    - Handle delivery failures gracefully
    - _Requirements: 8.2, 8.5_
  
  - [ ]* 10.4 Write property tests for notifications
    - **Property 15: Notification Type Generation**
    - **Property 16: Notification Serialization Round-Trip**
    - **Property 17: Concurrent Notification Delivery**
    - **Property 18: Notification Transport Delivery**
    - **Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5**

- [ ] 11. Implement handler registry
  - [ ] 11.1 Create HandlerRegistry type
    - Store path patterns mapped to handler methods
    - Support registration and lookup
    - Return default not-implemented handler for unregistered paths
    - _Requirements: 11.1, 11.2, 11.3, 11.5_
  
  - [ ]* 11.2 Write property test for registry lookup
    - **Property 20: Handler Registry Lookup**
    - **Validates: Requirements 11.2, 11.3**

- [ ] 12. Implement modular server code organization
  - [ ] 12.1 Create server/mod.rs generator
    - Generate module declarations for handlers, stubs, router, transport
    - Add re-exports for commonly used types
    - Include appropriate use statements
    - _Requirements: 12.1, 12.2, 12.5_
  
  - [ ] 12.2 Update main generator to create server subdirectory
    - Create server/ directory in output path
    - Generate handlers.rs, stubs.rs, router.rs files
    - Wire server module into main mod.rs
    - _Requirements: 12.3, 12.4_
  
  - [ ]* 12.3 Write property test for module compilation
    - **Property 21: Module Compilation**
    - **Validates: Requirements 12.5**

- [ ] 13. Ensure client-server type compatibility
  - [ ] 13.1 Verify shared type generation
    - Ensure types.rs is identical for client and server
    - Use same serde attributes on both sides
    - Verify validation.rs is shared
    - _Requirements: 13.1, 13.4, 13.5_
  
  - [ ]* 13.2 Write property test for type compatibility
    - **Property 22: Client-Server Type Compatibility**
    - **Validates: Requirements 13.1, 13.4, 13.5**
  
  - [ ]* 13.3 Write integration test for client-server round-trip
    - Generate both client and server from same YANG
    - Serialize request on client, deserialize on server
    - Serialize response on server, deserialize on client
    - Verify data equivalence
    - _Requirements: 13.2, 13.3_

- [ ] 14. Add server generation to RustconfBuilder API
  - [ ] 14.1 Add enable_server_generation() builder method
    - Add method to RustconfBuilder
    - Set enable_server_generation flag in config
    - Update documentation
    - _Requirements: 10.1_
  
  - [ ] 14.2 Add server_output_dir() builder method
    - Add method to configure server output location
    - Validate path is valid
    - Update documentation
    - _Requirements: 10.2_
  
  - [ ] 14.3 Update generate() to invoke server generation
    - Check enable_server_generation flag
    - Call server generator if enabled
    - Write server files to configured directory
    - _Requirements: 10.1_

- [ ] 15. Create example demonstrating server generation
  - [ ] 15.1 Create examples/server-basic directory
    - Set up Cargo.toml with rustconf and rustconf-runtime dependencies
    - Create build.rs with server generation enabled
    - Add YANG schema file
    - _Requirements: All_
  
  - [ ] 15.2 Implement example using stub handlers
    - Show how to use generated stub handlers as-is
    - Demonstrate call logging for testing
    - Show request/response flow
    - _Requirements: 2.1, 2.2, 2.4_
  
  - [ ] 15.3 Implement example with custom handler
    - Show how to override specific handler methods
    - Demonstrate mixing stub and custom implementations
    - Show production usage pattern
    - _Requirements: 3.5_

- [ ] 16. Final checkpoint - Ensure all tests pass and examples run
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Server generation reuses existing type and validation generation
- Handler traits use async/await for consistency with client operations
- Stub implementations provide immediate usability for testing
- Developers can selectively override handlers for production use
- The intermediate crate pattern works seamlessly with server generation
- **IMPORTANT**: Always re-read files before editing them, as they may have been modified by cargo fmt or clippy fixes since you last read them
