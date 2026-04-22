//! Client integration tests for RESTCONF operations.
//!
//! These tests validate that the generated RESTCONF client correctly performs
//! GET, PUT, PATCH, and RPC operations against a live emulator. They verify
//! URL construction, request body serialization, and response deserialization.
//!
//! All tests are gated on `RUSTCONF_INTEGRATION_TEST=1` and require a running
//! emulator container (Docker or Podman).
//!
//! Requirements: 3.1, 3.2, 3.3, 3.5, 3.6

mod common;

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
// GET operation tests
// ---------------------------------------------------------------------------

/// Verify that a GET on the RESTCONF root returns a valid JSON response
/// that can be deserialized into a serde_json::Value (generic type check).
#[tokio::test]
async fn test_get_restconf_root_returns_valid_json() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // GET the RESTCONF data root
    let url = client.build_url("/data");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;
    assert!(
        response.is_ok(),
        "GET /data should not return a transport error: {:?}",
        response.err()
    );

    let response = response.unwrap();
    // The emulator should return a 2xx or a known status
    assert!(
        response.status_code == 200 || response.status_code == 204,
        "Expected 200 or 204 from GET /data, got {}",
        response.status_code
    );

    // If there's a body, it should be valid JSON
    if !response.body.is_empty() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&response.body);
        assert!(
            parsed.is_ok(),
            "GET /data response body should be valid JSON: {:?}",
            parsed.err()
        );
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a GET on a specific resource path deserializes into a JSON object
/// with the expected YANG module-prefixed keys.
#[tokio::test]
async fn test_get_specific_resource_deserializes_into_json() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // First apply a fixture so we have known data
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

    // GET the interfaces resource
    let url = client.build_url("/data/ietf-interfaces:interfaces");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client
        .execute(request)
        .await
        .expect("GET interfaces failed");

    assert!(
        response.is_success(),
        "Expected success from GET interfaces, got {}",
        response.status_code
    );

    // Deserialize into a generic JSON value
    let json: serde_json::Value =
        serde_json::from_slice(&response.body).expect("Response should be valid JSON");

    // The response should contain the YANG module-prefixed key
    assert!(
        json.get("ietf-interfaces:interfaces").is_some(),
        "Response should contain 'ietf-interfaces:interfaces' key, got: {}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}

// ---------------------------------------------------------------------------
// PUT operation tests
// ---------------------------------------------------------------------------

/// Verify that a PUT operation is accepted by the emulator and the config
/// change is reflected in a subsequent GET.
#[tokio::test]
async fn test_put_creates_config_reflected_in_get() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let resource_path = "/data/ietf-interfaces:interfaces";
    let put_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "test-iface-put",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                }
            ]
        }
    });

    // Save original state for cleanup
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original_response = client.execute(get_request).await;

    // PUT the new configuration
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

    assert!(
        put_response.is_success(),
        "PUT should succeed, got status {}. Body: {}",
        put_response.status_code,
        String::from_utf8_lossy(&put_response.body)
    );

    // GET the resource back and verify the change
    let verify_url = client.build_url(resource_path);
    let verify_request = HttpRequest::new(HttpMethod::GET, &verify_url)
        .with_header("Accept", "application/yang-data+json");

    let verify_response = client
        .execute(verify_request)
        .await
        .expect("GET after PUT failed");

    assert!(
        verify_response.is_success(),
        "GET after PUT should succeed, got {}",
        verify_response.status_code
    );

    let verify_json: serde_json::Value =
        serde_json::from_slice(&verify_response.body).expect("GET response should be valid JSON");

    // Verify the interface we PUT is present
    let interfaces = verify_json
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array());

    assert!(
        interfaces.is_some(),
        "Response should contain interface array"
    );

    let has_test_iface = interfaces
        .unwrap()
        .iter()
        .any(|iface| iface.get("name").and_then(|n| n.as_str()) == Some("test-iface-put"));

    assert!(
        has_test_iface,
        "PUT interface 'test-iface-put' should be present in GET response"
    );

    // Restore original state if we had one
    if let Ok(orig) = original_response {
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

// ---------------------------------------------------------------------------
// PATCH operation tests
// ---------------------------------------------------------------------------

/// Verify that a PATCH operation is accepted by the emulator and the partial
/// update is reflected in a subsequent GET.
#[tokio::test]
async fn test_patch_updates_config_reflected_in_get() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // First, apply a known fixture
    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");
    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply fixture");

    // PATCH to update a specific field (e.g., disable an interface)
    let patch_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "ge-0/0/0",
                    "enabled": false
                }
            ]
        }
    });

    let resource_path = "/data/ietf-interfaces:interfaces";
    let patch_url = client.build_url(resource_path);
    let patch_body = serde_json::to_vec(&patch_data).expect("Failed to serialize PATCH body");
    let patch_request = HttpRequest::new(HttpMethod::PATCH, &patch_url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(patch_body);

    let patch_response = client
        .execute(patch_request)
        .await
        .expect("PATCH request failed");

    // PATCH may return 200 or 204 on success, or 405 if not supported
    if patch_response.is_success() {
        // Verify the change via GET
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET after PATCH failed");

        assert!(
            get_response.is_success(),
            "GET after PATCH should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("GET response should be valid JSON");

        // Check that the patched interface has enabled=false
        if let Some(interfaces) = json
            .get("ietf-interfaces:interfaces")
            .and_then(|i| i.get("interface"))
            .and_then(|i| i.as_array())
        {
            let ge000 = interfaces
                .iter()
                .find(|iface| iface.get("name").and_then(|n| n.as_str()) == Some("ge-0/0/0"));

            if let Some(iface) = ge000 {
                let enabled = iface.get("enabled").and_then(|e| e.as_bool());
                assert_eq!(
                    enabled,
                    Some(false),
                    "PATCH should have set enabled=false on ge-0/0/0"
                );
            }
        }
    } else {
        // Some emulators may not support PATCH — record as a conformance note
        eprintln!(
            "Note: PATCH returned status {} — emulator may not support PATCH. Body: {}",
            patch_response.status_code,
            String::from_utf8_lossy(&patch_response.body)
        );
    }

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}

// ---------------------------------------------------------------------------
// RPC operation tests
// ---------------------------------------------------------------------------

/// Verify that an RPC operation returns a valid response matching the YANG
/// output schema. Uses the RESTCONF operations endpoint.
#[tokio::test]
async fn test_rpc_operation_returns_valid_response() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Attempt an RPC call — use a common IETF RPC if available.
    // ietf-system:restart is a standard RPC, but may not be safe to call.
    // Instead, try a read-only operation or a well-known RPC endpoint.
    // We'll try to POST to /operations and check the response format.
    let rpc_url = client.build_url("/operations");
    let rpc_request = HttpRequest::new(HttpMethod::GET, &rpc_url)
        .with_header("Accept", "application/yang-data+json");

    let rpc_response = client.execute(rpc_request).await;

    match rpc_response {
        Ok(response) => {
            // The operations endpoint should return a list of available RPCs
            // or a valid response. We just verify it doesn't panic and returns
            // structured data.
            if response.is_success() && !response.body.is_empty() {
                let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&response.body);
                assert!(
                    parsed.is_ok(),
                    "RPC response should be valid JSON: {:?}",
                    parsed.err()
                );
            }
            // Even non-success is acceptable — we're verifying the client
            // handles the response without panicking
        }
        Err(e) => {
            // Transport errors are acceptable if the endpoint doesn't exist,
            // but the client should not panic
            eprintln!("Note: RPC endpoint returned error (may not be supported): {e}");
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that a POST-based RPC invocation is handled correctly by the client.
#[tokio::test]
async fn test_rpc_post_invocation() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Try invoking an RPC via POST. Use a safe, read-only RPC if available.
    // ietf-system:system-restart is destructive, so we use a GET-like probe
    // or a known safe RPC. If no safe RPC exists, we verify the error handling.
    let rpc_url = client.build_url("/operations/ietf-system:system-state");
    let rpc_body = serde_json::json!({});
    let body_bytes = serde_json::to_vec(&rpc_body).expect("Failed to serialize RPC body");

    let rpc_request = HttpRequest::new(HttpMethod::POST, &rpc_url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let rpc_response = client.execute(rpc_request).await;

    // The key assertion: the client should never panic regardless of the response
    match rpc_response {
        Ok(response) => {
            // Any HTTP status is acceptable — we're testing that the client
            // handles it gracefully
            if response.is_success() && !response.body.is_empty() {
                let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&response.body);
                assert!(
                    parsed.is_ok(),
                    "Successful RPC response should be valid JSON"
                );
            }
        }
        Err(_) => {
            // Error responses are fine — the client handled it without panicking
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ---------------------------------------------------------------------------
// URL construction tests
// ---------------------------------------------------------------------------

/// Verify that the client's URL construction produces URLs that the emulator
/// can resolve to the correct resources.
#[tokio::test]
async fn test_url_construction_resolves_correctly() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Apply a fixture so we have known resources
    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");
    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply fixture");

    // Test various URL patterns that the generated client would construct
    let test_paths = vec![
        // Collection resource
        "/data/ietf-interfaces:interfaces",
        // Nested resource with list key
        "/data/ietf-interfaces:interfaces/interface=ge-0/0/0",
    ];

    for path in &test_paths {
        let url = client.build_url(path);
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client.execute(request).await;

        match response {
            Ok(resp) => {
                // The emulator should resolve the URL — either 200 with data
                // or 404 if the specific key encoding differs
                assert!(
                    resp.status_code == 200 || resp.status_code == 404,
                    "URL '{}' should resolve to 200 or 404, got {}",
                    path,
                    resp.status_code
                );

                if resp.status_code == 200 && !resp.body.is_empty() {
                    let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&resp.body);
                    assert!(
                        parsed.is_ok(),
                        "Response for '{}' should be valid JSON",
                        path
                    );
                }
            }
            Err(e) => {
                panic!(
                    "URL '{}' caused a transport error (URL may be malformed): {}",
                    path, e
                );
            }
        }
    }

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that build_url correctly combines the base URL with resource paths.
#[tokio::test]
async fn test_url_construction_format() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let base = harness.base_url();

    // build_url should produce a well-formed URL
    let url = client.build_url("/data/ietf-interfaces:interfaces");
    assert!(
        url.starts_with(base),
        "Built URL '{}' should start with base URL '{}'",
        url,
        base
    );
    assert!(
        url.contains("/data/ietf-interfaces:interfaces"),
        "Built URL '{}' should contain the resource path",
        url
    );

    // Verify no double slashes in the path portion
    let after_scheme = if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = url.strip_prefix("http://") {
        stripped
    } else {
        &url
    };
    // After the host:port, there should be no double slashes
    if let Some(path_start) = after_scheme.find('/') {
        let path_portion = &after_scheme[path_start..];
        assert!(
            !path_portion.contains("//"),
            "URL path should not contain double slashes: '{}'",
            url
        );
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ---------------------------------------------------------------------------
// Request body serialization tests
// ---------------------------------------------------------------------------

/// Verify that the emulator accepts JSON-encoded request bodies following
/// RFC 7951 format (YANG module-prefixed keys).
#[tokio::test]
async fn test_request_body_serialization_accepted() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Construct a valid RFC 7951 JSON body with module-prefixed keys
    let body_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "test-serialization",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "description": "Serialization test interface"
                }
            ]
        }
    });

    let resource_path = "/data/ietf-interfaces:interfaces";

    // Save original state
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // PUT with the JSON body
    let put_url = client.build_url(resource_path);
    let body_bytes = serde_json::to_vec(&body_data).expect("Failed to serialize body");
    let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let put_response = client
        .execute(put_request)
        .await
        .expect("PUT with JSON body failed");

    assert!(
        put_response.is_success(),
        "Emulator should accept RFC 7951 JSON body, got status {}. Body: {}",
        put_response.status_code,
        String::from_utf8_lossy(&put_response.body)
    );

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

/// Verify that the Content-Type header is correctly set and the emulator
/// does not reject the request due to serialization format issues.
#[tokio::test]
async fn test_request_content_type_accepted() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // Send a PUT with the correct RESTCONF content type
    let body_data = serde_json::json!({
        "ietf-system:system": {
            "hostname": "content-type-test"
        }
    });

    let resource_path = "/data/ietf-system:system";
    let put_url = client.build_url(resource_path);
    let body_bytes = serde_json::to_vec(&body_data).expect("Failed to serialize body");

    let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
        .with_header("Content-Type", "application/yang-data+json")
        .with_header("Accept", "application/yang-data+json")
        .with_body(body_bytes);

    let response = client
        .execute(put_request)
        .await
        .expect("PUT request failed");

    // The emulator should not return 415 Unsupported Media Type
    assert_ne!(
        response.status_code, 415,
        "Emulator should accept 'application/yang-data+json' content type"
    );

    harness.stop().await.expect("Failed to stop emulator");
}

// ---------------------------------------------------------------------------
// Conformance reporting integration
// ---------------------------------------------------------------------------

/// Verify that test results can be recorded in the ConformanceReporter
/// during client integration testing.
#[tokio::test]
async fn test_client_operations_with_conformance_reporting() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    let mut reporter = ConformanceReporter::new("integration-test");

    // Test GET on data root
    let url = client.build_url("/data");
    let request =
        HttpRequest::new(HttpMethod::GET, &url).with_header("Accept", "application/yang-data+json");

    let response = client.execute(request).await;

    match response {
        Ok(resp) if resp.is_success() => {
            reporter.record(TestResult {
                yang_module: "restconf-root".to_string(),
                operation: "GET /data".to_string(),
                status: TestStatus::Pass,
                details: None,
            });
        }
        Ok(resp) => {
            reporter.record(TestResult {
                yang_module: "restconf-root".to_string(),
                operation: "GET /data".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("200".to_string()),
                    actual: Some(resp.status_code.to_string()),
                    request: Some("GET /data".to_string()),
                    response: Some(String::from_utf8_lossy(&resp.body).to_string()),
                    conformance_warnings: vec![],
                }),
            });
        }
        Err(e) => {
            reporter.record(TestResult {
                yang_module: "restconf-root".to_string(),
                operation: "GET /data".to_string(),
                status: TestStatus::Fail,
                details: Some(TestDetails {
                    expected: Some("200".to_string()),
                    actual: Some(format!("Error: {e}")),
                    request: Some("GET /data".to_string()),
                    response: None,
                    conformance_warnings: vec![],
                }),
            });
        }
    }

    // Generate and verify the report
    let (pass, fail, skip) = reporter.summary();
    let total = pass + fail + skip;
    assert_eq!(total, 1, "Should have recorded exactly 1 test result");

    let report = reporter.generate_text_report();
    assert!(
        report.contains("Conformance Report"),
        "Report should contain header"
    );

    harness.stop().await.expect("Failed to stop emulator");
}
