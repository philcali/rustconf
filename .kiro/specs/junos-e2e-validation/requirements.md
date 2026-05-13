# Requirements Document: JunOS End-to-End Validation

## Introduction

This document specifies requirements for end-to-end validation of the rustconf-generated RESTCONF client against a real Junos emulator (cRPD). The integration test harness infrastructure — emulator lifecycle management, fixture management, conformance reporting — already exists in `tests/integration/`. This spec focuses on the missing piece: actually running the generated client against a live cRPD instance and confirming it works, both on a developer's local machine and in GitHub Actions CI.

The existing infrastructure provides `TestHarness`, `FixtureManager`, `ConformanceReporter`, generated Juniper YANG client code, and CI gating via `RUSTCONF_INTEGRATION_TEST=1`. What it does not yet provide is:

- A working local developer workflow for spinning up cRPD and running tests against it
- A GitHub Actions workflow job that pulls the cRPD image and executes the integration test suite
- Validation that the generated Juniper-specific client types (from `junos-conf-*` YANG models) actually serialize/deserialize correctly against cRPD
- A documented, reproducible procedure for obtaining and loading the cRPD container image

This spec answers: "Does our generated client actually work against a real RESTCONF server?"

## Glossary

- **E2E_Validation_Suite**: The subset of integration tests that exercise the generated Juniper client against a live cRPD instance
- **cRPD**: Juniper's containerized Routing Protocol Daemon, a lightweight Junos instance that exposes RESTCONF
- **Local_Runner**: A developer's workstation with Docker or Podman installed, capable of running cRPD locally
- **CI_Runner**: A GitHub Actions runner configured to pull and run the cRPD container image
- **Generated_Junos_Client**: The Rust client code generated from Juniper YANG models in `tests/integration/src/generated/`
- **Smoke_Test**: A minimal test that verifies basic connectivity and authentication to the cRPD RESTCONF endpoint
- **CRUD_Validation**: Tests that perform Create, Read, Update, Delete operations on cRPD configuration via the Generated_Junos_Client
- **Schema_Conformance_Test**: A test that verifies the Generated_Junos_Client's type serialization matches what cRPD expects and returns
- **Runner_Script**: A shell script or Makefile target that automates the local developer workflow (pull image, start container, run tests, stop container)
- **CI_Workflow_Job**: A GitHub Actions job definition that executes the E2E_Validation_Suite

## Requirements

### Requirement 1: Local Developer Workflow

**User Story:** As a developer, I want a single command to run end-to-end validation against cRPD on my local machine, so that I can verify my changes work against a real device without manual setup.

#### Acceptance Criteria

1. THE Runner_Script SHALL provide a command that starts cRPD, waits for readiness, runs the E2E_Validation_Suite, and stops cRPD in a single invocation
2. WHEN the cRPD container image is not available locally, THE Runner_Script SHALL print instructions for obtaining the image and exit with a non-zero status
3. WHEN Docker or Podman is not installed, THE Runner_Script SHALL detect the absence and print a clear error message
4. THE Runner_Script SHALL support both Docker and Podman as container runtimes without requiring configuration changes
5. WHEN the E2E_Validation_Suite completes, THE Runner_Script SHALL stop and remove the cRPD container regardless of test pass or fail status
6. THE Runner_Script SHALL set the `RUSTCONF_INTEGRATION_TEST=1` environment variable automatically before invoking tests

### Requirement 2: cRPD Container Image Management

**User Story:** As a developer, I want clear documentation and tooling for obtaining the cRPD container image, so that I can set up my local environment without guesswork.

#### Acceptance Criteria

1. THE E2E_Validation_Suite SHALL document the required cRPD image name and tag in a single configuration file (`tests/integration/config/crpd.toml`)
2. WHEN the cRPD image requires authentication to pull (e.g., from a private registry), THE documentation SHALL specify the registry URL and authentication method
3. THE E2E_Validation_Suite SHALL support loading a cRPD image from a local tarball via `docker load` as an alternative to registry pulls
4. WHEN the configured cRPD image tag does not match the available local image, THE Runner_Script SHALL warn the developer about the version mismatch

### Requirement 3: Smoke Test Validation

**User Story:** As a developer, I want a fast smoke test that confirms basic connectivity to cRPD, so that I can quickly tell if the emulator is working before running the full suite.

#### Acceptance Criteria

1. WHEN the cRPD container is running, THE Smoke_Test SHALL verify that an HTTP GET to the RESTCONF root (`/restconf`) returns a 200 status with valid JSON
2. WHEN the cRPD container is running, THE Smoke_Test SHALL verify that Basic authentication with the configured credentials is accepted
3. WHEN the cRPD container is running, THE Smoke_Test SHALL verify that the `Content-Type` header in responses contains `application/yang-data+json`
4. THE Smoke_Test SHALL complete within 10 seconds after the health check passes
5. IF the Smoke_Test fails, THEN THE E2E_Validation_Suite SHALL skip all subsequent tests and report the smoke test failure

### Requirement 4: Junos-Specific CRUD Validation

**User Story:** As a developer, I want to verify that the generated Juniper client types can perform CRUD operations on cRPD configuration, so that I know the generated code is usable for real device management.

#### Acceptance Criteria

1. WHEN a test creates an interface configuration using the Generated_Junos_Client types, THE CRUD_Validation SHALL verify cRPD accepts the request and the interface appears in a subsequent GET
2. WHEN a test reads the system configuration from cRPD, THE CRUD_Validation SHALL verify the response deserializes into the generated `junos_conf_system` types without error
3. WHEN a test updates an existing configuration leaf (e.g., hostname), THE CRUD_Validation SHALL verify the change is reflected in a subsequent GET using the generated types
4. WHEN a test deletes a configuration element, THE CRUD_Validation SHALL verify the element is absent in a subsequent GET
5. WHEN the Generated_Junos_Client sends a request with Juniper-specific YANG extensions, THE CRUD_Validation SHALL verify cRPD does not reject the request due to serialization format issues
6. THE CRUD_Validation SHALL test at least three distinct Juniper YANG modules (e.g., `junos-conf-interfaces`, `junos-conf-system`, `junos-conf-routing-options`)

### Requirement 5: Schema Conformance Between Generated Types and cRPD

**User Story:** As a developer, I want to verify that the generated Rust types match cRPD's actual RESTCONF schema, so that I can trust the generated code handles real device responses correctly.

#### Acceptance Criteria

1. WHEN cRPD returns configuration data, THE Schema_Conformance_Test SHALL verify that all JSON keys in the response correspond to fields in the generated Rust types
2. WHEN the Generated_Junos_Client serializes a configuration object, THE Schema_Conformance_Test SHALL verify cRPD accepts the JSON without reporting schema validation errors
3. WHEN cRPD returns data with Juniper-specific YANG types (e.g., `junos:ipv4-prefix`), THE Schema_Conformance_Test SHALL verify the Generated_Junos_Client deserializes them into the correct Rust type
4. WHEN cRPD returns list entries with YANG list keys, THE Schema_Conformance_Test SHALL verify the Generated_Junos_Client correctly identifies and deserializes the key fields
5. IF the Generated_Junos_Client encounters an unknown JSON key in a cRPD response, THEN THE Schema_Conformance_Test SHALL log the key as a conformance warning rather than failing the test

### Requirement 6: GitHub Actions CI Integration

**User Story:** As a developer, I want the E2E validation to run automatically in CI on pull requests, so that regressions against real cRPD behavior are caught before merge.

#### Acceptance Criteria

1. THE CI_Workflow_Job SHALL run the E2E_Validation_Suite as a separate GitHub Actions job triggered on pull requests to `main`
2. WHEN the cRPD container image is available as a CI secret or artifact, THE CI_Workflow_Job SHALL pull and start the image before running tests
3. WHEN the cRPD container image is not available in CI, THE CI_Workflow_Job SHALL skip the E2E_Validation_Suite gracefully and report the skip reason in the job summary
4. THE CI_Workflow_Job SHALL enforce a total job timeout of 15 minutes to prevent hanging builds
5. THE CI_Workflow_Job SHALL cache the cRPD container image between runs to reduce pull times
6. WHEN the E2E_Validation_Suite fails in CI, THE CI_Workflow_Job SHALL upload the conformance report as a build artifact for debugging
7. THE CI_Workflow_Job SHALL be configured as a non-blocking check (failure does not prevent merge) until the suite is stable

### Requirement 7: Error Path Validation Against cRPD

**User Story:** As a developer, I want to verify that the generated client handles cRPD error responses correctly, so that error paths in production code work as expected.

#### Acceptance Criteria

1. WHEN a test sends a request to a non-existent RESTCONF path on cRPD, THE E2E_Validation_Suite SHALL verify the Generated_Junos_Client returns an error with HTTP 404 status
2. WHEN a test sends a configuration with an invalid value (e.g., invalid IP address format), THE E2E_Validation_Suite SHALL verify cRPD rejects it and the Generated_Junos_Client surfaces the error details
3. WHEN a test sends a request with a malformed JSON body, THE E2E_Validation_Suite SHALL verify the Generated_Junos_Client maps the cRPD error response to the correct `RpcError` variant
4. WHEN cRPD returns a RESTCONF error with `ietf-restconf:errors` structure, THE E2E_Validation_Suite SHALL verify the Generated_Junos_Client parses the error-type, error-tag, and error-message fields

### Requirement 8: Test Data Isolation

**User Story:** As a developer, I want each E2E test to run in isolation without affecting other tests, so that test results are deterministic and reproducible.

#### Acceptance Criteria

1. WHEN a test modifies cRPD configuration, THE E2E_Validation_Suite SHALL restore the original configuration after the test completes
2. WHEN a test fails mid-execution, THE E2E_Validation_Suite SHALL still attempt to restore the original configuration
3. THE E2E_Validation_Suite SHALL use unique resource names (e.g., interface names with test-specific prefixes) to avoid collisions between parallel test runs
4. WHEN the E2E_Validation_Suite starts, THE E2E_Validation_Suite SHALL verify cRPD is in a known baseline state before executing any tests

### Requirement 9: Performance and Timeout Constraints

**User Story:** As a developer, I want the E2E validation to complete in a reasonable time, so that it does not block my development workflow.

#### Acceptance Criteria

1. THE E2E_Validation_Suite SHALL complete all tests within 5 minutes after the cRPD health check passes
2. WHEN a single test operation does not receive a response within 30 seconds, THE E2E_Validation_Suite SHALL abort that test and mark it as timed out
3. WHEN the cRPD container does not become healthy within 120 seconds of starting, THE E2E_Validation_Suite SHALL abort and report a startup timeout
4. THE Runner_Script SHALL report the total elapsed time for the E2E validation run

### Requirement 10: Conformance Reporting for E2E Results

**User Story:** As a developer, I want a clear report of which Juniper YANG modules and operations passed or failed against cRPD, so that I can identify gaps in the generated code.

#### Acceptance Criteria

1. WHEN the E2E_Validation_Suite completes, THE E2E_Validation_Suite SHALL produce a conformance report listing each tested Juniper YANG module and operation with its result
2. WHEN a test fails, THE conformance report SHALL include the RESTCONF request sent, the cRPD response received, and the expected behavior
3. WHEN a Juniper YANG module is skipped due to code generation issues, THE conformance report SHALL note the module and the reason for skipping
4. THE conformance report SHALL be written to a file (`tests/integration/reports/e2e-conformance.txt`) for local runs and uploaded as an artifact in CI

