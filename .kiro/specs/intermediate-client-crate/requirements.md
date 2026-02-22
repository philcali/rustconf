# Requirements Document

## Introduction

This document specifies requirements for introducing an intermediate client crate architecture to simplify rustconf usage. The current approach requires every user to add rustconf as both a build-dependency and runtime dependency, run code generation in build.rs, and manually manage transport dependencies. The intermediate client crate architecture allows a crate author to create a dedicated client crate that performs code generation once and bundles their chosen runtime dependencies, eliminating build-time code generation for end users.

## Glossary

- **Rustconf**: The core library that parses YANG specifications and generates Rust code (build-dependency only)
- **Rustconf_Runtime**: A runtime-only crate containing HTTP abstractions, transport adapters, and error types
- **Intermediate_Client_Crate**: A user-created crate that uses rustconf at build-time and exposes generated code as a library
- **YANG_Schema**: A data modeling language specification (RFC 6020/7950)
- **RESTCONF**: A REST-like protocol for accessing YANG-modeled data (RFC 8040)
- **Build_Time_Generation**: Code generation that occurs during cargo build via build.rs
- **Transport_Layer**: HTTP client implementation (reqwest, hyper, or custom)
- **End_User**: Developer consuming the intermediate client crate
- **Crate_Author**: Developer creating and publishing the intermediate client crate

## Requirements

### Requirement 1: Simplified Dependency Management for End Users

**User Story:** As an end user, I want to add a single dependency to use RESTCONF bindings, so that I can avoid managing rustconf, build scripts, and transport dependencies.

#### Acceptance Criteria

1. WHEN an end user adds the intermediate client crate to their Cargo.toml, THE System SHALL provide all necessary runtime dependencies without requiring additional dependency declarations
2. WHEN an end user uses the intermediate client crate, THE System SHALL NOT require rustconf as a dependency
3. WHEN an end user uses the intermediate client crate, THE System SHALL NOT require a build.rs file
4. THE Intermediate_Client_Crate SHALL bundle the transport layer dependencies chosen by the crate author
5. THE Intermediate_Client_Crate SHALL bundle all required serialization dependencies (serde, serde_json)

### Requirement 2: Generated Code as Library Source

**User Story:** As an end user, I want to use RESTCONF bindings as a normal library, so that I can avoid build-time code generation and reduce compilation time.

#### Acceptance Criteria

1. THE Intermediate_Client_Crate SHALL expose generated Rust code as library source files
2. WHEN an end user compiles their project, THE System SHALL NOT perform YANG parsing or code generation
3. THE Intermediate_Client_Crate SHALL include all generated types, enums, and structs in its src/ directory
4. THE Intermediate_Client_Crate SHALL include all generated RPC operation functions in its src/ directory
5. THE Intermediate_Client_Crate SHALL include all generated validation logic in its src/ directory

### Requirement 3: Intermediate Crate Creation Pattern

**User Story:** As a crate author, I want to create an intermediate client crate using rustconf, so that I can publish RESTCONF bindings for end users.

#### Acceptance Criteria

1. THE Crate_Author SHALL be able to create a new Cargo project for the intermediate client crate
2. THE Crate_Author SHALL add rustconf as a build-dependency in the intermediate crate
3. THE Crate_Author SHALL create a build.rs that generates code into the src/ directory (not OUT_DIR)
4. THE Crate_Author SHALL choose and add their preferred transport dependencies (reqwest, hyper, custom)
5. THE Crate_Author SHALL be able to publish the intermediate crate with generated source files committed to version control
6. THE Rustconf SHALL support generating code to a specified output directory (not just OUT_DIR)

### Requirement 4: Transport Layer Flexibility

**User Story:** As a crate author, I want to choose which HTTP transport to bundle, so that I can control dependencies and provide the best fit for my use case.

#### Acceptance Criteria

1. THE Crate_Author SHALL be able to add reqwest as a dependency and include reqwest transport adapter
2. THE Crate_Author SHALL be able to add hyper as a dependency and include hyper transport adapter
3. THE Crate_Author SHALL be able to implement and bundle a custom HttpTransport implementation
4. THE Intermediate_Client_Crate SHALL expose the HttpTransport trait for end users who need custom behavior
5. THE Intermediate_Client_Crate SHALL expose the RestconfClient type that accepts any HttpTransport

### Requirement 5: Schema-Specific Client Crates

**User Story:** As a crate author, I want to create schema-specific client crates, so that end users get type-safe bindings for specific YANG models.

#### Acceptance Criteria

1. THE Crate_Author SHALL specify which YANG files to include in the intermediate client crate
2. THE Intermediate_Client_Crate SHALL include all types, operations, and validation from the specified YANG schemas
3. WHEN multiple YANG modules are specified, THE System SHALL generate code for all modules in the intermediate crate
4. THE System SHALL resolve YANG imports and include all dependent types
5. THE Intermediate_Client_Crate SHALL NOT include rustconf's YANG parsing or code generation capabilities at runtime

### Requirement 6: Feature Flags for Optional Functionality

**User Story:** As a crate author, I want to use Cargo features for optional functionality, so that end users can minimize dependencies.

#### Acceptance Criteria

1. THE Crate_Author SHALL be able to define Cargo features for optional transport implementations
2. THE Crate_Author SHALL be able to provide a default feature that includes their preferred transport
3. THE Intermediate_Client_Crate SHALL allow end users to disable default features if needed
4. THE Crate_Author SHALL document all available features in the intermediate crate's README
5. THE Intermediate_Client_Crate SHALL compile successfully with minimal features enabled

### Requirement 7: Code Generation to Source Directory

**User Story:** As a crate author, I want rustconf to generate code into my src/ directory, so that I can commit generated code and publish it as library source.

#### Acceptance Criteria

1. THE RustconfBuilder SHALL support an output directory parameter that can be set to src/ or any path
2. WHEN generating code to src/, THE System SHALL create properly formatted Rust source files
3. THE System SHALL generate a module structure that can be imported via lib.rs
4. THE System SHALL NOT require the include! macro when code is generated to src/
5. THE Crate_Author SHALL be able to commit generated source files to version control

### Requirement 8: Versioning and Publishing Strategy

**User Story:** As a crate author, I want to version and publish intermediate client crates, so that end users can depend on stable releases.

#### Acceptance Criteria

1. THE Crate_Author SHALL be able to set version numbers in the intermediate crate's Cargo.toml
2. THE Intermediate_Client_Crate SHALL follow semantic versioning conventions
3. WHEN YANG schemas change, THE Crate_Author SHALL regenerate code and publish a new version
4. THE Crate_Author SHALL be able to include metadata linking the crate version to source YANG schema versions
5. THE Intermediate_Client_Crate SHALL be publishable to crates.io or private registries

### Requirement 9: API Compatibility

**User Story:** As an end user, I want the intermediate client crate API to match the current generated code, so that the usage pattern is familiar and consistent.

#### Acceptance Criteria

1. THE Intermediate_Client_Crate SHALL expose the same public API as build-time generated code
2. THE RestconfClient type SHALL have the same constructor and methods
3. THE generated operation functions SHALL have the same signatures
4. THE generated types SHALL have the same field names and validation behavior
5. THE usage pattern SHALL be consistent whether using build-time generation or intermediate crate

### Requirement 9: Documentation and Examples

**User Story:** As a crate author, I want to include documentation and examples in my intermediate client crate, so that end users can quickly understand how to use it.

#### Acceptance Criteria

1. THE Crate_Author SHALL be able to add a README to the intermediate client crate
2. THE Crate_Author SHALL be able to include examples in the intermediate crate's examples/ directory
3. THE System SHALL generate rustdoc comments for all public types and functions
4. THE Crate_Author SHALL document how to use the bundled transport or provide custom transports
5. THE Crate_Author SHALL document all available features and their purposes

### Requirement 10: Validation and Error Handling Consistency

**User Story:** As an end user, I want the same validation and error handling as build-time generation, so that I get consistent behavior and type safety.

#### Acceptance Criteria

1. THE Intermediate_Client_Crate SHALL include all YANG constraint validation logic
2. THE Intermediate_Client_Crate SHALL validate data before serialization
3. THE Intermediate_Client_Crate SHALL return the same RpcError types as build-time generation
4. THE Intermediate_Client_Crate SHALL provide the same error messages and error handling patterns
5. WHEN validation fails, THE System SHALL provide descriptive error messages referencing YANG constraints

### Requirement 11: Build Script Pattern for Intermediate Crates

**User Story:** As a crate author, I want a clear pattern for setting up build.rs in my intermediate crate, so that I can generate code correctly.

#### Acceptance Criteria

1. THE Rustconf documentation SHALL provide a template build.rs for intermediate crate authors
2. THE build.rs pattern SHALL generate code to src/generated/ or similar directory
3. THE build.rs pattern SHALL configure rustconf to output library-ready source files
4. THE Crate_Author SHALL be able to run cargo build once to generate all source files
5. THE generated source files SHALL be suitable for committing to version control and publishing

### Requirement 12: Runtime Components Separation

**User Story:** As a crate author, I want static runtime components in a separate crate, so that I can avoid generating boilerplate code and benefit from explicit versioning of runtime behavior.

#### Acceptance Criteria

1. THE Rustconf_Runtime SHALL provide HttpTransport trait, RestconfClient, and RpcError types
2. THE Rustconf_Runtime SHALL provide transport adapter implementations (reqwest, hyper) as optional features
3. THE Rustconf_Runtime SHALL NOT include any build-time code generation capabilities
4. THE generated code SHALL import types from rustconf_runtime instead of generating them
5. THE Intermediate_Client_Crate SHALL depend on rustconf_runtime as a normal runtime dependency
6. WHEN rustconf_runtime is updated, THE Crate_Author SHALL be able to update the dependency version without regenerating code
