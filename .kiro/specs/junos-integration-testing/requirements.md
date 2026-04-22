# Requirements Document: JunOS Integration Testing

## Introduction

This document specifies requirements for an integration testing package that validates rustconf-generated RESTCONF client and server code against a real network OS emulator (JunOS or similar). Currently, rustconf has unit tests and client-server round-trip tests that operate entirely in-process using generated stubs and mock transports. While these verify code generation correctness, they do not validate protocol-level interoperability with real RESTCONF implementations.

This feature introduces an integration test harness that:
- Manages the lifecycle of a network OS emulator (e.g., JunOS vMX/vSRX in a container)
- Generates client and server code from vendor YANG models
- Executes RESTCONF operations against the live emulator
- Validates that generated types, serialization, URL construction, and error handling are compatible with real device behavior

The goal is to catch protocol-level mismatches, serialization quirks, and YANG interpretation differences that unit tests cannot detect.

## Glossary

- **Test_Harness**: The integration test framework that orchestrates emulator lifecycle, code generation, and test execution
- **Emulator**: A containerized network OS instance (e.g., JunOS vMX, vSRX, or cRPD) that exposes a RESTCONF API
- **Vendor_YANG_Model**: YANG schema files provided by the network OS vendor (e.g., Juniper YANG models)
- **Generated_Client**: A RESTCONF client produced by rustconf from Vendor_YANG_Model files
- **Generated_Server**: A RESTCONF server scaffold produced by rustconf from Vendor_YANG_Model files
- **Test_Suite**: A collection of integration tests grouped by RESTCONF operation type or YANG module
- **RESTCONF_Endpoint**: An HTTP endpoint on the Emulator that implements the RESTCONF protocol (RFC 8040)
- **Health_Check**: A probe that determines whether the Emulator RESTCONF_Endpoint is ready to accept requests
- **Test_Fixture**: Pre-configured device state established before a test runs
- **Conformance_Report**: A structured output summarizing which RESTCONF operations passed or failed against the Emulator

## Requirements

### Requirement 1: Emulator Lifecycle Management

**User Story:** As a developer, I want the Test_Harness to manage the Emulator lifecycle automatically, so that integration tests can run without manual setup.

#### Acceptance Criteria

1. WHEN a test session begins, THE Test_Harness SHALL start the Emulator as a container with RESTCONF enabled
2. WHEN the Emulator container is started, THE Test_Harness SHALL wait for the RESTCONF_Endpoint to pass a Health_Check before executing tests
3. WHEN the Health_Check does not pass within a configurable timeout, THE Test_Harness SHALL report a startup failure and skip all tests
4. WHEN a test session completes, THE Test_Harness SHALL stop and remove the Emulator container
5. IF the Emulator container crashes during a test session, THEN THE Test_Harness SHALL detect the failure and abort remaining tests with a clear error message
6. WHEN configuring the Emulator, THE Test_Harness SHALL support specifying the container image, RESTCONF port, and authentication credentials via environment variables or configuration files

### Requirement 2: YANG Model Management

**User Story:** As a developer, I want the Test_Harness to use vendor YANG models for code generation, so that the generated code matches the Emulator capabilities.

#### Acceptance Criteria

1. THE Test_Harness SHALL include a directory of Vendor_YANG_Model files corresponding to the target Emulator
2. WHEN preparing an integration test, THE Test_Harness SHALL invoke rustconf code generation using the Vendor_YANG_Model files
3. WHEN code generation encounters unsupported YANG constructs in a Vendor_YANG_Model, THE Test_Harness SHALL log the issue and skip tests for that module
4. THE Test_Harness SHALL support testing with multiple Vendor_YANG_Model files to cover different functional areas (interfaces, routing, system, etc.)
5. WHEN the Vendor_YANG_Model files are updated, THE Test_Harness SHALL regenerate client code before running tests

### Requirement 3: Client Integration Testing

**User Story:** As a developer, I want the Generated_Client to be tested against the live Emulator, so that I can verify RESTCONF operations work end-to-end.

#### Acceptance Criteria

1. WHEN a test calls an RPC operation on the Generated_Client, THE Test_Suite SHALL verify the Emulator returns a valid response matching the YANG output schema
2. WHEN a test performs a GET operation via the Generated_Client, THE Test_Suite SHALL verify the response deserializes into the generated Rust types without error
3. WHEN a test performs a PUT or PATCH operation via the Generated_Client, THE Test_Suite SHALL verify the Emulator accepts the request and the configuration change is reflected in a subsequent GET
4. WHEN a test sends a request with invalid data through the Generated_Client, THE Test_Suite SHALL verify the Emulator returns an appropriate RESTCONF error and the Generated_Client maps it to the correct RpcError variant
5. WHEN the Generated_Client constructs RESTCONF URLs, THE Test_Suite SHALL verify the Emulator resolves them to the correct resource
6. WHEN the Generated_Client serializes request bodies, THE Test_Suite SHALL verify the Emulator accepts the JSON encoding without deserialization errors

### Requirement 4: Server Conformance Testing

**User Story:** As a developer, I want the Generated_Server scaffolding to be validated against the same protocol expectations as the Emulator, so that server implementations are interoperable with standard RESTCONF clients.

#### Acceptance Criteria

1. WHEN a test sends a RESTCONF request to the Generated_Server, THE Test_Suite SHALL verify the response format matches the RESTCONF protocol (RFC 8040) as observed from the Emulator
2. WHEN the Generated_Server receives a request with the same input as the Emulator, THE Test_Suite SHALL verify the response structure (JSON keys, nesting, content-type headers) is compatible
3. WHEN the Generated_Server returns an error, THE Test_Suite SHALL verify the error response body follows the RESTCONF error structure (ietf-restconf:errors)
4. WHEN comparing Generated_Server responses with Emulator responses, THE Test_Suite SHALL report structural differences as conformance warnings

### Requirement 5: Authentication and Transport Configuration

**User Story:** As a developer, I want the Test_Harness to handle authentication and TLS configuration, so that tests can connect to the Emulator securely.

#### Acceptance Criteria

1. WHEN connecting to the Emulator, THE Test_Harness SHALL authenticate using credentials provided in configuration
2. WHEN the Emulator uses TLS with self-signed certificates, THE Test_Harness SHALL support disabling certificate verification for test environments
3. WHEN the Emulator requires HTTP Basic authentication, THE Test_Harness SHALL configure the Generated_Client with an appropriate RequestInterceptor
4. THE Test_Harness SHALL support configuring the Emulator RESTCONF base URL (scheme, host, port, path prefix)

### Requirement 6: Test Fixture Management

**User Story:** As a developer, I want tests to set up and tear down device configuration reliably, so that each test runs in a known state.

#### Acceptance Criteria

1. WHEN a test requires a specific device configuration, THE Test_Fixture SHALL apply the configuration to the Emulator before the test body runs
2. WHEN a test completes (whether pass or fail), THE Test_Fixture SHALL restore the Emulator to a clean baseline configuration
3. WHEN applying a Test_Fixture fails, THE Test_Harness SHALL skip the dependent test and report the fixture failure
4. THE Test_Harness SHALL support defining Test_Fixture configurations as YANG-modeled data in JSON files
5. WHEN multiple tests share the same Test_Fixture, THE Test_Harness SHALL apply the fixture once and run all related tests before teardown

### Requirement 7: Serialization Round-Trip Validation

**User Story:** As a developer, I want to verify that data round-trips correctly between the Generated_Client and the Emulator, so that serialization and deserialization are interoperable.

#### Acceptance Criteria

1. WHEN a test writes configuration via the Generated_Client and reads it back, THE Test_Suite SHALL verify the read data is equivalent to the written data
2. WHEN the Emulator returns data with YANG types that have constraints (ranges, patterns, enumerations), THE Test_Suite SHALL verify the Generated_Client deserializes them into the correct Rust types
3. WHEN the Generated_Client serializes a request body, THE Test_Suite SHALL verify the JSON field names use the YANG-defined names (kebab-case with module prefixes as required by RFC 7951)
4. FOR ALL tested YANG types, THE Test_Suite SHALL perform a write-read-compare round-trip to verify data integrity

### Requirement 8: Conformance Reporting

**User Story:** As a developer, I want a structured Conformance_Report after integration tests run, so that I can identify which YANG modules and operations are supported.

#### Acceptance Criteria

1. WHEN the test session completes, THE Test_Harness SHALL produce a Conformance_Report listing each tested operation and its result (pass, fail, skip)
2. WHEN a test fails, THE Conformance_Report SHALL include the expected and actual values along with the relevant RESTCONF request and response
3. WHEN an operation is skipped due to unsupported YANG constructs, THE Conformance_Report SHALL note the reason for skipping
4. THE Conformance_Report SHALL group results by YANG module for readability

### Requirement 9: CI Integration

**User Story:** As a developer, I want integration tests to run in CI when a suitable Emulator image is available, so that regressions are caught automatically.

#### Acceptance Criteria

1. WHEN the CI environment provides an Emulator container image, THE Test_Harness SHALL run the full integration Test_Suite
2. WHEN the CI environment does not have an Emulator image available, THE Test_Harness SHALL skip integration tests gracefully without failing the build
3. THE Test_Harness SHALL support gating integration test execution on an environment variable (e.g., RUSTCONF_INTEGRATION_TEST=1)
4. WHEN integration tests run in CI, THE Test_Harness SHALL enforce a configurable per-test timeout to prevent hanging builds
5. WHEN integration tests complete in CI, THE Test_Harness SHALL output results in a format compatible with standard test reporting (e.g., JUnit XML or cargo test output)

### Requirement 10: Error Scenario Testing

**User Story:** As a developer, I want integration tests that exercise error paths, so that I can verify the Generated_Client handles Emulator errors correctly.

#### Acceptance Criteria

1. WHEN a test sends a request to a non-existent RESTCONF path on the Emulator, THE Test_Suite SHALL verify the Generated_Client returns an RpcError with a 404 status
2. WHEN a test sends a malformed request body to the Emulator, THE Test_Suite SHALL verify the Generated_Client surfaces the Emulator error details
3. WHEN a test sends a request with constraint-violating data (e.g., out-of-range values), THE Test_Suite SHALL verify the Emulator rejects the request and the Generated_Client maps the error appropriately
4. WHEN the Emulator returns an unexpected HTTP status code, THE Test_Suite SHALL verify the Generated_Client does not panic and returns a structured error

### Requirement 11: Multi-Emulator Support

**User Story:** As a developer, I want the Test_Harness to support multiple emulator types, so that generated code can be validated across different vendor implementations.

#### Acceptance Criteria

1. THE Test_Harness SHALL define an Emulator configuration interface that abstracts vendor-specific details (container image, default credentials, YANG model paths, RESTCONF endpoint path)
2. WHEN a new emulator type is configured, THE Test_Harness SHALL use the corresponding Vendor_YANG_Model files for code generation
3. WHEN running the Test_Suite against multiple emulators, THE Conformance_Report SHALL include results per emulator type
4. THE Test_Harness SHALL provide at least one reference Emulator configuration (JunOS cRPD or equivalent)
