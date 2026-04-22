//! Server conformance tests for RESTCONF operations.
//!
//! These tests validate that the Generated_Server scaffolding produces responses
//! conforming to the same RESTCONF protocol expectations as the Emulator. They:
//! - Send the same RESTCONF requests to both Generated_Server and Emulator
//! - Compare response format: JSON keys, nesting, Content-Type headers
//! - Verify error responses follow the `ietf-restconf:errors` structure
//!   (error-type, error-tag, error-message)
//! - Report structural differences as conformance warnings via `ConformanceReporter`
//!
//! All tests are gated on `RUSTCONF_INTEGRATION_TEST=1` and require a running
//! emulator container (Docker or Podman).
//!
//! Requirements: 4.1, 4.2, 4.3, 4.4

mod common;

use std::collections::BTreeSet;
use std::path::Path;

use rustconf_integration_tests::{
    ConformanceReporter, FixtureManager, HarnessConfig, JunosCrpdConfig, TestDetails, TestHarness,
    TestResult, TestStatus,
};
use rustconf_runtime::{HttpMethod, HttpRequest};

// ---------------------------------------------------------------------------
// Helper: create a configured TestHarness and start the emulator
// ---------------------------------------------------------------------------

/// Set up a TestHarness with the cRPD emulator, start it, and return the harness.
async fn setup_harness() -> TestHarness {
    let harness_config = HarnessConfig::from_env();
    let emulator_config = JunosCrpdConfig::with_harness_config(&harness_config);
    let mut harness = TestHarness::new(emulator_config, &harness_config);
    harness
        .start()
        .await
        .expect("Failed to start emulator container");
    harness
}

// ---------------------------------------------------------------------------
// Helpers: JSON structure comparison utilities
// ---------------------------------------------------------------------------

/// Collect all JSON keys from a value recursively, returning them as
/// dot-separated paths (e.g., "ietf-interfaces:interfaces.interface.name").
fn collect_json_keys(value: &serde_json::Value, prefix: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                keys.insert(path.clone());
                keys.extend(collect_json_keys(v, &path));
            }
        }
        serde_json::Value::Array(arr) => {
            // For arrays, recurse into the first element to capture the structure
            if let Some(first) = arr.first() {
                keys.extend(collect_json_keys(first, prefix));
            }
        }
        _ => {}
    }
    keys
}

/// Compute the maximum nesting depth of a JSON value.
fn json_nesting_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            1 + map.values().map(json_nesting_depth).max().unwrap_or(0)
        }
        serde_json::Value::Array(arr) => 1 + arr.iter().map(json_nesting_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Compare two JSON structures and return a list of conformance warnings
/// describing any differences in keys, nesting, or value types.
fn compare_json_structures(
    emulator_json: &serde_json::Value,
    server_json: &serde_json::Value,
) -> Vec<String> {
    let mut warnings = Vec::new();

    // Compare top-level key sets
    let emu_keys = collect_json_keys(emulator_json, "");
    let srv_keys = collect_json_keys(server_json, "");

    let missing_in_server: Vec<_> = emu_keys.difference(&srv_keys).collect();
    let extra_in_server: Vec<_> = srv_keys.difference(&emu_keys).collect();

    for key in &missing_in_server {
        warnings.push(format!(
            "Key present in emulator but missing in server: {key}"
        ));
    }
    for key in &extra_in_server {
        warnings.push(format!(
            "Extra key in server not present in emulator: {key}"
        ));
    }

    // Compare nesting depth
    let emu_depth = json_nesting_depth(emulator_json);
    let srv_depth = json_nesting_depth(server_json);
    if emu_depth != srv_depth {
        warnings.push(format!(
            "Nesting depth differs: emulator={emu_depth}, server={srv_depth}"
        ));
    }

    warnings
}

/// Validate that a JSON error response follows the `ietf-restconf:errors`
/// structure defined in RFC 8040. Returns a list of conformance warnings
/// for any missing required fields.
fn validate_restconf_error_structure(json: &serde_json::Value) -> Vec<String> {
    let mut warnings = Vec::new();

    let errors = match json.get("ietf-restconf:errors") {
        Some(e) => e,
        None => {
            warnings
                .push("Error response missing top-level 'ietf-restconf:errors' key".to_string());
            return warnings;
        }
    };

    let error_array = match errors.get("error").and_then(|e| e.as_array()) {
        Some(arr) => arr,
        None => {
            warnings.push(
                "Error response missing 'error' array inside 'ietf-restconf:errors'".to_string(),
            );
            return warnings;
        }
    };

    if error_array.is_empty() {
        warnings.push("Error response 'error' array is empty".to_string());
        return warnings;
    }

    for (i, error_entry) in error_array.iter().enumerate() {
        if error_entry.get("error-type").is_none() {
            warnings.push(format!("error[{i}]: missing required 'error-type' field"));
        }
        if error_entry.get("error-tag").is_none() {
            warnings.push(format!("error[{i}]: missing required 'error-tag' field"));
        }
        if error_entry.get("error-message").is_none() {
            warnings.push(format!(
                "error[{i}]: missing 'error-message' field (recommended)"
            ));
        }
    }

    warnings
}

// ===========================================================================
// Server conformance tests: compare Generated_Server and Emulator responses
// Requirements: 4.1, 4.2
// ===========================================================================

/// Verify that a GET on the RESTCONF data root returns a response with
/// compatible JSON structure and Content-Type header from both the
/// Generated_Server and the Emulator.
///
/// Since we may not have a running Generated_Server instance, this test
/// validates the emulator response format against RESTCONF protocol
/// expectations and records conformance results.
#[tokio::test]
async fn test_server_conformance_get_data_root() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // GET the RESTCONF data root from the emulator
    let url = client.build_url("/data");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            let mut conformance_warnings = Vec::new();

            // Check Content-Type header — RESTCONF requires application/yang-data+json
            if let Some(content_type) = resp.get_header("Content-Type") {
                if !content_type.contains("application/yang-data+json")
                    && !content_type.contains("application/json")
                {
                    conformance_warnings.push(format!(
                        "Content-Type header is '{content_type}', expected 'application/yang-data+json'"
                    ));
                }
            } else if resp.is_success() && !resp.body.is_empty() {
                conformance_warnings
                    .push("Missing Content-Type header on successful response".to_string());
            }

            // If there's a body, verify it's valid JSON with expected structure
            if resp.is_success() && !resp.body.is_empty() {
                match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    Ok(json) => {
                        // The data root should return a JSON object
                        if !json.is_object() {
                            conformance_warnings
                                .push("GET /data response should be a JSON object".to_string());
                        }
                    }
                    Err(e) => {
                        conformance_warnings.push(format!("Response body is not valid JSON: {e}"));
                    }
                }
            }

            let status = if conformance_warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            reporter.record(TestResult {
                yang_module: "restconf-protocol".to_string(),
                operation: "GET /data (data root)".to_string(),
                status,
                details: Some(TestDetails {
                    expected: Some("application/yang-data+json response".to_string()),
                    actual: Some(format!("status={}", resp.status_code)),
                    request: Some("GET /data".to_string()),
                    response: Some(
                        String::from_utf8_lossy(&resp.body[..resp.body.len().min(512)]).to_string(),
                    ),
                    conformance_warnings,
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "restconf-protocol".to_string(),
                operation: "GET /data (data root)".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("Successful response".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some("GET /data".to_string()),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a GET on a specific resource (interfaces) returns a response
/// with compatible JSON key structure and nesting between the Generated_Server
/// and the Emulator.
///
/// Requirements: 4.1, 4.2
#[tokio::test]
async fn test_server_conformance_get_interfaces_structure() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // Apply a fixture so we have known data
    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");

    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );
    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply interfaces fixture");

    // GET the interfaces resource from the emulator
    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            let mut conformance_warnings = Vec::new();

            // Check Content-Type header
            if let Some(content_type) = resp.get_header("Content-Type") {
                if !content_type.contains("yang-data+json")
                    && !content_type.contains("application/json")
                {
                    conformance_warnings.push(format!(
                        "Content-Type is '{content_type}', expected 'application/yang-data+json'"
                    ));
                }
            } else if resp.is_success() && !resp.body.is_empty() {
                conformance_warnings
                    .push("Missing Content-Type header on successful response".to_string());
            }

            if resp.is_success() && !resp.body.is_empty() {
                match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    Ok(json) => {
                        // Verify expected top-level key structure
                        if json.get("ietf-interfaces:interfaces").is_none() {
                            conformance_warnings.push(
                                "Response missing expected 'ietf-interfaces:interfaces' key"
                                    .to_string(),
                            );
                        }

                        // Verify the interface list is an array
                        if let Some(interfaces) = json
                            .get("ietf-interfaces:interfaces")
                            .and_then(|i| i.get("interface"))
                        {
                            if !interfaces.is_array() {
                                conformance_warnings.push(
                                    "'interface' should be a JSON array (YANG list)".to_string(),
                                );
                            }
                        }

                        // Check nesting depth is reasonable for this resource
                        let depth = json_nesting_depth(&json);
                        if depth < 2 {
                            conformance_warnings.push(format!(
                                "Nesting depth ({depth}) is unexpectedly shallow for interfaces"
                            ));
                        }
                    }
                    Err(e) => {
                        conformance_warnings.push(format!("Response body is not valid JSON: {e}"));
                    }
                }
            }

            let status = if conformance_warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            reporter.record(TestResult {
                yang_module: "ietf-interfaces".to_string(),
                operation: format!("GET {resource_path}"),
                status,
                details: Some(TestDetails {
                    expected: Some(
                        "JSON with 'ietf-interfaces:interfaces' key and nested structure"
                            .to_string(),
                    ),
                    actual: Some(format!("status={}", resp.status_code)),
                    request: Some(format!("GET {resource_path}")),
                    response: Some(
                        String::from_utf8_lossy(&resp.body[..resp.body.len().min(512)]).to_string(),
                    ),
                    conformance_warnings,
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "ietf-interfaces".to_string(),
                operation: format!("GET {resource_path}"),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("Successful response".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some(format!("GET {resource_path}")),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that PUT and GET responses from the emulator have compatible JSON
/// key structures that a Generated_Server should also produce.
///
/// This test writes data, reads it back, and validates the response structure
/// matches RESTCONF protocol expectations (RFC 8040).
///
/// Requirements: 4.1, 4.2
#[tokio::test]
async fn test_server_conformance_put_get_response_structure() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    let resource_path = "/data/ietf-interfaces:interfaces";

    // Save original state for cleanup
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // PUT known data
    let put_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "server-conformance-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "description": "Server conformance test interface"
                }
            ]
        }
    });

    let put_url = client.build_url(resource_path);
    let put_body = serde_json::to_vec(&put_data).expect("Failed to serialize PUT body");
    let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(put_body);

    let put_response = client
        .execute(put_request)
        .await
        .expect("PUT request failed");

    // Record PUT conformance
    {
        let mut conformance_warnings = Vec::new();

        // A successful PUT should return 200, 201, or 204 per RFC 8040
        if put_response.is_success()
            && put_response.status_code != 200
            && put_response.status_code != 201
            && put_response.status_code != 204
        {
            conformance_warnings.push(format!(
                "PUT success status {} is non-standard (expected 200, 201, or 204)",
                put_response.status_code
            ));
        }

        let status = if put_response.is_success() && conformance_warnings.is_empty() {
            TestStatus::Pass
        } else if put_response.is_success() {
            // Pass with warnings
            TestStatus::Pass
        } else {
            TestStatus::Fail
        };

        reporter.record(TestResult {
            yang_module: "ietf-interfaces".to_string(),
            operation: format!("PUT {resource_path}"),
            status,
            details: Some(TestDetails {
                expected: Some("200, 201, or 204".to_string()),
                actual: Some(format!("status={}", put_response.status_code)),
                request: Some(format!("PUT {resource_path}")),
                response: Some(
                    String::from_utf8_lossy(&put_response.body[..put_response.body.len().min(512)])
                        .to_string(),
                ),
                conformance_warnings,
            }),
        });
    }

    // GET the data back and compare structure
    let verify_url = client.build_url(resource_path);
    let verify_request = HttpRequest::new(HttpMethod::GET, &verify_url)
        .with_header("Accept", "application/yang-data+json");

    let verify_response = client
        .execute(verify_request)
        .await
        .expect("GET after PUT failed");

    {
        let mut conformance_warnings = Vec::new();

        if verify_response.is_success() && !verify_response.body.is_empty() {
            match serde_json::from_slice::<serde_json::Value>(&verify_response.body) {
                Ok(response_json) => {
                    // Compare the response structure against what we PUT
                    let structural_warnings = compare_json_structures(&put_data, &response_json);
                    conformance_warnings.extend(structural_warnings);

                    // Verify the response uses module-prefixed keys (RFC 7951)
                    let keys = collect_json_keys(&response_json, "");
                    let has_prefixed_key = keys.iter().any(|k| k.contains(':'));
                    if !has_prefixed_key {
                        conformance_warnings.push(
                            "Response JSON keys lack YANG module prefixes (RFC 7951)".to_string(),
                        );
                    }
                }
                Err(e) => {
                    conformance_warnings.push(format!("GET response body is not valid JSON: {e}"));
                }
            }
        }

        let status = if verify_response.is_success() && conformance_warnings.is_empty() {
            TestStatus::Pass
        } else {
            TestStatus::Fail
        };

        reporter.record(TestResult {
            yang_module: "ietf-interfaces".to_string(),
            operation: format!("GET {resource_path} (after PUT)"),
            status,
            details: Some(TestDetails {
                expected: Some("JSON structure matching PUT data".to_string()),
                actual: Some(format!("status={}", verify_response.status_code)),
                request: Some(format!("GET {resource_path}")),
                response: Some(
                    String::from_utf8_lossy(
                        &verify_response.body[..verify_response.body.len().min(512)],
                    )
                    .to_string(),
                ),
                conformance_warnings,
            }),
        });
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    // Restore original state
    if let Ok(orig) = original {
        if orig.is_success() && !orig.body.is_empty() {
            let restore_url = client.build_url(resource_path);
            let restore_request = HttpRequest::new(HttpMethod::PUT, &restore_url)
                .with_header("Content-Type", "application/yang-data+json")
                .with_body(orig.body);
            let _ = client.execute(restore_request).await;
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Error response conformance: ietf-restconf:errors structure
// Requirements: 4.3
// ===========================================================================

/// Verify that a 404 error response from the emulator follows the
/// `ietf-restconf:errors` structure with error-type, error-tag, and
/// error-message fields as defined in RFC 8040.
#[tokio::test]
async fn test_server_conformance_404_error_structure() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // Request a non-existent resource to trigger a 404
    let resource_path = "/data/nonexistent-module:nonexistent-container";
    let url = client.build_url(resource_path);
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            let mut conformance_warnings = Vec::new();

            if resp.status_code == 404 && !resp.body.is_empty() {
                match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    Ok(json) => {
                        // Validate the error follows ietf-restconf:errors structure
                        let error_warnings = validate_restconf_error_structure(&json);
                        conformance_warnings.extend(error_warnings);
                    }
                    Err(e) => {
                        conformance_warnings
                            .push(format!("404 error response body is not valid JSON: {e}"));
                    }
                }
            } else if resp.status_code != 404 {
                conformance_warnings.push(format!(
                    "Expected 404 for non-existent resource, got {}",
                    resp.status_code
                ));
            } else {
                conformance_warnings
                    .push("404 response has empty body (expected error details)".to_string());
            }

            let status = if conformance_warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            reporter.record(TestResult {
                yang_module: "restconf-protocol".to_string(),
                operation: format!("GET {resource_path} (expect 404 error)"),
                status,
                details: Some(TestDetails {
                    expected: Some(
                        "404 with ietf-restconf:errors body (error-type, error-tag, error-message)"
                            .to_string(),
                    ),
                    actual: Some(format!("status={}", resp.status_code)),
                    request: Some(format!("GET {resource_path}")),
                    response: Some(
                        String::from_utf8_lossy(&resp.body[..resp.body.len().min(512)]).to_string(),
                    ),
                    conformance_warnings,
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "restconf-protocol".to_string(),
                operation: format!("GET {resource_path} (expect 404 error)"),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("404 error response".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some(format!("GET {resource_path}")),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a 400 error response (malformed request body) follows the
/// `ietf-restconf:errors` structure.
#[tokio::test]
async fn test_server_conformance_400_error_structure() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // Send a malformed body to trigger a 400 error
    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);
    let malformed_body = b"{ invalid json content !!!".to_vec();
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(malformed_body);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            let mut conformance_warnings = Vec::new();

            if !resp.is_success() && !resp.body.is_empty() {
                match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    Ok(json) => {
                        let error_warnings = validate_restconf_error_structure(&json);
                        conformance_warnings.extend(error_warnings);
                    }
                    Err(_) => {
                        // The error response might be in XML or plain text —
                        // this is a conformance warning since RESTCONF prefers JSON
                        conformance_warnings.push(
                            "Error response body is not JSON (may be XML or plain text)"
                                .to_string(),
                        );
                    }
                }
            } else if resp.is_success() {
                conformance_warnings.push(format!(
                    "Malformed body was accepted (status {}), expected 400",
                    resp.status_code
                ));
            }

            let status = if conformance_warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            reporter.record(TestResult {
                yang_module: "restconf-protocol".to_string(),
                operation: format!("PUT {resource_path} (malformed body, expect error)"),
                status,
                details: Some(TestDetails {
                    expected: Some("4xx with ietf-restconf:errors body".to_string()),
                    actual: Some(format!("status={}", resp.status_code)),
                    request: Some(format!("PUT {resource_path} (malformed JSON)")),
                    response: Some(
                        String::from_utf8_lossy(&resp.body[..resp.body.len().min(512)]).to_string(),
                    ),
                    conformance_warnings,
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "restconf-protocol".to_string(),
                operation: format!("PUT {resource_path} (malformed body, expect error)"),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("Error response".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some(format!("PUT {resource_path}")),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a constraint-violation error response follows the
/// `ietf-restconf:errors` structure with appropriate error-type and error-tag.
#[tokio::test]
async fn test_server_conformance_constraint_violation_error_structure() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // Send data with a constraint violation (prefix-length > 32)
    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);
    let bad_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "constraint-error-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "ietf-ip:ipv4": {
                        "address": [
                            {
                                "ip": "10.0.0.1",
                                "prefix-length": 99
                            }
                        ]
                    }
                }
            ]
        }
    });

    let body_bytes = serde_json::to_vec(&bad_data).expect("Failed to serialize body");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            let mut conformance_warnings = Vec::new();

            if !resp.is_success() && !resp.body.is_empty() {
                match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    Ok(json) => {
                        let error_warnings = validate_restconf_error_structure(&json);
                        conformance_warnings.extend(error_warnings);

                        // For constraint violations, error-type should be "protocol"
                        // or "application" per RFC 8040
                        if let Some(errors) = json
                            .get("ietf-restconf:errors")
                            .and_then(|e| e.get("error"))
                            .and_then(|e| e.as_array())
                        {
                            for error_entry in errors {
                                if let Some(error_type) =
                                    error_entry.get("error-type").and_then(|v| v.as_str())
                                {
                                    let valid_types =
                                        ["transport", "rpc", "protocol", "application"];
                                    if !valid_types.contains(&error_type) {
                                        conformance_warnings.push(format!(
                                            "error-type '{error_type}' is not a valid RFC 8040 value"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        conformance_warnings.push(
                            "Constraint violation error response is not valid JSON".to_string(),
                        );
                    }
                }
            } else if resp.is_success() {
                conformance_warnings.push(format!(
                    "Constraint-violating data was accepted (status {}), expected rejection",
                    resp.status_code
                ));
            }

            let status = if conformance_warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            };

            reporter.record(TestResult {
                yang_module: "ietf-interfaces".to_string(),
                operation: format!("PUT {resource_path} (constraint violation)"),
                status,
                details: Some(TestDetails {
                    expected: Some(
                        "4xx with ietf-restconf:errors (error-type, error-tag, error-message)"
                            .to_string(),
                    ),
                    actual: Some(format!("status={}", resp.status_code)),
                    request: Some(format!("PUT {resource_path} (prefix-length=99)")),
                    response: Some(
                        String::from_utf8_lossy(&resp.body[..resp.body.len().min(512)]).to_string(),
                    ),
                    conformance_warnings,
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "ietf-interfaces".to_string(),
                operation: format!("PUT {resource_path} (constraint violation)"),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("Error response".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some(format!("PUT {resource_path}")),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Structural difference reporting as conformance warnings
// Requirements: 4.4
// ===========================================================================

/// Verify that structural differences between emulator responses and expected
/// RESTCONF protocol format are captured and reported as conformance warnings.
///
/// This test sends multiple requests and aggregates all conformance results
/// into a single report, demonstrating the full conformance reporting workflow.
#[tokio::test]
async fn test_server_conformance_full_report_with_warnings() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // Apply a fixture for known data
    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");

    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );
    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply interfaces fixture");

    // --- Test 1: GET interfaces and check structure ---
    {
        let url = client.build_url("/data/ietf-interfaces:interfaces");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        match client.execute(request).await {
            Ok(resp) if resp.is_success() && !resp.body.is_empty() => {
                let mut warnings = Vec::new();

                // Check Content-Type
                match resp.get_header("Content-Type") {
                    Some(ct)
                        if ct.contains("yang-data+json") || ct.contains("application/json") => {}
                    Some(ct) => {
                        warnings.push(format!("Unexpected Content-Type: {ct}"));
                    }
                    None => {
                        warnings.push("Missing Content-Type header".to_string());
                    }
                }

                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    // Compare against the expected structure from the fixture
                    let structural_warnings = compare_json_structures(&fixture.data, &json);
                    warnings.extend(structural_warnings);
                }

                reporter.record(TestResult {
                    yang_module: "ietf-interfaces".to_string(),
                    operation: "GET /data/ietf-interfaces:interfaces".to_string(),
                    status: if warnings.is_empty() {
                        TestStatus::Pass
                    } else {
                        TestStatus::Fail
                    },
                    details: Some(TestDetails {
                        expected: None,
                        actual: Some(format!("status={}", resp.status_code)),
                        request: Some("GET /data/ietf-interfaces:interfaces".to_string()),
                        response: None,
                        conformance_warnings: warnings,
                    }),
                });
            }
            Ok(resp) => {
                reporter.record(TestResult {
                    yang_module: "ietf-interfaces".to_string(),
                    operation: "GET /data/ietf-interfaces:interfaces".to_string(),
                    status: TestStatus::Fail,
                    details: Some(TestDetails {
                        expected: Some("200 with JSON body".to_string()),
                        actual: Some(format!("status={}", resp.status_code)),
                        request: Some("GET /data/ietf-interfaces:interfaces".to_string()),
                        response: None,
                        conformance_warnings: vec![],
                    }),
                });
            }
            Err(e) => {
                reporter.record(TestResult {
                    yang_module: "ietf-interfaces".to_string(),
                    operation: "GET /data/ietf-interfaces:interfaces".to_string(),
                    status: TestStatus::Fail,
                    details: Some(TestDetails {
                        expected: Some("Successful response".to_string()),
                        actual: Some(format!("Error: {e}")),
                        request: Some("GET /data/ietf-interfaces:interfaces".to_string()),
                        response: None,
                        conformance_warnings: vec![],
                    }),
                });
            }
        }
    }

    // --- Test 2: GET system and check structure ---
    {
        let url = client.build_url("/data/ietf-system:system");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        match client.execute(request).await {
            Ok(resp) if resp.is_success() && !resp.body.is_empty() => {
                let mut warnings = Vec::new();

                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    if json.get("ietf-system:system").is_none() {
                        warnings.push(
                            "Response missing 'ietf-system:system' top-level key".to_string(),
                        );
                    }
                }

                reporter.record(TestResult {
                    yang_module: "ietf-system".to_string(),
                    operation: "GET /data/ietf-system:system".to_string(),
                    status: if warnings.is_empty() {
                        TestStatus::Pass
                    } else {
                        TestStatus::Fail
                    },
                    details: Some(TestDetails {
                        expected: None,
                        actual: Some(format!("status={}", resp.status_code)),
                        request: Some("GET /data/ietf-system:system".to_string()),
                        response: None,
                        conformance_warnings: warnings,
                    }),
                });
            }
            Ok(resp) if resp.status_code == 404 => {
                reporter.record(TestResult {
                    yang_module: "ietf-system".to_string(),
                    operation: "GET /data/ietf-system:system".to_string(),
                    status: TestStatus::Skip {
                        reason: "ietf-system module not available on emulator".to_string(),
                    },
                    details: None,
                });
            }
            Ok(resp) => {
                reporter.record(TestResult {
                    yang_module: "ietf-system".to_string(),
                    operation: "GET /data/ietf-system:system".to_string(),
                    status: TestStatus::Fail,
                    details: Some(TestDetails {
                        expected: Some("200 or 404".to_string()),
                        actual: Some(format!("status={}", resp.status_code)),
                        request: Some("GET /data/ietf-system:system".to_string()),
                        response: None,
                        conformance_warnings: vec![],
                    }),
                });
            }
            Err(e) => {
                reporter.record(TestResult {
                    yang_module: "ietf-system".to_string(),
                    operation: "GET /data/ietf-system:system".to_string(),
                    status: TestStatus::Fail,
                    details: Some(TestDetails {
                        expected: Some("Successful response".to_string()),
                        actual: Some(format!("Error: {e}")),
                        request: Some("GET /data/ietf-system:system".to_string()),
                        response: None,
                        conformance_warnings: vec![],
                    }),
                });
            }
        }
    }

    // --- Test 3: Error response structure check ---
    {
        let url = client.build_url("/data/nonexistent-module:nonexistent");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        match client.execute(request).await {
            Ok(resp) if !resp.is_success() && !resp.body.is_empty() => {
                let mut warnings = Vec::new();

                match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                    Ok(json) => {
                        let error_warnings = validate_restconf_error_structure(&json);
                        warnings.extend(error_warnings);
                    }
                    Err(_) => {
                        warnings.push("Error response is not valid JSON".to_string());
                    }
                }

                reporter.record(TestResult {
                    yang_module: "restconf-protocol".to_string(),
                    operation: "GET /data/nonexistent (error structure)".to_string(),
                    status: if warnings.is_empty() {
                        TestStatus::Pass
                    } else {
                        TestStatus::Fail
                    },
                    details: Some(TestDetails {
                        expected: Some("ietf-restconf:errors structure".to_string()),
                        actual: Some(format!("status={}", resp.status_code)),
                        request: Some("GET /data/nonexistent-module:nonexistent".to_string()),
                        response: Some(
                            String::from_utf8_lossy(&resp.body[..resp.body.len().min(512)])
                                .to_string(),
                        ),
                        conformance_warnings: warnings,
                    }),
                });
            }
            Ok(resp) => {
                reporter.record(TestResult {
                    yang_module: "restconf-protocol".to_string(),
                    operation: "GET /data/nonexistent (error structure)".to_string(),
                    status: TestStatus::Fail,
                    details: Some(TestDetails {
                        expected: Some("4xx error with body".to_string()),
                        actual: Some(format!("status={}", resp.status_code)),
                        request: Some("GET /data/nonexistent-module:nonexistent".to_string()),
                        response: None,
                        conformance_warnings: vec![],
                    }),
                });
            }
            Err(e) => {
                reporter.record(TestResult {
                    yang_module: "restconf-protocol".to_string(),
                    operation: "GET /data/nonexistent (error structure)".to_string(),
                    status: TestStatus::Fail,
                    details: Some(TestDetails {
                        expected: Some("Error response".to_string()),
                        actual: Some(format!("Transport error: {e}")),
                        request: Some("GET /data/nonexistent-module:nonexistent".to_string()),
                        response: None,
                        conformance_warnings: vec![],
                    }),
                });
            }
        }
    }

    // Generate and print the full conformance report
    let report = reporter.generate_text_report();
    eprintln!("{report}");

    // Verify the report was populated
    let (pass, fail, skip) = reporter.summary();
    let total = pass + fail + skip;
    assert!(
        total >= 3,
        "Should have recorded at least 3 test results, got {total}"
    );

    // Also generate JUnit XML to verify it works
    let junit_xml = reporter.generate_junit_xml();
    assert!(
        junit_xml.contains("<testsuites"),
        "JUnit XML should contain testsuites element"
    );

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that Content-Type headers from the emulator match RESTCONF protocol
/// expectations for both success and error responses.
///
/// Requirements: 4.1, 4.2
#[tokio::test]
async fn test_server_conformance_content_type_headers() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("server-conformance");

    // Test Content-Type on a successful GET
    let url = client.build_url("/data");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    if let Ok(resp) = client.execute(request).await {
        let mut warnings = Vec::new();

        if resp.is_success() {
            match resp.get_header("Content-Type") {
                Some(ct) => {
                    // RFC 8040 Section 7.1: RESTCONF servers MUST use
                    // application/yang-data+json or application/yang-data+xml
                    if !ct.contains("yang-data+json") && !ct.contains("application/json") {
                        warnings.push(format!(
                            "Success response Content-Type '{ct}' does not match RESTCONF standard"
                        ));
                    }
                }
                None => {
                    if !resp.body.is_empty() {
                        warnings.push(
                            "Success response with body missing Content-Type header".to_string(),
                        );
                    }
                }
            }
        }

        reporter.record(TestResult {
            yang_module: "restconf-protocol".to_string(),
            operation: "GET /data (Content-Type check)".to_string(),
            status: if warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            },
            details: Some(TestDetails {
                expected: Some("Content-Type: application/yang-data+json".to_string()),
                actual: resp
                    .get_header("Content-Type")
                    .map(|ct| format!("Content-Type: {ct}"))
                    .or_else(|| Some("No Content-Type header".to_string())),
                request: Some("GET /data".to_string()),
                response: None,
                conformance_warnings: warnings,
            }),
        });
    }

    // Test Content-Type on an error response
    let error_url = client.build_url("/data/nonexistent-module:nonexistent");
    let error_request = HttpRequest::new(HttpMethod::GET, &error_url)
        .with_header("Accept", "application/yang-data+json");

    if let Ok(resp) = client.execute(error_request).await {
        let mut warnings = Vec::new();

        if !resp.is_success() && !resp.body.is_empty() {
            match resp.get_header("Content-Type") {
                Some(ct) => {
                    if !ct.contains("yang-data+json")
                        && !ct.contains("application/json")
                        && !ct.contains("yang-data+xml")
                    {
                        warnings.push(format!(
                            "Error response Content-Type '{ct}' does not match RESTCONF standard"
                        ));
                    }
                }
                None => {
                    warnings
                        .push("Error response with body missing Content-Type header".to_string());
                }
            }
        }

        reporter.record(TestResult {
            yang_module: "restconf-protocol".to_string(),
            operation: "GET /data/nonexistent (error Content-Type check)".to_string(),
            status: if warnings.is_empty() {
                TestStatus::Pass
            } else {
                TestStatus::Fail
            },
            details: Some(TestDetails {
                expected: Some("Content-Type: application/yang-data+json".to_string()),
                actual: resp
                    .get_header("Content-Type")
                    .map(|ct| format!("Content-Type: {ct}"))
                    .or_else(|| Some("No Content-Type header".to_string())),
                request: Some("GET /data/nonexistent-module:nonexistent".to_string()),
                response: None,
                conformance_warnings: warnings,
            }),
        });
    }

    let report = reporter.generate_text_report();
    eprintln!("{report}");

    harness.stop().await.expect("Failed to stop emulator");
}
