# Implementation Plan: rustconf

## Overview

This implementation plan breaks down the rustconf library into incremental coding tasks. The approach follows a bottom-up strategy: first building the YANG parser, then the code generator, then the build system integration, and finally the example module. Each task builds on previous work and includes testing to validate correctness early.

## Tasks

- [x] 1. Set up project structure and dependencies
  - Create Cargo workspace with library crate and example crate
  - Add dependencies: nom (parsing), serde (serialization), quote/syn (code generation), proptest (property testing)
  - Set up basic module structure: parser, generator, build modules
  - Configure rustfmt and clippy settings
  - Create GitHub workflow for CI/CD (build, test, clippy, rustfmt checks)
  - _Requirements: 8.2, 8.4_

- [x] 2. Implement YANG lexer and basic parser infrastructure
  - [x] 2.1 Create YANG token types and lexer
    - Define token enum for YANG keywords, identifiers, strings, numbers, operators
    - Implement lexer using nom to tokenize YANG input
    - Handle comments and whitespace
    - _Requirements: 1.1, 1.2_
  
  - [x] 2.2 Write property test for lexer
    - **Property 1: Valid YANG Parsing Success (partial - lexing phase)**
    - **Validates: Requirements 1.1, 1.2**
  
  - [x] 2.3 Implement error types with location tracking
    - Create ParseError enum with syntax and semantic variants
    - Implement Display for user-friendly error messages
    - Track line and column numbers during parsing
    - _Requirements: 1.3, 7.1_
  
  - [x] 2.4 Write unit tests for error reporting
    - Test that syntax errors include location information
    - Test error message formatting
    - _Requirements: 1.3, 7.1_

- [x] 3. Implement YANG AST data structures
  - [x] 3.1 Define core AST types
    - Create structs for YangModule, ModuleHeader, Import, TypeDef
    - Create enums for DataNode variants (Container, List, Leaf, etc.)
    - Create TypeSpec enum for YANG type system
    - Implement Debug and Clone for all AST types
    - _Requirements: 1.1, 1.2_
  
  - [x] 3.2 Implement constraint types
    - Create RangeConstraint, LengthConstraint, PatternConstraint types
    - Add validation methods for constraint checking
    - _Requirements: 2.1, 2.5_

- [ ] 4. Implement YANG statement parsers
  - [x] 4.1 Parse module header statements
    - Implement parsers for yang-version, namespace, prefix, import using nom
    - Build ModuleHeader from parsed statements
    - Implement YangParser::parse_string and parse_file methods
    - Handle basic module structure: module name { statements }
    - _Requirements: 1.1, 1.2, 1.4_
  
  - [x] 4.2 Write unit tests for module header parsing
    - Test parsing simple module with namespace and prefix
    - Test parsing module with yang-version statement
    - Test parsing module with import statements
    - Test error cases (missing required statements, invalid syntax)
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  
  - [x] 4.3 Parse typedef and grouping statements
    - Implement typedef parser with type specifications
    - Implement grouping parser with nested data nodes
    - Store definitions in YangModule for later resolution
    - _Requirements: 1.5_
  
  - [x] 4.4 Parse data definition statements
    - Implement parsers for container, list, leaf, leaf-list, choice, case
    - Handle nested structures recursively
    - Parse config, mandatory, default, description statements
    - Build DataNode enum variants from parsed statements
    - _Requirements: 1.1, 1.2, 2.2, 2.3, 2.4, 2.6, 2.7_
  
  - [x] 4.5 Parse type specifications
    - Implement parser for built-in types (int8, uint32, string, boolean, etc.)
    - Parse range, length, pattern constraints
    - Parse enumeration and union types
    - Build TypeSpec enum variants from parsed type statements
    - _Requirements: 2.1, 2.5_
  
  - [x] 4.6 Write unit tests for data definition parsing
    - Test parsing containers with nested children
    - Test parsing lists with keys
    - Test parsing leaves with various types and constraints
    - Test parsing choices with cases
    - Test error cases (invalid nesting, missing required fields)
    - _Requirements: 1.1, 1.2, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [ ]* 4.7 Write property test for statement parsing
    - **Property 1: Valid YANG Parsing Success (complete)**
    - **Validates: Requirements 1.1, 1.2**

- [ ] 5. Implement module resolution and semantic validation
  - [x] 5.1 Implement import resolution
    - Enhance YangParser to use search paths for finding imported modules
    - Recursively load imported modules using parse_file
    - Build module dependency graph to track imports
    - Store loaded modules to avoid duplicate parsing
    - _Requirements: 1.4, 7.4_
  
  - [x] 5.2 Write unit tests for import resolution
    - Test resolving imports from search paths
    - Test handling missing imported modules (UnresolvedImport error)
    - Test recursive import loading
    - Test import with revision dates
    - _Requirements: 1.4, 7.4_
  
  - [x] 5.3 Implement typedef and grouping expansion
    - Resolve typedef references to concrete types
    - Expand grouping uses into data nodes
    - Handle nested groupings and typedefs
    - Replace type references with expanded TypeSpec
    - _Requirements: 1.5_
  
  - [x] 5.4 Implement semantic validation
    - Validate all references are defined (typedefs, groupings, leafrefs)
    - Detect circular dependencies in imports and groupings
    - Validate type constraints are well-formed (range min < max, etc.)
    - Return SemanticError with descriptive messages
    - _Requirements: 7.2, 7.5_
  
  - [x] 5.5 Write unit tests for semantic validation
    - Test detection of undefined references
    - Test detection of circular dependencies
    - Test validation of malformed constraints
    - Test error messages include constraint details
    - _Requirements: 7.2, 7.5_
  
  - [ ]* 5.6 Write property tests for resolution
    - **Property 3: Import Resolution**
    - **Property 4: Grouping and Typedef Expansion**
    - **Property 23: Semantic Validation Errors**
    - **Validates: Requirements 1.4, 1.5, 7.2, 7.4, 7.5**

- [x] 6. Checkpoint - Ensure parser tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement code generator infrastructure
  - [x] 7.1 Implement basic code generation scaffolding
    - Implement CodeGenerator::generate to create GeneratedCode structure
    - Create file writing logic to output_dir
    - Generate basic module structure with use statements
    - Add file header comments with generation metadata
    - _Requirements: 9.1, 9.2_
  
  - [ ] 7.2 Implement naming convention conversion
    - Convert YANG identifiers to snake_case for fields/functions
    - Convert YANG identifiers to PascalCase for types
    - Handle Rust keyword escaping (type → type_, match → match_, etc.)
    - Create utility functions: to_snake_case, to_pascal_case, escape_keyword
    - _Requirements: 8.1_
  
  - [ ] 7.3 Write unit tests for naming conventions
    - Test kebab-case to snake_case conversion
    - Test kebab-case to PascalCase conversion
    - Test Rust keyword escaping
    - Test handling of special characters and numbers
    - _Requirements: 8.1_
  
  - [ ]* 7.4 Write property test for naming conventions
    - **Property 24: Naming Convention Conversion**
    - **Validates: Requirements 8.1**
  
  - [ ] 7.5 Implement code formatting utilities
    - Use quote crate for generating Rust token streams
    - Use prettyplease for formatting generated code
    - Create helper functions for common code patterns (struct generation, impl blocks)
    - _Requirements: 8.4_

- [ ] 8. Implement type generation
  - [ ] 8.1 Generate Rust types from YANG containers
    - Create struct definitions with fields for child nodes
    - Add #[derive(Debug, Clone, Serialize, Deserialize)] attributes
    - Generate rustdoc comments from YANG descriptions
    - Handle nested containers recursively
    - _Requirements: 2.2, 5.1, 6.5, 8.1_
  
  - [ ] 8.2 Write unit tests for container generation
    - Test simple container with leaf children
    - Test nested containers
    - Test container with description generating rustdoc
    - Test empty containers
    - _Requirements: 2.2, 6.5_
  
  - [ ] 8.3 Generate Rust types from YANG lists
    - Create struct for list items with key fields
    - Generate Vec<T> type alias for list collections
    - Ensure key fields are non-optional
    - _Requirements: 2.3_
  
  - [ ] 8.4 Generate Rust enums from YANG choices
    - Create enum with variants for each case
    - Add #[serde(rename_all = "kebab-case")] for proper serialization
    - Handle nested data nodes within cases
    - _Requirements: 2.4_
  
  - [ ] 8.5 Generate validated types for constrained leaves
    - Create newtype wrappers for range/pattern/length constraints
    - Implement validation in constructor methods (new, try_from)
    - Implement Serialize/Deserialize with validation
    - Return ValidationError for constraint violations
    - _Requirements: 2.1, 2.5, 5.3_
  
  - [ ] 8.6 Handle mandatory vs optional fields
    - Generate non-Option types for mandatory nodes
    - Generate Option<T> types for optional nodes
    - Add #[serde(skip_serializing_if = "Option::is_none")] for optional fields
    - _Requirements: 2.6, 2.7_
  
  - [ ] 8.7 Write unit tests for type generation
    - Test list generation with keys
    - Test choice/case enum generation
    - Test validated type generation with constraints
    - Test mandatory vs optional field handling
    - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [ ]* 8.8 Write property tests for type generation
    - **Property 5: Type Constraint Preservation**
    - **Property 6: Container to Struct Mapping**
    - **Property 7: List to Collection Mapping**
    - **Property 8: Choice to Enum Mapping**
    - **Property 9: Optionality Mapping**
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7**

- [ ] 9. Implement serialization support
  - [ ] 9.1 Generate serde attributes for JSON encoding
    - Add #[serde(rename = "...")] attributes for YANG naming conventions
    - Configure serde for RESTCONF JSON compliance (RFC 8040)
    - Handle namespace prefixes in JSON field names
    - _Requirements: 5.1, 5.2, 5.5_
  
  - [ ] 9.2 Write unit tests for JSON serialization
    - Test serialization of simple structs to JSON
    - Test field name conversion (snake_case to kebab-case)
    - Test optional field handling (skip_serializing_if)
    - Test namespace prefix handling
    - _Requirements: 5.1, 5.2, 5.5_
  
  - [ ]* 9.3 Generate XML serialization support (optional feature)
    - Add serde-xml-rs support when enable_xml is true
    - Generate XML namespace attributes
    - Add conditional compilation for XML features
    - _Requirements: 5.4, 5.5, 9.3_
  
  - [ ] 9.4 Generate validation in Deserialize implementations
    - Add custom deserialize implementations for constrained types
    - Return descriptive errors for constraint violations
    - Include violating value and constraint in error messages
    - _Requirements: 5.3, 7.3_
  
  - [ ] 9.5 Write unit tests for deserialization validation
    - Test deserialization of valid data succeeds
    - Test deserialization of invalid data returns errors
    - Test error messages include constraint details
    - Test range, length, and pattern constraint validation
    - _Requirements: 5.3, 7.3_
  
  - [ ]* 9.6 Write property tests for serialization
    - **Property 17: Required Trait Implementations**
    - **Property 18: RESTCONF JSON Compliance**
    - **Property 19: Deserialization Validation**
    - **Property 29: Serialization Round-Trip**
    - **Property 30: Deserialization Round-Trip**
    - **Validates: Requirements 5.1, 5.2, 5.3, 10.1, 10.2, 10.3, 10.4**

- [ ] 10. Implement RESTCONF operation generation
  - [ ] 10.1 Generate RPC function signatures
    - Create async functions for each YANG RPC
    - Generate input/output types from RPC definitions
    - Add proper error handling with Result types
    - Generate rustdoc comments from RPC descriptions
    - _Requirements: 4.1, 4.4_
  
  - [ ] 10.2 Generate notification types
    - Create structs for notification payloads
    - Add serde attributes for serialization
    - Generate rustdoc comments from notification descriptions
    - _Requirements: 4.2_
  
  - [ ] 10.3 Generate RESTCONF CRUD operations
    - Generate functions for GET, POST, PUT, PATCH, DELETE
    - Determine applicable operations based on config/state
    - Add proper error handling with Result types
    - _Requirements: 4.3, 4.4_
  
  - [ ] 10.4 Generate URL path construction
    - Create functions to build RESTCONF URLs from data tree paths
    - Handle key encoding in URLs (percent-encoding)
    - Generate path helpers for each data node
    - _Requirements: 4.5_
  
  - [ ] 10.5 Write unit tests for operation generation
    - Test RPC function generation with input/output
    - Test notification type generation
    - Test CRUD operation generation
    - Test URL path construction with keys
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  
  - [ ]* 10.6 Write property tests for operation generation
    - **Property 12: RPC Function Generation**
    - **Property 13: Notification Type Generation**
    - **Property 14: RESTCONF Operation Generation**
    - **Property 15: Operation Error Handling**
    - **Property 16: URL Path Generation**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

- [ ] 11. Checkpoint - Ensure generator tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 12. Implement build system integration
  - [ ] 12.1 Implement RustconfBuilder::generate method
    - Orchestrate parsing: create YangParser, add search paths, parse files
    - Orchestrate code generation: create CodeGenerator, generate code
    - Write generated files to output directory
    - Emit cargo:rerun-if-changed directives for all input files
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 9.1, 9.2_
  
  - [ ] 12.2 Implement configuration validation
    - Validate required fields are present (at least one YANG file)
    - Check paths exist and are accessible
    - Detect conflicting options
    - Return ConfigurationError with descriptive messages
    - _Requirements: 9.5_
  
  - [ ] 12.3 Implement error handling and reporting
    - Convert ParseError and GeneratorError to BuildError
    - Report errors through cargo build script protocol (println!)
    - Provide actionable error messages with file paths and line numbers
    - _Requirements: 3.5, 7.1, 7.2, 7.3_
  
  - [ ] 12.4 Write unit tests for build integration
    - Test successful generation with valid YANG files
    - Test error handling for missing files
    - Test error handling for invalid YANG syntax
    - Test configuration validation
    - Test cargo directive emission
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 9.5_
  
  - [ ]* 12.5 Write property tests for build integration
    - **Property 10: Build Directive Emission**
    - **Property 11: Build Error Reporting**
    - **Property 26: Feature Configuration**
    - **Property 27: Module Name Customization**
    - **Property 28: Configuration Validation**
    - **Validates: Requirements 3.4, 3.5, 9.3, 9.4, 9.5**

- [ ] 13. Create example module
  - [ ] 13.1 Create example YANG specification
    - Write interface-config.yang demonstrating common constructs
    - Include containers (interface, config, state)
    - Include lists (interfaces with name key)
    - Include leaves with constraints (mtu with range, enabled boolean)
    - Include choices (address-type: ipv4 or ipv6)
    - Include RPCs (reset-interface, get-statistics)
    - Use realistic networking domain
    - _Requirements: 6.2_
  
  - [ ] 13.2 Create example build.rs
    - Demonstrate RustconfBuilder usage with interface-config.yang
    - Show configuration options (search_path, output_dir, enable_validation)
    - Include proper error handling
    - Emit cargo directives
    - _Requirements: 6.1, 6.3_
  
  - [ ] 13.3 Create example application code
    - Demonstrate using generated types (Interface, InterfaceConfig)
    - Show serialization/deserialization with serde_json
    - Show RESTCONF operation usage (if implemented)
    - Show validation error handling
    - Include example data and assertions
    - _Requirements: 6.1, 6.4_
  
  - [ ] 13.4 Write integration test for example
    - Test that example builds successfully
    - Test that generated code compiles
    - Test that example runs without errors
    - _Requirements: 3.3, 6.1, 6.3, 6.4_
  
  - [ ]* 13.5 Write property test for example
    - **Property test: Example builds and runs successfully**
    - **Validates: Requirements 3.3, 6.1, 6.3, 6.4**

- [ ] 14. Implement documentation generation
  - [ ] 14.1 Generate rustdoc comments from YANG descriptions
    - Extract description statements from YANG nodes
    - Format as rustdoc comments (///) in generated code
    - Include examples where appropriate
    - Handle multi-line descriptions
    - Escape special characters in descriptions
    - _Requirements: 6.5_
  
  - [ ] 14.2 Write unit tests for documentation generation
    - Test rustdoc comment generation from descriptions
    - Test handling of multi-line descriptions
    - Test handling of special characters
    - Test that generated code includes documentation
    - _Requirements: 6.5_
  
  - [ ]* 14.3 Write property test for documentation
    - **Property 22: Documentation Generation**
    - **Validates: Requirements 6.5**

- [ ] 15. Code quality and polish
  - [ ] 15.1 Ensure generated code passes clippy
    - Run clippy on generated code from test cases
    - Fix any generator issues that produce warnings
    - Add #[allow(clippy::...)] where necessary for generated code patterns
    - _Requirements: 8.2_
  
  - [ ] 15.2 Ensure generated code uses Rust idioms
    - Verify Result types for errors
    - Verify Option types for optional values
    - Use iterators where applicable
    - Follow Rust API guidelines
    - _Requirements: 8.3_
  
  - [ ] 15.3 Write unit tests for code quality
    - Test that generated code compiles without warnings
    - Test that generated code follows Rust idioms
    - Test that generated code is properly formatted
    - _Requirements: 8.2, 8.3, 8.4_
  
  - [ ]* 15.4 Write property test for code quality
    - **Property 25: Code Quality Standards**
    - **Validates: Requirements 8.2, 8.3, 8.4**

- [ ] 16. Final checkpoint - Comprehensive testing
  - Run all unit tests and property tests
  - Build example project and verify it works
  - Run clippy and rustfmt on entire codebase
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties across all inputs
- Unit tests validate specific examples and edge cases
- The implementation follows a bottom-up approach: parser → generator → build system → examples
- Checkpoints ensure incremental validation at major milestones
- **IMPORTANT**: Always read files before editing them, as they may have been reformatted or modified between tasks. Test files in particular require careful review before modifications.
