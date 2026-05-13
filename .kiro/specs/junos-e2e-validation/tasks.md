# Implementation Plan: JunOS End-to-End Validation

## Overview

This plan implements the E2E validation layer that exercises the rustconf-generated Juniper RESTCONF client against a live cRPD container. It adds a Makefile-based local runner, a GitHub Actions CI job, Junos-specific test fixtures, and a comprehensive E2E test file organized into smoke, CRUD, schema conformance, and error path categories. All tasks build on the existing `tests/integration/` infrastructure (`TestHarness`, `FixtureManager`, `ConformanceReporter`, generated Junos types).

## Tasks

- [x] 1. Create reports directory and update .gitignore
  - [x] 1.1 Create `tests/integration/reports/.gitkeep` to ensure the reports directory exists in the repo
    - Create the directory with a `.gitkeep` placeholder file
    - _Requirements: 10.4_
  - [x] 1.2 Add `tests/integration/reports/` exclusion to `.gitignore` (except `.gitkeep`)
    - Add lines to `.gitignore`: `tests/integration/reports/*` and `!tests/integration/reports/.gitkeep`
    - _Requirements: 10.4_

- [x] 2. Create Junos-specific test fixture files
  - [x] 2.1 Create `tests/integration/fixtures/junos-interfaces.json`
    - Define fixture with `resource_path` targeting `junos-conf-interfaces` YANG module
    - Include interface configuration with unit and inet family address matching cRPD's expected schema
    - Use the existing `FixtureDefinition` format (`resource_path` + `data` fields)
    - _Requirements: 4.1, 4.5, 4.6_
  - [x] 2.2 Create `tests/integration/fixtures/junos-system.json`
    - Define fixture with `resource_path` targeting `junos-conf-system` YANG module
    - Include host-name, domain-name, and name-server configuration
    - _Requirements: 4.2, 4.3, 4.6_
  - [x] 2.3 Create `tests/integration/fixtures/junos-routing-options.json`
    - Define fixture with `resource_path` targeting `junos-conf-routing-options` YANG module
    - Include static route configuration with next-hop
    - _Requirements: 4.6_

- [x] 3. Implement test resource naming utility and smoke gate mechanism
  - [x] 3.1 Create test helper module with `e2e_resource_name` function and `SMOKE_PASSED` gate
    - Add `e2e_resource_name(category: &str) -> String` that generates unique names with format `e2e-{category}-{short_uuid}`
    - Add `static SMOKE_PASSED: AtomicBool` and `skip_unless_smoke_passed!()` macro
    - Place in `tests/integration/tests/e2e_helpers.rs` or inline in the test file's common section
    - _Requirements: 8.3, 3.5_
  - [ ]* 3.2 Write property test for resource name uniqueness (Property 6)
    - **Property 6: Unique test resource names never collide**
    - Generate 1000 names and assert all are distinct
    - **Validates: Requirements 8.3**

- [ ] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement E2E smoke tests
  - [x] 5.1 Create `tests/integration/tests/e2e_junos_tests.rs` with module structure and smoke tests
    - Create the file with `mod common;` import and internal modules: `smoke`, `crud`, `schema`, `errors`
    - Implement smoke tests: connectivity (GET /restconf returns 200 with valid JSON), authentication (Basic auth accepted), content-type verification (`application/yang-data+json` in response)
    - Set `SMOKE_PASSED = true` after all smoke tests pass
    - Each smoke test uses `skip_unless_integration!()` and `skip_unless_emulator!()` macros
    - Smoke tests must complete within 10 seconds after health check
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - [ ]* 5.2 Write property test for per-operation timeout enforcement (Property 7)
    - **Property 7: Per-operation timeout enforcement**
    - Test that operations exceeding 30s timeout are aborted with a timeout error (use mock/simulated delay)
    - **Validates: Requirements 9.2**

- [x] 6. Implement CRUD validation tests
  - [x] 6.1 Implement interface CRUD tests in the `crud` module
    - Create interface using generated `junos_conf_interfaces` types via PUT, verify with GET
    - Update interface unit configuration, verify change reflected in subsequent GET
    - Delete interface, verify absent in subsequent GET
    - Use `FixtureManager` for state isolation (apply/teardown)
    - Use `e2e_resource_name("crud")` for unique interface names
    - Use `skip_unless_smoke_passed!()` gate
    - _Requirements: 4.1, 4.3, 4.4, 4.5_
  - [x] 6.2 Implement system configuration CRUD tests in the `crud` module
    - Read system configuration from cRPD, deserialize into generated `junos_conf_system` types
    - Update hostname leaf via PUT, verify change in subsequent GET using generated types
    - Use `FixtureManager` with `junos-system.json` fixture for baseline state
    - _Requirements: 4.2, 4.3, 4.6_
  - [x] 6.3 Implement routing-options CRUD tests in the `crud` module
    - Create static route using generated `junos_conf_routing_options` types
    - Read routing table, verify route present
    - Delete route, verify absent
    - Use `junos-routing-options.json` fixture
    - _Requirements: 4.4, 4.6_
  - [ ]* 6.4 Write property test for CRUD write-read round-trip (Property 1)
    - **Property 1: CRUD write-read round-trip**
    - For valid configuration values conforming to YANG schema, PUT then GET should return equivalent deserialized value
    - **Validates: Requirements 4.1, 4.3, 4.5, 5.2**
  - [ ]* 6.5 Write property test for delete removes resource (Property 2)
    - **Property 2: Delete removes resource**
    - For any successfully created configuration element, DELETE then GET should return 404 or absence
    - **Validates: Requirements 4.4**

- [x] 7. Implement schema conformance tests
  - [x] 7.1 Implement schema key coverage tests in the `schema` module
    - GET configuration from cRPD, verify all JSON keys in response correspond to fields in generated Rust types
    - Log unknown keys as conformance warnings (not failures) using `ConformanceReporter`
    - Test across `junos-conf-interfaces` and `junos-conf-system` modules
    - _Requirements: 5.1, 5.5_
  - [x] 7.2 Implement Juniper-specific type deserialization tests in the `schema` module
    - Verify generated types correctly deserialize Juniper-specific YANG types (e.g., `junos:ipv4-prefix`)
    - Verify list entries with YANG list keys are correctly identified and deserialized
    - Serialize a configuration object and verify cRPD accepts it without schema validation errors
    - _Requirements: 5.2, 5.3, 5.4_
  - [ ]* 7.3 Write property test for schema key coverage (Property 3)
    - **Property 3: Schema key coverage**
    - For any JSON response from cRPD, every key should either map to a generated type field or be logged as a warning — never cause deserialization failure
    - **Validates: Requirements 5.1, 5.5**

- [ ] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Implement error path validation tests
  - [x] 9.1 Implement 404 and invalid path tests in the `errors` module
    - Send GET to non-existent RESTCONF path, verify generated client returns error with HTTP 404
    - Verify client does not panic on any error response
    - _Requirements: 7.1_
  - [x] 9.2 Implement invalid value and malformed JSON tests in the `errors` module
    - Send configuration with invalid value (e.g., invalid IP format), verify cRPD rejects and client surfaces error details
    - Send malformed JSON body, verify client maps cRPD error to correct `RpcError` variant
    - _Requirements: 7.2, 7.3_
  - [x] 9.3 Implement RESTCONF error structure parsing tests in the `errors` module
    - Trigger a cRPD error that returns `ietf-restconf:errors` structure
    - Verify generated client parses `error-type`, `error-tag`, and `error-message` fields
    - _Requirements: 7.4_
  - [ ]* 9.4 Write property test for non-existent paths return structured 404 (Property 4)
    - **Property 4: Non-existent paths return structured 404**
    - For any RESTCONF path not corresponding to an existing resource, client should return 404 and never panic
    - **Validates: Requirements 7.1**
  - [ ]* 9.5 Write property test for invalid inputs produce structured errors (Property 5)
    - **Property 5: Invalid inputs produce structured errors with RESTCONF error fields**
    - For any invalid input sent to cRPD, client should return structured error with HTTP status and parsed error fields
    - **Validates: Requirements 7.2, 7.3, 7.4**

- [x] 10. Implement conformance report generation
  - [x] 10.1 Add report generation logic to the E2E test suite
    - After all tests complete, generate `tests/integration/reports/e2e-conformance.txt` using `ConformanceReporter::generate_text_report()`
    - Generate `tests/integration/reports/e2e-junit.xml` using `ConformanceReporter::generate_junit_xml()`
    - Use emulator name `"Juniper cRPD (E2E)"` for the reporter
    - Include request/response details for failures, skip reasons for skipped modules
    - _Requirements: 10.1, 10.2, 10.3, 10.4_
  - [ ]* 10.2 Write property test for conformance report completeness (Property 8)
    - **Property 8: Conformance report completeness**
    - For any set of TestResult values, the generated report should contain every result grouped by YANG module with failure details and skip reasons
    - **Validates: Requirements 10.1, 10.2, 10.3**

- [x] 11. Create Makefile-based local runner
  - [x] 11.1 Create `tests/integration/Makefile` with all targets
    - Implement targets: `e2e`, `e2e-smoke`, `e2e-clean`, `check-runtime`, `check-image`, `load-image`, `help`
    - Auto-detect Docker vs Podman (try `docker info`, fall back to `podman info`)
    - Read `CRPD_IMAGE` from `config/crpd.toml` using grep/sed extraction
    - Set `RUSTCONF_INTEGRATION_TEST=1` and `RUSTCONF_CONTAINER_IMAGE` before invoking `cargo test`
    - Support `CRPD_TARBALL` variable for loading image from tarball
    - Support `TEST_FILTER` variable for filtering specific tests
    - Print clear error messages when Docker/Podman not installed or image not available
    - Ensure container cleanup on failure (trap-equivalent error handling)
    - Report total elapsed time at the end
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.3, 2.4, 9.4_

- [x] 12. Add GitHub Actions E2E validation job
  - [x] 12.1 Add `e2e-validation` job to `.github/workflows/build.yml`
    - Add new job that depends on `build` job, triggered only on pull requests to `main`
    - Set `timeout-minutes: 15` and `continue-on-error: true` (non-blocking)
    - Include steps: checkout, setup Rust, restore Docker image cache, load cRPD image, run E2E suite
    - Skip gracefully if cRPD image unavailable (check `docker image inspect` exit code)
    - Cache cRPD Docker image using `actions/cache` keyed on image tag from `config/crpd.toml`
    - Upload `tests/integration/reports/e2e-conformance.txt` and `tests/integration/reports/e2e-junit.xml` as artifacts (always, even on failure)
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_

- [x] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Tier 1 property tests (Properties 6, 7, 8) run without a cRPD emulator
- Tier 2 property tests (Properties 1–5) require `RUSTCONF_INTEGRATION_TEST=1` and a live cRPD instance
- The existing infrastructure (`TestHarness`, `FixtureManager`, `ConformanceReporter`, generated types) is used directly — no rebuilding
- All E2E tests use `skip_unless_integration!()` and `skip_unless_emulator!()` macros for graceful CI gating
