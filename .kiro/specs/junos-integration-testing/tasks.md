# Implementation Plan: JunOS Integration Testing

## Overview

Implement an integration testing harness for rustconf that validates generated RESTCONF client and server code against live network OS emulators. The harness lives in a new workspace crate (`tests/integration/`) and provides emulator lifecycle management, fixture-based test state, conformance reporting, and CI gating. Tasks are ordered to build foundational types first, then emulator management, then test utilities, and finally the test suites themselves.

## Tasks

- [x] 1. Set up integration test crate and core types
  - [x] 1.1 Create the `tests/integration/` workspace crate
    - Create `tests/integration/Cargo.toml` with dependencies: `testcontainers` (0.23), `tokio` (async runtime), `reqwest` (HTTP client), `serde`, `serde_json`, `thiserror`, `proptest`, `toml` (config parsing), `quick-xml` (JUnit XML), and workspace dependencies (`rustconf`, `rustconf-runtime`)
    - Add `tests/integration` to the workspace `members` list in root `Cargo.toml`
    - Create `tests/integration/src/lib.rs` re-exporting all harness modules
    - _Requirements: 1.1, 9.1_

  - [x] 1.2 Implement `HarnessError` unified error type
    - Create `tests/integration/src/error.rs`
    - Define `HarnessError` enum with variants: `StartupFailed`, `HealthCheckTimeout`, `EmulatorCrashed`, `FixtureApplyFailed`, `FixtureTeardownFailed`, `CodegenFailed`, `RestconfError`, `ContainerError`, `ConfigError`, `TestTimeout`, `Io`
    - Implement `From` conversions for `std::io::Error` and `RpcError`
    - _Requirements: 1.3, 1.5, 10.4_

  - [x] 1.3 Implement `HarnessConfig` runtime configuration
    - Create `tests/integration/src/config.rs`
    - Define `HarnessConfig` struct with fields: `emulator_type`, `container_image`, `restconf_port`, `username`, `password`, `health_timeout`, `test_timeout`, `skip_tls_verify`, `base_url`
    - Implement `from_env()` loading from environment variables (`RUSTCONF_EMULATOR_TYPE`, `RUSTCONF_CONTAINER_IMAGE`, `RUSTCONF_RESTCONF_PORT`, `RUSTCONF_USERNAME`, `RUSTCONF_PASSWORD`, `RUSTCONF_HEALTH_TIMEOUT_SECS`, `RUSTCONF_TEST_TIMEOUT_SECS`, `RUSTCONF_SKIP_TLS_VERIFY`, `RUSTCONF_BASE_URL`)
    - Implement `from_file(path)` loading from TOML with env var overrides
    - _Requirements: 1.6, 5.1, 5.2, 5.4, 9.3_

  - [ ]* 1.4 Write property test for HarnessConfig env round-trip (Property 1)
    - **Property 1: Configuration round-trip from environment**
    - Generate arbitrary valid env var combinations, set them, load via `from_env()`, verify all fields match
    - **Validates: Requirements 1.6**

- [x] 2. Implement EmulatorConfig trait and emulator implementations
  - [x] 2.1 Define the `EmulatorConfig` trait
    - Create `tests/integration/src/emulators/mod.rs`
    - Define `EmulatorConfig` trait with methods: `image_name()`, `restconf_port()`, `credentials()`, `restconf_base_path()`, `uses_tls()`, `yang_model_dir()`, `health_check_path()`, `vendor_name()`, `container_env()`
    - _Requirements: 11.1_

  - [x] 2.2 Implement `JunosCrpdConfig` emulator configuration
    - Create `tests/integration/src/emulators/crpd.rs`
    - Implement `EmulatorConfig` for `JunosCrpdConfig` with Juniper cRPD defaults (image, port 3000, root/Juniper1 credentials, `/restconf` base path, TLS enabled)
    - Support overriding defaults from `HarnessConfig`
    - _Requirements: 11.1, 11.4_

  - [x] 2.3 Implement `NetopeerConfig` emulator configuration
    - Create `tests/integration/src/emulators/netopeer2.rs`
    - Implement `EmulatorConfig` for `NetopeerConfig` with Netopeer2 defaults (sysrepo/netopeer2 image, port 6443, IETF YANG models)
    - _Requirements: 11.1, 11.2_

- [x] 3. Implement TestHarness with container lifecycle management
  - [x] 3.1 Implement `TestHarness` struct and container orchestration
    - Create `tests/integration/src/harness.rs`
    - Define `TestHarness` struct with fields: config, container, client, base_url, health_timeout, test_timeout
    - Implement `new(config)` constructor
    - Implement `start()` — use `testcontainers-rs` to start container with image, port mapping, and env vars from `EmulatorConfig`
    - Implement health check polling loop in `start()` — GET the health check path until 200 OK or timeout
    - Implement `stop()` — stop and remove the container
    - Implement `is_running()` state check
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [x] 3.2 Implement RESTCONF client initialization in TestHarness
    - Implement `client()` method that returns a configured `RestconfClient<ReqwestTransport>` pointed at the emulator
    - Configure HTTP Basic auth via `RequestInterceptor` using credentials from `EmulatorConfig`
    - Configure TLS certificate verification skip when `skip_tls_verify` is set
    - Implement `base_url()` accessor
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

  - [ ]* 3.3 Write property test for timeout enforcement (Property 12)
    - **Property 12: Per-test timeout enforcement**
    - Use a mock/delayed endpoint to verify that operations exceeding `test_timeout` return `HarnessError::TestTimeout`
    - **Validates: Requirements 9.4**

  - [ ]* 3.4 Write unit tests for TestHarness lifecycle
    - Test health check timeout produces `HarnessError::HealthCheckTimeout`
    - Test startup failure produces `HarnessError::StartupFailed`
    - Test `is_running()` state transitions
    - _Requirements: 1.2, 1.3, 1.5_

- [x] 4. Checkpoint — Ensure all core infrastructure compiles
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement FixtureManager for test state management
  - [x] 5.1 Implement `FixtureDefinition` and `FixtureManager`
    - Create `tests/integration/src/fixture.rs`
    - Define `FixtureDefinition` struct with `resource_path` and `data` fields
    - Define `AppliedFixture` struct tracking the resource path and original data for rollback
    - Implement `FixtureManager::new(client)` constructor
    - Implement `load_fixture(path)` to parse JSON fixture files into `FixtureDefinition`
    - Implement `apply(fixture)` — GET current state (save for rollback), PUT fixture data
    - Implement `teardown()` — restore all applied fixtures to original state via PUT/DELETE
    - Handle shared fixtures (apply once, run multiple tests, then teardown)
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [x] 5.2 Create example fixture JSON files
    - Create `tests/integration/fixtures/interfaces.json` with sample interface configuration
    - Create `tests/integration/fixtures/system.json` with sample system configuration
    - Ensure fixtures follow RFC 7951 JSON encoding format
    - _Requirements: 6.4_

  - [ ]* 5.3 Write property test for fixture JSON round-trip (Property 10)
    - **Property 10: Fixture JSON loading round-trip**
    - Generate arbitrary valid `FixtureDefinition` values, serialize to JSON file, load via `load_fixture()`, verify equivalence
    - **Validates: Requirements 6.4**

  - [ ]* 5.4 Write unit tests for FixtureManager error handling
    - Test fixture apply failure produces `HarnessError::FixtureApplyFailed`
    - Test fixture teardown failure produces `HarnessError::FixtureTeardownFailed`
    - Test loading invalid JSON fixture file
    - _Requirements: 6.3_

- [x] 6. Implement ConformanceReporter
  - [x] 6.1 Implement `ConformanceReporter` and report generation
    - Create `tests/integration/src/reporter.rs`
    - Define `TestResult`, `TestStatus` (Pass, Fail, Skip), and `TestDetails` structs
    - Define `ConformanceReporter` struct with results collection and emulator name
    - Implement `record(result)` to collect test results
    - Implement `generate_text_report()` — group results by `yang_module`, show pass/fail/skip markers, include expected/actual/request/response for failures, include skip reasons
    - Implement `generate_junit_xml()` — produce JUnit XML format for CI
    - Implement `summary()` — return (pass, fail, skip) counts
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 9.5_

  - [ ]* 6.2 Write property test for report completeness (Property 11)
    - **Property 11: Conformance report completeness and structure**
    - Generate arbitrary sets of `TestResult` values, record them, generate report, verify: all results present, grouped by module, failures include details, skips include reasons
    - **Validates: Requirements 8.2, 8.3, 8.4**

  - [ ]* 6.3 Write property test for multi-emulator report separation (Property 14)
    - **Property 14: Multi-emulator report separation**
    - Generate test results from N emulators, verify report contains exactly N sections, no cross-contamination of results
    - **Validates: Requirements 11.3**

- [x] 7. Checkpoint — Ensure all harness components compile and Tier 1 tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Set up YANG model directory and build-time code generation
  - [x] 8.1 Create YANG model directory structure
    - Create `tests/integration/yang/juniper/` for Juniper vendor models
    - Create `tests/integration/yang/ietf/` for IETF standard models (dependencies)
    - Add placeholder README files documenting how to populate with vendor YANG models from YangModels/yang
    - _Requirements: 2.1, 2.4_

  - [x] 8.2 Implement build-time code generation via `build.rs`
    - Create `tests/integration/build.rs` that invokes rustconf code generation using vendor YANG models
    - Generate client and server code into `tests/integration/src/generated/`
    - Handle unsupported YANG constructs by logging and skipping affected modules
    - Implement regeneration when YANG model files change (via `cargo:rerun-if-changed`)
    - _Requirements: 2.2, 2.3, 2.5_

- [x] 9. Implement emulator config TOML files and CI gating
  - [x] 9.1 Create reference emulator configuration files
    - Create `tests/integration/config/crpd.toml` with Juniper cRPD defaults
    - Create `tests/integration/config/netopeer2.toml` with Netopeer2 defaults
    - _Requirements: 11.4_

  - [x] 9.2 Implement CI gating logic
    - In `tests/integration/tests/common/mod.rs`, create shared test helpers
    - Implement `skip_unless_integration()` helper that checks `RUSTCONF_INTEGRATION_TEST=1`
    - Implement `skip_unless_emulator()` helper that verifies emulator container is available
    - When env var is not set, tests skip gracefully without failing the build
    - _Requirements: 9.1, 9.2, 9.3_

- [x] 10. Implement client integration test suite
  - [x] 10.1 Implement client integration tests for RESTCONF operations
    - Create `tests/integration/tests/client_tests.rs`
    - Write tests for GET operations: verify response deserializes into generated Rust types
    - Write tests for PUT/PATCH operations: verify emulator accepts request, verify config change reflected in subsequent GET
    - Write tests for RPC operations: verify emulator returns valid response matching YANG output schema
    - Write tests for URL construction: verify emulator resolves generated URLs to correct resources
    - Write tests for request body serialization: verify emulator accepts JSON encoding
    - Gate all tests on `RUSTCONF_INTEGRATION_TEST=1`
    - _Requirements: 3.1, 3.2, 3.3, 3.5, 3.6_

  - [ ]* 10.2 Write property test for GET deserialization (Property 3)
    - **Property 3: GET responses deserialize into generated types**
    - For RESTCONF resources on the emulator, GET via Generated_Client should deserialize successfully into generated Rust types satisfying YANG constraints
    - **Validates: Requirements 3.2, 7.2**

  - [ ]* 10.3 Write property test for data write-read round-trip (Property 2)
    - **Property 2: Data write-read round-trip**
    - For valid YANG-typed values and RESTCONF paths, PUT then GET should return equivalent data
    - **Validates: Requirements 3.3, 7.1, 7.4**

- [x] 11. Implement serialization round-trip and JSON field name tests
  - [x] 11.1 Implement serialization round-trip tests
    - Create `tests/integration/tests/roundtrip_tests.rs`
    - Write tests verifying write-read-compare round-trips for multiple YANG types
    - Write tests verifying YANG types with constraints (ranges, patterns, enumerations) deserialize correctly
    - Gate emulator-dependent tests on `RUSTCONF_INTEGRATION_TEST=1`
    - _Requirements: 7.1, 7.2, 7.4_

  - [ ]* 11.2 Write property test for RFC 7951 JSON field names (Property 4)
    - **Property 4: JSON serialization uses RFC 7951 field names**
    - Generate arbitrary Rust type instances, serialize to JSON, verify all field names are kebab-case with module prefixes per RFC 7951 (no snake_case)
    - **Validates: Requirements 7.3**

- [x] 12. Implement error scenario test suite
  - [x] 12.1 Implement error scenario tests
    - Create `tests/integration/tests/error_tests.rs`
    - Write tests for 404 on non-existent RESTCONF paths
    - Write tests for malformed request bodies surfacing emulator error details
    - Write tests for constraint-violating data (out-of-range values) rejection and error mapping
    - Write tests for unexpected HTTP status codes (no panic, structured error returned)
    - Gate emulator-dependent tests on `RUSTCONF_INTEGRATION_TEST=1`
    - _Requirements: 10.1, 10.2, 10.3, 10.4_

  - [ ]* 12.2 Write property test for invalid input error mapping (Property 5)
    - **Property 5: Invalid inputs produce correctly mapped errors**
    - Generate invalid inputs (malformed JSON, constraint violations), send to emulator, verify `RpcError` variant and status code match error category
    - **Validates: Requirements 3.4, 10.2, 10.3**

  - [ ]* 12.3 Write property test for no-panic on any HTTP status (Property 6)
    - **Property 6: No panic on any HTTP status code**
    - For arbitrary HTTP status codes (including 500, 502, 418), verify the Generated_Client never panics and always returns a structured `RpcError`
    - **Validates: Requirements 10.1, 10.4**

- [x] 13. Implement server conformance test suite
  - [x] 13.1 Implement server conformance tests
    - Create `tests/integration/tests/server_tests.rs`
    - Write tests sending same RESTCONF requests to both Generated_Server and Emulator
    - Compare response format: JSON keys, nesting, Content-Type headers
    - Verify error responses follow `ietf-restconf:errors` structure (error-type, error-tag, error-message)
    - Report structural differences as conformance warnings via `ConformanceReporter`
    - Gate all tests on `RUSTCONF_INTEGRATION_TEST=1`
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [ ]* 13.2 Write property test for server response conformance (Property 7)
    - **Property 7: Server response conformance**
    - For RESTCONF requests sent to both Generated_Server and Emulator, verify matching JSON key structure, nesting depth, and Content-Type headers
    - **Validates: Requirements 4.1, 4.2**

  - [ ]* 13.3 Write property test for server error format (Property 8)
    - **Property 8: Server error responses follow RESTCONF format**
    - For error conditions on Generated_Server, verify response body is valid `ietf-restconf:errors` JSON with error-type, error-tag, error-message
    - **Validates: Requirements 4.3**

- [x] 14. Checkpoint — Ensure all test suites compile and Tier 1 tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 15. Wire together: fixture apply-teardown round-trip and emulator YANG validation
  - [x] 15.1 Implement fixture apply-teardown integration tests
    - Add fixture apply-teardown round-trip tests to `tests/integration/tests/roundtrip_tests.rs`
    - Verify that applying a fixture and tearing it down restores original emulator state
    - Gate on `RUSTCONF_INTEGRATION_TEST=1`
    - _Requirements: 6.2_

  - [ ]* 15.2 Write property test for fixture apply-teardown (Property 9)
    - **Property 9: Fixture apply-teardown restores original state**
    - For any fixture and initial state, apply then teardown should restore original data at the resource path
    - **Validates: Requirements 6.2**

  - [ ]* 15.3 Write property test for emulator config YANG model validation (Property 13)
    - **Property 13: Emulator config selects correct YANG models**
    - For any `EmulatorConfig` implementation, verify `yang_model_dir()` exists and contains `.yang` files for that vendor
    - **Validates: Requirements 11.2**

- [x] 16. Multi-emulator conformance report integration
  - [x] 16.1 Wire multi-emulator test execution and report aggregation
    - Implement logic to run the test suite against multiple emulator configs sequentially
    - Aggregate results into per-emulator conformance reports via `ConformanceReporter`
    - Ensure each emulator's results are identified by `vendor_name` in the report
    - Output combined conformance report at end of test session
    - _Requirements: 11.3, 8.1_

- [x] 17. Final checkpoint — Ensure all tests pass and crate compiles cleanly
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- Tier 1 tests (Properties 1, 4, 6, 10, 11, 12, 14) run without an emulator
- Tier 2 tests (Properties 2, 3, 5, 7, 8, 9, 13) require `RUSTCONF_INTEGRATION_TEST=1` and an emulator container
