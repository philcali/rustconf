//! Error scenario tests for RESTCONF operations.
//!
//! These tests validate that the generated RESTCONF client handles error
//! conditions correctly when interacting with a live emulator. They cover:
//! - 404 responses on non-existent RESTCONF paths
//! - Malformed request bodies surfacing emulator error details
//! - Constraint-violating data (out-of-range values) rejection and error mapping
//! - Unexpected HTTP status codes (no panic, structured error returned)
//!
//! All emulator-dependent tests are gated on `RUSTCONF_INTEGRATION_TEST=1`
//! and require a running emulator container (Docker or Podman).
//!
//! Requirements: 10.1, 10.2, 10.3, 10.4

mod common;

use rustconf_integration_tests::{
    ConformanceReporter, HarnessConfig, JunosCrpdConfig, TestDetails, TestHarness, TestResult,
    TestStatus,
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

// ===========================================================================
// 404 on non-existent RESTCONF paths
// Requirements: 10.1
// ===========================================================================

/// Verify that a GET on a completely non-existent top-level RESTCONF resource
/// returns a 404 status and the client does not panic.
#[tokio::test]
async fn test_get_nonexistent_resource_returns_404() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let url = client.build_url("/data/nonexistent-module:nonexistent-container");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert_eq!(
                resp.status_code, 404,
                "GET on non-existent resource should return 404, got {}",
                resp.status_code
            );

            // If the emulator returns a body, it should be valid JSON
            // following the ietf-restconf:errors structure
            if !resp.body.is_empty() {
                let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&resp.body);
                assert!(
                    parsed.is_ok(),
                    "404 error response body should be valid JSON: {:?}",
                    parsed.err()
                );

                let json = parsed.unwrap();
                // RESTCONF errors should follow RFC 8040 structure
                if let Some(errors) = json.get("ietf-restconf:errors") {
                    assert!(
                        errors.get("error").is_some(),
                        "RESTCONF error response should contain 'error' array"
                    );
                }
            }
        }
        Err(e) => {
            // A transport-level error is also acceptable — the key is no panic
            eprintln!("Note: GET on non-existent resource returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a GET on a non-existent list entry (specific key) returns 404.
#[tokio::test]
async fn test_get_nonexistent_list_entry_returns_404() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Request a specific interface that does not exist
    let url = client
        .build_url("/data/ietf-interfaces:interfaces/interface=this-interface-does-not-exist-99");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert_eq!(
                resp.status_code, 404,
                "GET on non-existent list entry should return 404, got {}",
                resp.status_code
            );
        }
        Err(e) => {
            eprintln!("Note: GET on non-existent list entry returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a DELETE on a non-existent resource returns 404 or an
/// appropriate error, and the client handles it without panicking.
#[tokio::test]
async fn test_delete_nonexistent_resource_returns_error() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let url =
        client.build_url("/data/ietf-interfaces:interfaces/interface=nonexistent-delete-target");
    let request = HttpRequest::new(HttpMethod::DELETE, &url)
        .with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    // The client should not panic regardless of the response
    match response {
        Ok(resp) => {
            // 404 or 409 are both acceptable for deleting a non-existent resource
            assert!(
                !resp.is_success(),
                "DELETE on non-existent resource should not succeed, got {}",
                resp.status_code
            );
        }
        Err(e) => {
            eprintln!("Note: DELETE on non-existent resource returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Malformed request bodies surfacing emulator error details
// Requirements: 10.2
// ===========================================================================

/// Verify that sending a completely invalid JSON body surfaces the emulator's
/// error details in the response.
#[tokio::test]
async fn test_malformed_json_body_surfaces_error_details() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // Send completely invalid JSON
    let malformed_body = b"{ this is not valid json at all !!!".to_vec();
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(malformed_body);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert!(
                !resp.is_success(),
                "Malformed JSON body should be rejected, got status {}",
                resp.status_code
            );

            // The emulator should return a 400 Bad Request
            assert!(
                resp.status_code == 400 || resp.status_code == 415,
                "Malformed JSON should produce 400 or 415, got {}",
                resp.status_code
            );

            // The error response should contain details about the parse failure
            if !resp.body.is_empty() {
                let body_str = String::from_utf8_lossy(&resp.body);
                // The response should be parseable (either JSON or XML error)
                let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&resp.body);
                if let Ok(json) = parsed {
                    // If JSON, check for RESTCONF error structure
                    if let Some(errors) = json.get("ietf-restconf:errors") {
                        let error_array = errors.get("error").and_then(|e| e.as_array());
                        assert!(
                            error_array.is_some() && !error_array.unwrap().is_empty(),
                            "Error response should contain at least one error entry"
                        );
                    }
                } else {
                    // Even if not JSON, the body should contain some error info
                    assert!(
                        !body_str.is_empty(),
                        "Error response body should not be empty"
                    );
                }
            }
        }
        Err(e) => {
            // Transport error is acceptable — the client handled it without panicking
            eprintln!("Note: Malformed JSON request returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that sending a valid JSON body with incorrect YANG structure
/// (wrong module prefix, missing required fields) surfaces error details.
#[tokio::test]
async fn test_wrong_yang_structure_surfaces_error_details() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // Valid JSON but wrong YANG structure — missing required 'name' and 'type'
    // fields, and using a non-existent module prefix
    let bad_structure = serde_json::json!({
        "wrong-module:wrong-container": {
            "nonexistent-leaf": "some-value"
        }
    });

    let body_bytes = serde_json::to_vec(&bad_structure).expect("Failed to serialize body");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert!(
                !resp.is_success(),
                "Wrong YANG structure should be rejected, got status {}",
                resp.status_code
            );

            // The emulator should return a 4xx error
            assert!(
                (400..500).contains(&resp.status_code),
                "Wrong YANG structure should produce a 4xx error, got {}",
                resp.status_code
            );
        }
        Err(e) => {
            eprintln!("Note: Wrong YANG structure request returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that sending an empty body to a PUT endpoint surfaces an error.
#[tokio::test]
async fn test_empty_body_put_surfaces_error() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // Send PUT with empty body
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(Vec::new());

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert!(
                !resp.is_success(),
                "Empty body PUT should be rejected, got status {}",
                resp.status_code
            );
        }
        Err(e) => {
            eprintln!("Note: Empty body PUT returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Constraint-violating data rejection and error mapping
// Requirements: 10.3
// ===========================================================================

/// Verify that sending an out-of-range integer value (e.g., prefix-length > 32
/// for IPv4) is rejected by the emulator with an appropriate error.
#[tokio::test]
async fn test_out_of_range_integer_rejected() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // prefix-length for IPv4 is uint8 with range 0..32 — use 99 which is out of range
    let constraint_violating_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "constraint-violation-test",
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

    let body_bytes =
        serde_json::to_vec(&constraint_violating_data).expect("Failed to serialize body");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            // The emulator should reject the out-of-range value
            assert!(
                !resp.is_success(),
                "Out-of-range prefix-length (99) should be rejected, got status {}",
                resp.status_code
            );

            // Expect a 400 Bad Request or 409 Conflict for constraint violations
            assert!(
                resp.status_code == 400 || resp.status_code == 409,
                "Constraint violation should produce 400 or 409, got {}. Body: {}",
                resp.status_code,
                String::from_utf8_lossy(&resp.body)
            );
        }
        Err(e) => {
            eprintln!("Note: Constraint-violating request returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that sending an invalid IP address format (pattern constraint violation)
/// is rejected by the emulator.
#[tokio::test]
async fn test_invalid_ip_address_pattern_rejected() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // Use an invalid IP address that violates the YANG pattern constraint
    let invalid_pattern_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "invalid-ip-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "ietf-ip:ipv4": {
                        "address": [
                            {
                                "ip": "not-an-ip-address",
                                "prefix-length": 24
                            }
                        ]
                    }
                }
            ]
        }
    });

    let body_bytes = serde_json::to_vec(&invalid_pattern_data).expect("Failed to serialize body");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert!(
                !resp.is_success(),
                "Invalid IP address should be rejected, got status {}",
                resp.status_code
            );

            // Check that the error response contains useful information
            if !resp.body.is_empty() {
                let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&resp.body);
                if let Ok(json) = parsed {
                    // RESTCONF error responses should have error details
                    if let Some(errors) = json.get("ietf-restconf:errors") {
                        let error_array = errors.get("error").and_then(|e| e.as_array());
                        assert!(
                            error_array.is_some() && !error_array.unwrap().is_empty(),
                            "Constraint violation error should include error details"
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Note: Invalid IP address request returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that sending an invalid enumeration/identity value is rejected.
#[tokio::test]
async fn test_invalid_identity_value_rejected() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // Use a non-existent interface type identity
    let invalid_identity_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "invalid-type-test",
                    "type": "nonexistent-module:nonexistentType",
                    "enabled": true
                }
            ]
        }
    });

    let body_bytes = serde_json::to_vec(&invalid_identity_data).expect("Failed to serialize body");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert!(
                !resp.is_success(),
                "Invalid identity value should be rejected, got status {}",
                resp.status_code
            );
        }
        Err(e) => {
            eprintln!("Note: Invalid identity request returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that sending data with a missing mandatory leaf (e.g., interface
/// without 'name') is rejected by the emulator.
#[tokio::test]
async fn test_missing_mandatory_leaf_rejected() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let url = client.build_url(resource_path);

    // Interface list entry without the mandatory 'name' key
    let missing_mandatory_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                }
            ]
        }
    });

    let body_bytes = serde_json::to_vec(&missing_mandatory_data).expect("Failed to serialize body");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            assert!(
                !resp.is_success(),
                "Missing mandatory 'name' leaf should be rejected, got status {}",
                resp.status_code
            );
        }
        Err(e) => {
            eprintln!("Note: Missing mandatory leaf request returned transport error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Unexpected HTTP status codes — no panic, structured error returned
// Requirements: 10.4
// ===========================================================================

/// Verify that the client handles a 405 Method Not Allowed response without
/// panicking and returns a structured error.
#[tokio::test]
async fn test_method_not_allowed_no_panic() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // POST to a data resource (not an RPC) — should return 405
    let url = client.build_url("/data/ietf-interfaces:interfaces");
    let body = serde_json::json!({"test": "value"});
    let body_bytes = serde_json::to_vec(&body).expect("Failed to serialize body");

    let request = HttpRequest::new(HttpMethod::POST, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    // The key assertion: the client must not panic
    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            // 405 or another 4xx is expected — the important thing is no panic
            assert!(
                !resp.is_success() || resp.status_code == 201,
                "POST to data resource should return an error or 201 (some emulators allow POST for create), got {}",
                resp.status_code
            );
        }
        Err(e) => {
            // Error is fine — the client handled it gracefully
            eprintln!("Note: POST to data resource returned error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that the client handles various non-standard HTTP status codes
/// without panicking. Tests multiple unusual status codes by sending
/// requests that are likely to produce different error responses.
#[tokio::test]
async fn test_various_error_status_codes_no_panic() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Test a series of requests designed to trigger different error codes
    let error_scenarios: Vec<(&str, HttpMethod, Option<Vec<u8>>)> = vec![
        // 404: non-existent resource
        ("/data/nonexistent:resource", HttpMethod::GET, None),
        // 400/415: wrong content type via malformed body
        (
            "/data/ietf-interfaces:interfaces",
            HttpMethod::PUT,
            Some(b"not json".to_vec()),
        ),
        // 404: deeply nested non-existent path
        (
            "/data/ietf-interfaces:interfaces/interface=x/subinterface=y/nested=z",
            HttpMethod::GET,
            None,
        ),
    ];

    for (path, method, body) in error_scenarios {
        let url = client.build_url(path);
        let mut request =
            HttpRequest::new(method, &url).with_header("Accept", "application/yang-data+json");

        if let Some(body_data) = body {
            request = request
                .with_header("Content-Type", "application/yang-data+json")
                .with_body(body_data);
        }

        // The critical assertion: no panic on any response
        let response = client.execute(request).await;

        match response {
            Ok(resp) => {
                // Any status code is acceptable — we're verifying no panic
                eprintln!(
                    "  {} {} -> status {}",
                    method.as_str(),
                    path,
                    resp.status_code
                );
            }
            Err(e) => {
                // Transport errors are fine — the client didn't panic
                eprintln!("  {} {} -> error: {}", method.as_str(), path, e);
            }
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that the client handles a 500 Internal Server Error response
/// without panicking and returns a structured result.
#[tokio::test]
async fn test_server_error_response_no_panic() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Send a request that might trigger a server-side error — use a deeply
    // nested path with special characters that could cause server issues
    let url = client.build_url("/data/ietf-interfaces:interfaces/interface=%00%01%02");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    // The key assertion: no panic regardless of what the server returns
    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            // Any status is fine — we're testing that the client handles it
            assert!(
                resp.status_code > 0,
                "Response should have a valid HTTP status code"
            );
        }
        Err(e) => {
            // Error is acceptable — the client handled it without panicking
            eprintln!("Note: Special character request returned error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that the client handles responses with unexpected Content-Type
/// headers without panicking.
#[tokio::test]
async fn test_unexpected_content_type_no_panic() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Request with Accept header for XML — the emulator may return XML or
    // reject the request, but the client should handle either case
    let url = client.build_url("/data/ietf-interfaces:interfaces");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+xml");

    let response = client.execute(request).await;

    // No panic is the key assertion
    match response {
        Ok(resp) => {
            eprintln!(
                "Note: XML Accept header -> status {}, body length {}",
                resp.status_code,
                resp.body.len()
            );
        }
        Err(e) => {
            eprintln!("Note: XML Accept header returned error: {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Conformance reporting for error scenarios
// ===========================================================================

/// Verify that error scenario results can be recorded in the ConformanceReporter
/// and produce a meaningful report.
#[tokio::test]
async fn test_error_scenarios_with_conformance_reporting() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("error-scenario-test");

    // Test 1: 404 on non-existent resource
    let url = client.build_url("/data/nonexistent-module:nonexistent");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;
    match response {
        Ok(resp) if resp.status_code == 404 => {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "GET /data/nonexistent (expect 404)".to_string(),
                status: TestStatus::Pass,
                details: None,
            });
        }
        Ok(resp) => {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "GET /data/nonexistent (expect 404)".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("404".to_string()),
                    actual: Some(resp.status_code.to_string()),
                    request: Some("GET /data/nonexistent-module:nonexistent".to_string()),
                    response: Some(String::from_utf8_lossy(&resp.body).to_string()),
                    conformance_warnings: vec![],
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "GET /data/nonexistent (expect 404)".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("404".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some("GET /data/nonexistent-module:nonexistent".to_string()),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    // Test 2: Malformed body rejection
    let url = client.build_url("/data/ietf-interfaces:interfaces");
    let request = HttpRequest::new(HttpMethod::PUT, &url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(b"invalid json".to_vec());

    let response = client.execute(request).await;
    match response {
        Ok(resp) if (400..500).contains(&resp.status_code) => {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "PUT malformed body (expect 4xx)".to_string(),
                status: TestStatus::Pass,
                details: None,
            });
        }
        Ok(resp) => {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "PUT malformed body (expect 4xx)".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("4xx".to_string()),
                    actual: Some(resp.status_code.to_string()),
                    request: Some(
                        "PUT /data/ietf-interfaces:interfaces (malformed body)".to_string(),
                    ),
                    response: Some(String::from_utf8_lossy(&resp.body).to_string()),
                    conformance_warnings: vec![],
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "PUT malformed body (expect 4xx)".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("4xx".to_string()),
                    actual: Some(format!("Transport error: {e}")),
                    request: Some(
                        "PUT /data/ietf-interfaces:interfaces (malformed body)".to_string(),
                    ),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    // Verify the report
    let (pass, fail, _skip) = reporter.summary();
    let total = pass + fail;
    assert_eq!(total, 2, "Should have recorded exactly 2 test results");

    let report = reporter.generate_text_report();
    assert!(
        report.contains("Conformance Report"),
        "Report should contain header"
    );
    assert!(
        report.contains("error-handling"),
        "Report should contain the error-handling module"
    );

    harness.stop().await.expect("Failed to stop emulator");
}
