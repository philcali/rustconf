# Implementation Plan: Intermediate Client Crate Architecture

## Overview

This implementation plan breaks down the work into two main phases: creating the rustconf-runtime crate with static runtime components, and modifying rustconf to support modular code generation that imports from rustconf-runtime.

## Tasks

- [x] 1. Create rustconf-runtime crate structure
  - Create new crate directory rustconf-runtime/
  - Set up Cargo.toml with appropriate dependencies and features
  - Create src/ directory structure (lib.rs, transport.rs, error.rs, adapters/)
  - _Requirements: 12.1, 12.2, 12.3, 12.5_

- [x] 2. Implement core runtime types in rustconf-runtime
  - [x] 2.1 Implement HttpMethod, HttpRequest, and HttpResponse types
    - Move existing types from generator to rustconf-runtime
    - Add serde derives and documentation
    - _Requirements: 12.1_
  
  - [x] 2.2 Implement HttpTransport trait
    - Define async trait with execute method
    - Add documentation and usage examples
    - _Requirements: 12.1, 4.4_
  
  - [x] 2.3 Implement RpcError type
    - Move existing error type from generator
    - Implement Display and Error traits
    - _Requirements: 12.1, 10.3_
  
  - [x] 2.4 Implement RestconfClient
    - Move existing client implementation from generator
    - Support interceptor pattern
    - _Requirements: 12.1, 4.5_
  
  - [ ]* 2.5 Write unit tests for core runtime types
    - Test RestconfClient construction and interceptor chaining
    - Test error type conversions
    - _Requirements: 12.1_

- [x] 3. Implement transport adapters in rustconf-runtime
  - [x] 3.1 Implement reqwest adapter with feature gate
    - Move existing reqwest adapter from generator
    - Feature-gate with #[cfg(feature = "reqwest")]
    - _Requirements: 12.2, 4.1_
  
  - [x] 3.2 Implement hyper adapter with feature gate
    - Move existing hyper adapter from generator
    - Feature-gate with #[cfg(feature = "hyper")]
    - _Requirements: 12.2, 4.2_
  
  - [ ]* 3.3 Write integration tests for transport adapters
    - Test reqwest adapter with mock server
    - Test hyper adapter with mock server
    - _Requirements: 12.2_

- [x] 4. Add modular_output support to RustconfBuilder
  - [x] 4.1 Add modular_output field to GeneratorConfig
    - Add boolean field with default false
    - Add builder method modular_output()
    - _Requirements: 3.3, 7.1_
  
  - [ ]* 4.2 Write property test for output directory flexibility
    - **Property 1: Output Directory Flexibility**
    - **Validates: Requirements 3.6, 7.1**
  
  - [ ]* 4.3 Write unit tests for builder configuration
    - Test modular_output() method
    - Test output_dir() with src/ paths
    - _Requirements: 3.3, 7.1_

- [x] 5. Implement modular code generation in CodeGenerator
  - [x] 5.1 Implement generate_modular() method
    - Add logic to generate multiple files instead of single file
    - Call individual file generators (mod, types, operations, validation)
    - _Requirements: 2.1, 7.3_
  
  - [x] 5.2 Implement generate_mod_file() method
    - Generate mod.rs with submodule declarations
    - Add re-exports for rustconf-runtime types
    - _Requirements: 7.3, 12.4_
  
  - [x] 5.3 Implement generate_types_file() method
    - Generate types.rs with only YANG-derived types
    - Remove HTTP abstraction generation
    - Add imports from rustconf-runtime
    - _Requirements: 2.3, 12.4_
  
  - [x] 5.4 Implement generate_operations_file() method
    - Generate operations.rs with RPC functions
    - Use rustconf-runtime types instead of generating them
    - _Requirements: 2.4, 12.4_
  
  - [x] 5.5 Implement generate_validation_file() method
    - Generate validation.rs with validation types and logic
    - _Requirements: 2.5, 10.1_
  
  - [ ]* 5.6 Write property test for generated code completeness
    - **Property 2: Generated Code Completeness**
    - **Validates: Requirements 2.3, 2.4, 2.5, 5.2**
  
  - [ ]* 5.7 Write property test for modular file organization
    - **Property 11: Modular File Organization**
    - **Validates: Requirements 2.1, 7.3**

- [x] 6. Remove HTTP abstraction generation from single-file mode
  - [x] 6.1 Modify generate_single_file() to import from rustconf-runtime
    - Add rustconf-runtime imports at top of generated file
    - Remove generation of HttpTransport, RestconfClient, RpcError
    - Remove generation of transport adapters
    - _Requirements: 12.4_
  
  - [ ]* 6.2 Write property test for runtime type imports
    - **Property 12: Runtime Type Imports**
    - **Validates: Requirements 12.4**

- [ ] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Update operations generator to use rustconf-runtime types
  - [x] 8.1 Modify OperationsGenerator to not generate HTTP abstractions
    - Remove generate_http_method(), generate_http_request(), etc.
    - Update generate_operations_module() to use rustconf-runtime imports
    - _Requirements: 12.4_
  
  - [ ]* 8.2 Write property test for API compatibility
    - **Property 6: API Compatibility Across Generation Methods**
    - **Validates: Requirements 9.1, 9.2, 9.3, 9.4**

- [x] 9. Update validation generator for modular output
  - [x] 9.1 Create generate_validation_file() method
    - Extract validation type generation into separate file
    - Ensure ValidationError type is included
    - _Requirements: 10.1_
  
  - [x] 9.2 Write property test for validation logic preservation
    - **Property 7: Validation Logic Preservation**
    - **Validates: Requirements 10.1, 10.2**
  
  - [ ]* 9.3 Write property test for validation error messages
    - **Property 9: Validation Error Messages**
    - **Validates: Requirements 10.5**

- [x] 10. Create integration test for intermediate crate pattern
  - [x] 10.1 Create test intermediate crate in tests/fixtures/
    - Set up Cargo.toml with rustconf build-dep and rustconf-runtime dep
    - Create build.rs that generates to src/generated/
    - Add sample YANG file
    - _Requirements: 3.3, 7.1_
  
  - [x] 10.2 Create test end-user project
    - Set up Cargo.toml depending only on test intermediate crate
    - Write code that uses the intermediate crate API
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [x]* 10.3 Write integration test that builds both crates
    - Test that intermediate crate builds successfully
    - Test that end-user project builds without rustconf
    - Test that end-user project has no build.rs
    - _Requirements: 1.2, 1.3, 2.2_
  
  - [x] 10.4 Fix code generation issues discovered in integration testing
    - Add missing serde imports (Serialize, Deserialize) to operations.rs
    - Add missing HttpRequest and HttpMethod imports to operations.rs
    - Add urlencoding dependency to intermediate crate or use percent_encode function
    - Ensure all generated RPC input/output structs have proper serde derives
    - _Requirements: 2.4, 9.1, 12.4_

- [ ] 11. Update documentation and examples
  - [ ] 11.1 Update rustconf README with intermediate crate pattern
    - Add section explaining the pattern
    - Provide template build.rs
    - _Requirements: 11.1, 11.2_
  
  - [ ] 11.2 Create example intermediate crate
    - Create examples/intermediate-client/ directory
    - Set up as a complete intermediate crate example
    - _Requirements: 9.2, 11.1_
  
  - [ ] 11.3 Update rustconf-runtime README
    - Document all public types and traits
    - Explain feature flags
    - _Requirements: 9.4, 12.1, 12.2_
  
  - [ ] 11.4 Update existing examples to use rustconf-runtime
    - Update examples/interface-config to use rustconf-runtime
    - Update examples/restful-basic to use rustconf-runtime
    - Update examples/restful-custom-transport to use rustconf-runtime
    - Update examples/restful-interceptor to use rustconf-runtime
    - Update examples/restful-error-handling to use rustconf-runtime
    - Re-add examples to Cargo workspace after updates
    - _Requirements: 12.4_

- [ ] 12. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- The rustconf-runtime crate should be created first as it's a dependency for testing
- Modular generation is the recommended approach for intermediate crates
- Single-file generation remains supported for backward compatibility
