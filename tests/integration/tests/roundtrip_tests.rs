//! Serialization round-trip tests for RESTCONF operations.
//!
//! These tests validate that data written to the emulator via the generated
//! RESTCONF client can be read back and compared for equivalence. They also
//! verify that YANG types with constraints (ranges, patterns, enumerations)
//! deserialize correctly into the expected Rust types.
//!
//! All tests are gated on `RUSTCONF_INTEGRATION_TEST=1` and require a running
//! emulator container (Docker or Podman).
//!
//! Requirements: 7.1, 7.2, 7.4

mod common;

use std::path::Path;

use rustconf_integration_tests::{FixtureManager, HarnessConfig, JunosCrpdConfig, TestHarness};
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

/// Helper: PUT a JSON value at a resource path and GET it back, returning the
/// read-back JSON value. Panics on transport or HTTP errors.
async fn put_then_get(
    harness: &TestHarness,
    resource_path: &str,
    data: &serde_json::Value,
) -> serde_json::Value {
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // PUT the data
    let put_url = client.build_url(resource_path);
    let put_body = serde_json::to_vec(data).expect("Failed to serialize PUT body");
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
        "PUT {} should succeed, got status {}. Body: {}",
        resource_path,
        put_response.status_code,
        String::from_utf8_lossy(&put_response.body)
    );

    // GET the data back
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let get_response = client
        .execute(get_request)
        .await
        .expect("GET request failed");

    assert!(
        get_response.is_success(),
        "GET {} should succeed after PUT, got status {}",
        resource_path,
        get_response.status_code
    );

    serde_json::from_slice(&get_response.body).expect("GET response should be valid JSON")
}

// ===========================================================================
// Write-read-compare round-trip tests for multiple YANG types
// Requirements: 7.1, 7.4
// ===========================================================================

/// Round-trip test for string-typed YANG leaves (e.g., hostname, description).
///
/// Writes a system configuration with string values, reads it back, and verifies
/// the strings are preserved exactly.
#[tokio::test]
async fn test_roundtrip_string_type() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-system:system";
    let write_data = serde_json::json!({
        "ietf-system:system": {
            "hostname": "roundtrip-string-test",
            "contact": "Test Contact <test@example.com>",
            "location": "Lab Rack 42"
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    // Compare string fields
    let system = read_back
        .get("ietf-system:system")
        .expect("Response should contain 'ietf-system:system'");

    assert_eq!(
        system.get("hostname").and_then(|v| v.as_str()),
        Some("roundtrip-string-test"),
        "hostname string should round-trip exactly"
    );
    assert_eq!(
        system.get("contact").and_then(|v| v.as_str()),
        Some("Test Contact <test@example.com>"),
        "contact string should round-trip exactly"
    );
    assert_eq!(
        system.get("location").and_then(|v| v.as_str()),
        Some("Lab Rack 42"),
        "location string should round-trip exactly"
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

/// Round-trip test for boolean-typed YANG leaves (e.g., enabled).
///
/// Writes interface configurations with boolean values, reads them back,
/// and verifies the booleans are preserved.
#[tokio::test]
async fn test_roundtrip_boolean_type() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-interfaces:interfaces";
    let write_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "roundtrip-bool-true",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                },
                {
                    "name": "roundtrip-bool-false",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": false
                }
            ]
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    let interfaces = read_back
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Response should contain interface array");

    // Find and verify the boolean values
    let true_iface = interfaces
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("roundtrip-bool-true"));
    let false_iface = interfaces
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("roundtrip-bool-false"));

    assert!(
        true_iface.is_some(),
        "Interface 'roundtrip-bool-true' should be present"
    );
    assert!(
        false_iface.is_some(),
        "Interface 'roundtrip-bool-false' should be present"
    );

    assert_eq!(
        true_iface.unwrap().get("enabled").and_then(|v| v.as_bool()),
        Some(true),
        "enabled=true should round-trip correctly"
    );
    assert_eq!(
        false_iface
            .unwrap()
            .get("enabled")
            .and_then(|v| v.as_bool()),
        Some(false),
        "enabled=false should round-trip correctly"
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

/// Round-trip test for integer-typed YANG leaves (e.g., prefix-length, port).
///
/// Writes configuration with integer values, reads them back, and verifies
/// the integers are preserved.
#[tokio::test]
async fn test_roundtrip_integer_type() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-interfaces:interfaces";
    let write_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "roundtrip-int-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "ietf-ip:ipv4": {
                        "address": [
                            {
                                "ip": "10.0.0.1",
                                "prefix-length": 30
                            }
                        ]
                    }
                }
            ]
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    let interfaces = read_back
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Response should contain interface array");

    let iface = interfaces
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("roundtrip-int-test"))
        .expect("Interface 'roundtrip-int-test' should be present");

    // Verify the integer prefix-length round-trips
    let prefix_length = iface
        .get("ietf-ip:ipv4")
        .and_then(|v| v.get("address"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|addr| addr.get("prefix-length"))
        .and_then(|v| v.as_u64());

    assert_eq!(
        prefix_length,
        Some(30),
        "prefix-length integer should round-trip correctly"
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

/// Round-trip test for nested container and list YANG structures.
///
/// Writes a complex nested configuration (interfaces with IP addresses),
/// reads it back, and verifies the full structure is preserved.
#[tokio::test]
async fn test_roundtrip_nested_containers_and_lists() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-system:system";
    let write_data = serde_json::json!({
        "ietf-system:system": {
            "hostname": "roundtrip-nested-test",
            "ntp": {
                "enabled": true,
                "server": [
                    {
                        "name": "ntp1.example.com",
                        "udp": {
                            "address": "198.51.100.1",
                            "port": 123
                        },
                        "prefer": true
                    },
                    {
                        "name": "ntp2.example.com",
                        "udp": {
                            "address": "198.51.100.2",
                            "port": 123
                        }
                    }
                ]
            }
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    let system = read_back
        .get("ietf-system:system")
        .expect("Response should contain 'ietf-system:system'");

    // Verify nested NTP container
    let ntp = system.get("ntp").expect("system should contain 'ntp'");
    assert_eq!(
        ntp.get("enabled").and_then(|v| v.as_bool()),
        Some(true),
        "ntp.enabled should round-trip correctly"
    );

    // Verify NTP server list
    let servers = ntp
        .get("server")
        .and_then(|s| s.as_array())
        .expect("ntp should contain 'server' array");

    assert!(
        servers.len() >= 2,
        "Should have at least 2 NTP servers, got {}",
        servers.len()
    );

    let ntp1 = servers
        .iter()
        .find(|s| s.get("name").and_then(|n| n.as_str()) == Some("ntp1.example.com"));
    assert!(
        ntp1.is_some(),
        "NTP server 'ntp1.example.com' should be present"
    );

    // Verify nested UDP container within the NTP server
    let udp = ntp1
        .unwrap()
        .get("udp")
        .expect("NTP server should contain 'udp'");
    assert_eq!(
        udp.get("address").and_then(|v| v.as_str()),
        Some("198.51.100.1"),
        "NTP server UDP address should round-trip correctly"
    );
    assert_eq!(
        udp.get("port").and_then(|v| v.as_u64()),
        Some(123),
        "NTP server UDP port should round-trip correctly"
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

// ===========================================================================
// Fixture apply-teardown round-trip tests
// Requirements: 6.2
// ===========================================================================

/// Verify that applying a fixture and tearing it down restores the original
/// emulator state for the interfaces resource.
///
/// This test:
/// 1. Reads the original state at the fixture's resource path
/// 2. Applies the interfaces fixture (overwriting the resource)
/// 3. Verifies the fixture data is present
/// 4. Tears down the fixture
/// 5. Reads the state again and verifies it matches the original
#[tokio::test]
async fn test_fixture_apply_teardown_restores_interfaces_state() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");

    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // 1. Capture original state before applying the fixture
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let original_response = client
        .execute(get_request)
        .await
        .expect("GET original state failed");

    // The resource may or may not exist initially — both are valid baselines.
    let original_data: Option<serde_json::Value> =
        if original_response.is_success() && !original_response.body.is_empty() {
            Some(
                serde_json::from_slice(&original_response.body)
                    .expect("Original response should be valid JSON"),
            )
        } else {
            None
        };

    // 2. Apply the fixture
    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply interfaces fixture");

    // 3. Verify the fixture data is present
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let applied_response = client
        .execute(get_request)
        .await
        .expect("GET after fixture apply failed");

    assert!(
        applied_response.is_success(),
        "GET should succeed after fixture apply, got {}",
        applied_response.status_code
    );

    let applied_data: serde_json::Value = serde_json::from_slice(&applied_response.body)
        .expect("Applied response should be valid JSON");

    // The fixture data should contain the interfaces we wrote
    let applied_interfaces = applied_data
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Applied data should contain interface array");

    let has_ge000 = applied_interfaces
        .iter()
        .any(|i| i.get("name").and_then(|n| n.as_str()) == Some("ge-0/0/0"));
    assert!(
        has_ge000,
        "Fixture interface 'ge-0/0/0' should be present after apply"
    );

    // 4. Tear down the fixture
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");

    // 5. Read the state again and verify it matches the original
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let restored_response = client
        .execute(get_request)
        .await
        .expect("GET after teardown failed");

    match &original_data {
        Some(original) => {
            // Original existed — restored data should match
            assert!(
                restored_response.is_success(),
                "GET after teardown should succeed, got {}",
                restored_response.status_code
            );

            let restored: serde_json::Value = serde_json::from_slice(&restored_response.body)
                .expect("Restored response should be valid JSON");

            assert_eq!(
                original, &restored,
                "State after teardown should match original state"
            );
        }
        None => {
            // Original did not exist — resource should be gone (404) or empty
            assert!(
                restored_response.status_code == 404
                    || restored_response.body.is_empty()
                    || {
                        // Some emulators return an empty container instead of 404
                        let v: serde_json::Value =
                            serde_json::from_slice(&restored_response.body)
                                .unwrap_or(serde_json::Value::Null);
                        v.is_null()
                            || v.get("ietf-interfaces:interfaces")
                                .and_then(|i| i.get("interface"))
                                .and_then(|i| i.as_array())
                                .is_some_and(|a| a.is_empty())
                    },
                "Resource should be absent or empty after teardown when it did not exist originally, got status {}",
                restored_response.status_code
            );
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that applying a fixture and tearing it down restores the original
/// emulator state for the system resource.
///
/// Uses the system fixture (hostname, NTP, DNS) to exercise a different
/// resource path and data shape than the interfaces test above.
#[tokio::test]
async fn test_fixture_apply_teardown_restores_system_state() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let fixture = FixtureManager::load_fixture(Path::new("fixtures/system.json"))
        .expect("Failed to load system fixture");

    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // 1. Capture original state
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let original_response = client
        .execute(get_request)
        .await
        .expect("GET original system state failed");

    let original_data: Option<serde_json::Value> =
        if original_response.is_success() && !original_response.body.is_empty() {
            Some(
                serde_json::from_slice(&original_response.body)
                    .expect("Original system response should be valid JSON"),
            )
        } else {
            None
        };

    // 2. Apply the system fixture
    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply system fixture");

    // 3. Verify the fixture data is present
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let applied_response = client
        .execute(get_request)
        .await
        .expect("GET after system fixture apply failed");

    assert!(
        applied_response.is_success(),
        "GET should succeed after system fixture apply, got {}",
        applied_response.status_code
    );

    let applied_data: serde_json::Value = serde_json::from_slice(&applied_response.body)
        .expect("Applied system response should be valid JSON");

    let system = applied_data
        .get("ietf-system:system")
        .expect("Applied data should contain 'ietf-system:system'");

    assert_eq!(
        system.get("hostname").and_then(|v| v.as_str()),
        Some("rustconf-test-device"),
        "Fixture hostname should be present after apply"
    );

    // 4. Tear down the fixture
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown system fixture");

    // 5. Read the state again and verify it matches the original
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let restored_response = client
        .execute(get_request)
        .await
        .expect("GET after system teardown failed");

    match &original_data {
        Some(original) => {
            assert!(
                restored_response.is_success(),
                "GET after teardown should succeed, got {}",
                restored_response.status_code
            );

            let restored: serde_json::Value = serde_json::from_slice(&restored_response.body)
                .expect("Restored system response should be valid JSON");

            assert_eq!(
                original, &restored,
                "System state after teardown should match original state"
            );
        }
        None => {
            assert!(
                restored_response.status_code == 404 || restored_response.body.is_empty(),
                "System resource should be absent after teardown when it did not exist originally, got status {}",
                restored_response.status_code
            );
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that applying multiple fixtures sequentially and tearing them all
/// down restores the original state for every resource path.
///
/// This exercises the FixtureManager's LIFO teardown ordering — the most
/// recently applied fixture is restored first.
#[tokio::test]
async fn test_fixture_apply_teardown_multiple_fixtures() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let iface_fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");
    let system_fixture = FixtureManager::load_fixture(Path::new("fixtures/system.json"))
        .expect("Failed to load system fixture");

    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // 1. Capture original state for both resources
    let original_iface = {
        let url = client.build_url(&iface_fixture.resource_path);
        let req = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");
        let resp = client
            .execute(req)
            .await
            .expect("GET original interfaces failed");
        if resp.is_success() && !resp.body.is_empty() {
            Some(
                serde_json::from_slice::<serde_json::Value>(&resp.body)
                    .expect("Original interfaces should be valid JSON"),
            )
        } else {
            None
        }
    };

    let original_system = {
        let url = client.build_url(&system_fixture.resource_path);
        let req = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");
        let resp = client
            .execute(req)
            .await
            .expect("GET original system failed");
        if resp.is_success() && !resp.body.is_empty() {
            Some(
                serde_json::from_slice::<serde_json::Value>(&resp.body)
                    .expect("Original system should be valid JSON"),
            )
        } else {
            None
        }
    };

    // 2. Apply both fixtures
    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    fixture_mgr
        .apply(&iface_fixture)
        .await
        .expect("Failed to apply interfaces fixture");
    fixture_mgr
        .apply(&system_fixture)
        .await
        .expect("Failed to apply system fixture");

    assert_eq!(
        fixture_mgr.applied_count(),
        2,
        "Two fixtures should be tracked"
    );

    // 3. Tear down all fixtures at once
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");

    assert_eq!(
        fixture_mgr.applied_count(),
        0,
        "No fixtures should be tracked after teardown"
    );

    // 4. Verify interfaces resource is restored
    let restored_iface = {
        let url = client.build_url(&iface_fixture.resource_path);
        let req = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");
        client
            .execute(req)
            .await
            .expect("GET restored interfaces failed")
    };

    match &original_iface {
        Some(original) => {
            assert!(
                restored_iface.is_success(),
                "Interfaces GET should succeed after teardown"
            );
            let restored: serde_json::Value = serde_json::from_slice(&restored_iface.body)
                .expect("Restored interfaces should be valid JSON");
            assert_eq!(
                original, &restored,
                "Interfaces state should match original after multi-fixture teardown"
            );
        }
        None => {
            assert!(
                restored_iface.status_code == 404 || restored_iface.body.is_empty(),
                "Interfaces should be absent after teardown when originally absent"
            );
        }
    }

    // 5. Verify system resource is restored
    let restored_system = {
        let url = client.build_url(&system_fixture.resource_path);
        let req = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");
        client
            .execute(req)
            .await
            .expect("GET restored system failed")
    };

    match &original_system {
        Some(original) => {
            assert!(
                restored_system.is_success(),
                "System GET should succeed after teardown"
            );
            let restored: serde_json::Value = serde_json::from_slice(&restored_system.body)
                .expect("Restored system should be valid JSON");
            assert_eq!(
                original, &restored,
                "System state should match original after multi-fixture teardown"
            );
        }
        None => {
            assert!(
                restored_system.status_code == 404 || restored_system.body.is_empty(),
                "System should be absent after teardown when originally absent"
            );
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

/// Verify that applying the same fixture twice and tearing down restores
/// the state that existed before the first apply.
///
/// This catches bugs where re-applying a fixture overwrites the saved
/// original state with the fixture data itself.
#[tokio::test]
async fn test_fixture_double_apply_teardown_restores_original() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");

    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");

    // 1. Capture original state
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let original_response = client
        .execute(get_request)
        .await
        .expect("GET original state failed");

    let original_data: Option<serde_json::Value> =
        if original_response.is_success() && !original_response.body.is_empty() {
            Some(
                serde_json::from_slice(&original_response.body)
                    .expect("Original response should be valid JSON"),
            )
        } else {
            None
        };

    // 2. Apply the fixture twice
    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    fixture_mgr
        .apply(&fixture)
        .await
        .expect("First fixture apply failed");
    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Second fixture apply failed");

    assert_eq!(
        fixture_mgr.applied_count(),
        2,
        "Both applies should be tracked"
    );

    // 3. Tear down all
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");

    // 4. Verify original state is restored (not the fixture data)
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let restored_response = client
        .execute(get_request)
        .await
        .expect("GET after double-apply teardown failed");

    match &original_data {
        Some(original) => {
            assert!(
                restored_response.is_success(),
                "GET after teardown should succeed, got {}",
                restored_response.status_code
            );

            let restored: serde_json::Value = serde_json::from_slice(&restored_response.body)
                .expect("Restored response should be valid JSON");

            assert_eq!(
                original, &restored,
                "State after double-apply teardown should match the original state, not the fixture data"
            );
        }
        None => {
            assert!(
                restored_response.status_code == 404 || restored_response.body.is_empty(),
                "Resource should be absent after teardown when originally absent"
            );
        }
    }

    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// Write-read-compare round-trip tests using FixtureManager
// Requirements: 7.1, 7.4
// ===========================================================================

/// Round-trip test using the fixture manager for write-read-compare.
///
/// Loads a fixture from a JSON file, applies it via FixtureManager, reads
/// the data back, and compares key fields for equivalence.
#[tokio::test]
async fn test_roundtrip_via_fixture_manager() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let fixture = FixtureManager::load_fixture(Path::new("fixtures/interfaces.json"))
        .expect("Failed to load interfaces fixture");

    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    // Apply the fixture
    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply interfaces fixture");

    // Read back the data
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let get_response = client
        .execute(get_request)
        .await
        .expect("GET after fixture apply failed");

    assert!(
        get_response.is_success(),
        "GET should succeed after fixture apply, got {}",
        get_response.status_code
    );

    let read_back: serde_json::Value =
        serde_json::from_slice(&get_response.body).expect("GET response should be valid JSON");

    // Compare: the fixture data should be reflected in the read-back
    let written_interfaces = fixture
        .data
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Fixture should contain interface array");

    let read_interfaces = read_back
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Read-back should contain interface array");

    // Verify each written interface is present in the read-back
    for written_iface in written_interfaces {
        let name = written_iface
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Written interface should have a name");

        let read_iface = read_interfaces
            .iter()
            .find(|i| i.get("name").and_then(|n| n.as_str()) == Some(name));

        assert!(
            read_iface.is_some(),
            "Written interface '{}' should be present in read-back",
            name
        );

        // Compare key fields
        let read_iface = read_iface.unwrap();
        assert_eq!(
            written_iface.get("type").and_then(|v| v.as_str()),
            read_iface.get("type").and_then(|v| v.as_str()),
            "Interface '{}' type should match after round-trip",
            name
        );
        assert_eq!(
            written_iface.get("enabled").and_then(|v| v.as_bool()),
            read_iface.get("enabled").and_then(|v| v.as_bool()),
            "Interface '{}' enabled should match after round-trip",
            name
        );
    }

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}

// ===========================================================================
// YANG types with constraints: ranges, patterns, enumerations
// Requirements: 7.2
// ===========================================================================

/// Verify that YANG enumeration types (e.g., interface type identity) deserialize
/// correctly as strings matching the YANG-defined values.
#[tokio::test]
async fn test_yang_enumeration_type_deserializes_correctly() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-interfaces:interfaces";
    let write_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "enum-type-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true
                }
            ]
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    let interfaces = read_back
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Response should contain interface array");

    let iface = interfaces
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("enum-type-test"))
        .expect("Interface 'enum-type-test' should be present");

    // The 'type' field is a YANG identityref — it should deserialize as a
    // module-prefixed string matching the YANG-defined identity value.
    let iface_type = iface
        .get("type")
        .and_then(|v| v.as_str())
        .expect("Interface should have a 'type' field");

    assert!(
        iface_type.contains("ethernetCsmacd"),
        "Interface type should contain 'ethernetCsmacd', got: '{}'",
        iface_type
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

/// Verify that YANG range-constrained integer types (e.g., prefix-length 0..32)
/// deserialize correctly and the value falls within the expected range.
#[tokio::test]
async fn test_yang_range_constrained_integer_deserializes_correctly() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-interfaces:interfaces";

    // prefix-length is defined as uint8 with range 0..32 for IPv4
    let write_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "range-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "ietf-ip:ipv4": {
                        "address": [
                            {
                                "ip": "10.1.1.1",
                                "prefix-length": 24
                            }
                        ]
                    }
                }
            ]
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    let interfaces = read_back
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Response should contain interface array");

    let iface = interfaces
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("range-test"))
        .expect("Interface 'range-test' should be present");

    let prefix_length = iface
        .get("ietf-ip:ipv4")
        .and_then(|v| v.get("address"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|addr| addr.get("prefix-length"))
        .and_then(|v| v.as_u64())
        .expect("prefix-length should be present and numeric");

    // Verify the value is within the YANG-defined range for IPv4 prefix-length
    assert!(
        prefix_length <= 32,
        "IPv4 prefix-length should be in range 0..32, got {}",
        prefix_length
    );
    assert_eq!(prefix_length, 24, "prefix-length should round-trip as 24");

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

/// Verify that YANG pattern-constrained string types (e.g., IP addresses)
/// deserialize correctly and match the expected pattern format.
#[tokio::test]
async fn test_yang_pattern_constrained_string_deserializes_correctly() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    let resource_path = "/data/ietf-interfaces:interfaces";
    let write_data = serde_json::json!({
        "ietf-interfaces:interfaces": {
            "interface": [
                {
                    "name": "pattern-test",
                    "type": "iana-if-type:ethernetCsmacd",
                    "enabled": true,
                    "ietf-ip:ipv4": {
                        "address": [
                            {
                                "ip": "192.168.1.100",
                                "prefix-length": 24
                            }
                        ]
                    }
                }
            ]
        }
    });

    // Save original state for cleanup
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");
    let original = client.execute(get_request).await;

    // Write and read back
    let read_back = put_then_get(&harness, resource_path, &write_data).await;

    let interfaces = read_back
        .get("ietf-interfaces:interfaces")
        .and_then(|i| i.get("interface"))
        .and_then(|i| i.as_array())
        .expect("Response should contain interface array");

    let iface = interfaces
        .iter()
        .find(|i| i.get("name").and_then(|n| n.as_str()) == Some("pattern-test"))
        .expect("Interface 'pattern-test' should be present");

    let ip_address = iface
        .get("ietf-ip:ipv4")
        .and_then(|v| v.get("address"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|addr| addr.get("ip"))
        .and_then(|v| v.as_str())
        .expect("IP address should be present and a string");

    // Verify the IP address matches the IPv4 dotted-decimal pattern
    let octets: Vec<&str> = ip_address.split('.').collect();
    assert_eq!(
        octets.len(),
        4,
        "IPv4 address should have 4 octets, got: '{}'",
        ip_address
    );

    for octet in &octets {
        let val: u32 = octet
            .parse()
            .unwrap_or_else(|_| panic!("Each octet should be numeric, got: '{}'", octet));
        assert!(
            val <= 255,
            "Each octet should be 0..255, got {} in '{}'",
            val,
            ip_address
        );
    }

    assert_eq!(
        ip_address, "192.168.1.100",
        "IP address should round-trip exactly"
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

/// Verify that YANG union types and multiple YANG types in a single resource
/// all deserialize correctly in a combined round-trip.
///
/// This test writes a system fixture containing strings, booleans, integers,
/// and nested structures, then verifies all types round-trip correctly.
#[tokio::test]
async fn test_roundtrip_multiple_yang_types_combined() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let mut harness = setup_harness().await;

    // Use the system fixture which contains multiple YANG types
    let fixture = FixtureManager::load_fixture(Path::new("fixtures/system.json"))
        .expect("Failed to load system fixture");

    let mut fixture_mgr = FixtureManager::new(
        harness
            .restconf_client()
            .expect("Failed to create fixture client"),
    );

    fixture_mgr
        .apply(&fixture)
        .await
        .expect("Failed to apply system fixture");

    // Read back the data
    let client = harness
        .restconf_client()
        .expect("Failed to create RESTCONF client");
    let get_url = client.build_url(&fixture.resource_path);
    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
        .with_header("Accept", "application/yang-data+json");

    let get_response = client
        .execute(get_request)
        .await
        .expect("GET after fixture apply failed");

    assert!(
        get_response.is_success(),
        "GET should succeed, got {}",
        get_response.status_code
    );

    let read_back: serde_json::Value =
        serde_json::from_slice(&get_response.body).expect("GET response should be valid JSON");

    let system = read_back
        .get("ietf-system:system")
        .expect("Response should contain 'ietf-system:system'");

    // String type: hostname
    assert_eq!(
        system.get("hostname").and_then(|v| v.as_str()),
        Some("rustconf-test-device"),
        "hostname (string) should round-trip"
    );

    // String type: contact
    assert_eq!(
        system.get("contact").and_then(|v| v.as_str()),
        Some("Integration Test Harness"),
        "contact (string) should round-trip"
    );

    // Boolean type: ntp.enabled
    let ntp = system.get("ntp").expect("system should contain 'ntp'");
    assert_eq!(
        ntp.get("enabled").and_then(|v| v.as_bool()),
        Some(true),
        "ntp.enabled (boolean) should round-trip"
    );

    // Integer type: ntp.server[0].udp.port
    let servers = ntp
        .get("server")
        .and_then(|s| s.as_array())
        .expect("ntp should contain 'server' array");

    let ntp1 = servers
        .iter()
        .find(|s| s.get("name").and_then(|n| n.as_str()) == Some("ntp1.example.com"));
    assert!(
        ntp1.is_some(),
        "NTP server 'ntp1.example.com' should be present"
    );

    let udp_port = ntp1
        .unwrap()
        .get("udp")
        .and_then(|u| u.get("port"))
        .and_then(|p| p.as_u64());
    assert_eq!(
        udp_port,
        Some(123),
        "NTP UDP port (integer) should round-trip"
    );

    // Boolean type: ntp.server[0].prefer
    let prefer = ntp1.unwrap().get("prefer").and_then(|p| p.as_bool());
    assert_eq!(
        prefer,
        Some(true),
        "NTP server prefer (boolean) should round-trip"
    );

    // Teardown
    fixture_mgr
        .teardown()
        .await
        .expect("Failed to teardown fixtures");
    harness.stop().await.expect("Failed to stop emulator");
}
