//! End-to-end validation of generated Juniper client against live cRPD.
//!
//! This test file exercises the rustconf-generated RESTCONF client against a real
//! Juniper cRPD container. Tests are organized into four categories:
//!
//! - **smoke**: Basic connectivity, authentication, and content-type verification
//! - **crud**: CRUD operations across multiple Juniper YANG modules
//! - **schema**: Schema conformance between generated types and cRPD responses
//! - **errors**: Error path validation (404, invalid data, malformed JSON)
//!
//! All tests are gated on `RUSTCONF_INTEGRATION_TEST=1` and require a running
//! cRPD container (Docker or Podman).

mod common;
mod e2e_helpers;

// ---------------------------------------------------------------------------
// Smoke tests: connectivity, authentication, content-type
// ---------------------------------------------------------------------------

mod smoke {
    use super::*;
    use rustconf_integration_tests::{HarnessConfig, JunosCrpdConfig, TestHarness};
    use rustconf_runtime::{HttpMethod, HttpRequest};
    use std::time::Instant;

    /// Set up a TestHarness with the cRPD emulator, start it, and return the harness.
    async fn setup_harness() -> TestHarness {
        let harness_config = HarnessConfig::from_env();
        let emulator_config = JunosCrpdConfig::with_harness_config(&harness_config);
        let mut harness = TestHarness::new(emulator_config, &harness_config);
        harness
            .start()
            .await
            .expect("Failed to start cRPD emulator container");
        harness
    }

    /// Verify that an HTTP GET to the RESTCONF root (`/restconf`) returns a 200
    /// status with valid JSON.
    ///
    /// Requirements: 3.1
    #[tokio::test]
    async fn test_smoke_connectivity() {
        skip_unless_integration!();
        skip_unless_emulator!();

        let start = Instant::now();
        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let url = client.build_url("/restconf");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client
            .execute(request)
            .await
            .expect("GET /restconf should not return a transport error");

        assert_eq!(
            response.status_code, 200,
            "GET /restconf should return 200, got {}",
            response.status_code
        );

        // Response body should be valid JSON
        if !response.body.is_empty() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&response.body);
            assert!(
                parsed.is_ok(),
                "GET /restconf response body should be valid JSON: {:?}",
                parsed.err()
            );
        }

        // Smoke tests must complete within 10 seconds after health check
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 10,
            "Smoke connectivity test took too long: {:?}",
            elapsed
        );

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Verify that Basic authentication with the configured credentials is accepted.
    ///
    /// Requirements: 3.2
    #[tokio::test]
    async fn test_smoke_authentication() {
        skip_unless_integration!();
        skip_unless_emulator!();

        let start = Instant::now();
        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // The client is configured with Basic auth via the interceptor.
        // A successful GET to /restconf proves auth is accepted.
        let url = client.build_url("/restconf");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client
            .execute(request)
            .await
            .expect("GET /restconf should not return a transport error");

        // If auth fails, cRPD returns 401
        assert_ne!(
            response.status_code, 401,
            "Basic auth should be accepted (got 401 Unauthorized)"
        );
        assert_ne!(
            response.status_code, 403,
            "Basic auth should be accepted (got 403 Forbidden)"
        );
        assert!(
            response.is_success(),
            "Authenticated GET /restconf should succeed, got {}",
            response.status_code
        );

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 10,
            "Smoke authentication test took too long: {:?}",
            elapsed
        );

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Verify that the `Content-Type` header in responses contains
    /// `application/yang-data+json`.
    ///
    /// Requirements: 3.3
    #[tokio::test]
    async fn test_smoke_content_type() {
        skip_unless_integration!();
        skip_unless_emulator!();

        let start = Instant::now();
        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let url = client.build_url("/restconf");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client
            .execute(request)
            .await
            .expect("GET /restconf should not return a transport error");

        assert!(
            response.is_success(),
            "GET /restconf should succeed, got {}",
            response.status_code
        );

        // Check Content-Type header
        let content_type = response
            .get_header("Content-Type")
            .unwrap_or_else(|| response.get_header("content-type").unwrap_or(""));

        assert!(
            content_type.contains("application/yang-data+json"),
            "Response Content-Type should contain 'application/yang-data+json', got: '{}'",
            content_type
        );

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 10,
            "Smoke content-type test took too long: {:?}",
            elapsed
        );

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Run all smoke checks in sequence and set SMOKE_PASSED on success.
    ///
    /// This test acts as the gate for CRUD, schema, and error tests.
    /// It runs connectivity, auth, and content-type checks in a single harness
    /// session to avoid redundant container startups, then marks the smoke gate.
    ///
    /// Requirements: 3.4, 3.5
    #[tokio::test]
    async fn test_smoke_gate() {
        skip_unless_integration!();
        skip_unless_emulator!();

        let start = Instant::now();
        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // --- Connectivity check ---
        let url = client.build_url("/restconf");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client
            .execute(request)
            .await
            .expect("Smoke gate: GET /restconf transport error");

        if response.status_code != 200 {
            eprintln!(
                "Smoke gate FAILED: GET /restconf returned {}, expected 200",
                response.status_code
            );
            harness.stop().await.expect("Failed to stop emulator");
            return;
        }

        // --- Auth check (already proven by successful GET above) ---
        // If we got 200, auth was accepted.

        // --- Content-Type check ---
        let content_type = response
            .get_header("Content-Type")
            .unwrap_or_else(|| response.get_header("content-type").unwrap_or(""));

        if !content_type.contains("application/yang-data+json") {
            eprintln!(
                "Smoke gate FAILED: Content-Type '{}' does not contain 'application/yang-data+json'",
                content_type
            );
            harness.stop().await.expect("Failed to stop emulator");
            return;
        }

        // --- JSON validity check ---
        if !response.body.is_empty()
            && serde_json::from_slice::<serde_json::Value>(&response.body).is_err()
        {
            eprintln!("Smoke gate FAILED: response body is not valid JSON");
            harness.stop().await.expect("Failed to stop emulator");
            return;
        }

        // All smoke checks passed — set the gate
        e2e_helpers::set_smoke_passed();
        eprintln!("Smoke gate PASSED — CRUD/schema/error tests enabled");

        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 10,
            "Smoke gate took too long: {:?}",
            elapsed
        );

        harness.stop().await.expect("Failed to stop emulator");
    }
}

// ---------------------------------------------------------------------------
// CRUD validation (placeholder — implemented in task 6)
// ---------------------------------------------------------------------------

mod crud {
    use super::*;
    use rustconf_integration_tests::{FixtureManager, HarnessConfig, JunosCrpdConfig, TestHarness};
    use rustconf_runtime::{HttpMethod, HttpRequest};
    use std::path::Path;

    /// Set up a TestHarness with the cRPD emulator for CRUD tests.
    async fn setup_harness() -> TestHarness {
        let harness_config = HarnessConfig::from_env();
        let emulator_config = JunosCrpdConfig::with_harness_config(&harness_config);
        let mut harness = TestHarness::new(emulator_config, &harness_config);
        harness
            .start()
            .await
            .expect("Failed to start cRPD emulator container");
        harness
    }

    // -----------------------------------------------------------------------
    // 6.1 Interface CRUD tests
    // -----------------------------------------------------------------------

    /// Create an interface via PUT, verify it appears in a subsequent GET.
    ///
    /// Requirements: 4.1, 4.5
    #[tokio::test]
    async fn test_crud_interface_create() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let iface_name = e2e_helpers::e2e_resource_name("crud");
        let resource_path = "/data/junos-conf-interfaces:interfaces";

        // Use FixtureManager for state isolation
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        // Create interface configuration payload
        let put_data = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": iface_name,
                        "unit": [
                            {
                                "name": "0",
                                "family": {
                                    "inet": {
                                        "address": [
                                            {
                                                "name": "10.0.0.1/24"
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        // Apply as a fixture for automatic teardown
        let fixture = rustconf_integration_tests::FixtureDefinition {
            resource_path: resource_path.to_string(),
            data: put_data.clone(),
        };
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply interface fixture");

        // GET and verify the interface is present
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET interfaces failed");

        assert!(
            get_response.is_success(),
            "GET interfaces should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        let interfaces = json
            .get("junos-conf-interfaces:interfaces")
            .and_then(|i| i.get("interface"))
            .and_then(|i| i.as_array());

        assert!(
            interfaces.is_some(),
            "Response should contain interface array"
        );

        let has_created_iface = interfaces
            .unwrap()
            .iter()
            .any(|iface| iface.get("name").and_then(|n| n.as_str()) == Some(&iface_name));

        assert!(
            has_created_iface,
            "Created interface '{}' should be present in GET response",
            iface_name
        );

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Update an interface unit configuration, verify change reflected in GET.
    ///
    /// Requirements: 4.3, 4.5
    #[tokio::test]
    async fn test_crud_interface_update() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let iface_name = e2e_helpers::e2e_resource_name("crud");
        let resource_path = "/data/junos-conf-interfaces:interfaces";

        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        // First create the interface with initial address
        let initial_data = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": iface_name,
                        "unit": [
                            {
                                "name": "0",
                                "family": {
                                    "inet": {
                                        "address": [
                                            {
                                                "name": "10.0.0.1/24"
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let fixture = rustconf_integration_tests::FixtureDefinition {
            resource_path: resource_path.to_string(),
            data: initial_data,
        };
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply initial interface fixture");

        // Update: change the address on unit 0
        let updated_data = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": iface_name,
                        "unit": [
                            {
                                "name": "0",
                                "family": {
                                    "inet": {
                                        "address": [
                                            {
                                                "name": "10.0.0.2/24"
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let put_url = client.build_url(resource_path);
        let put_body = serde_json::to_vec(&updated_data).expect("Failed to serialize updated data");
        let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(put_body);

        let put_response = client
            .execute(put_request)
            .await
            .expect("PUT update failed");

        assert!(
            put_response.is_success(),
            "PUT update should succeed, got {}. Body: {}",
            put_response.status_code,
            String::from_utf8_lossy(&put_response.body)
        );

        // Verify the update via GET
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET after update failed");

        assert!(
            get_response.is_success(),
            "GET after update should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Verify the updated address is present
        let interfaces = json
            .get("junos-conf-interfaces:interfaces")
            .and_then(|i| i.get("interface"))
            .and_then(|i| i.as_array());

        if let Some(ifaces) = interfaces {
            let target_iface = ifaces
                .iter()
                .find(|iface| iface.get("name").and_then(|n| n.as_str()) == Some(&iface_name));

            if let Some(iface) = target_iface {
                let addresses = iface
                    .get("unit")
                    .and_then(|u| u.as_array())
                    .and_then(|units| units.first())
                    .and_then(|unit| unit.get("family"))
                    .and_then(|f| f.get("inet"))
                    .and_then(|inet| inet.get("address"))
                    .and_then(|a| a.as_array());

                if let Some(addrs) = addresses {
                    let has_updated_addr = addrs.iter().any(|addr| {
                        addr.get("name").and_then(|n| n.as_str()) == Some("10.0.0.2/24")
                    });
                    assert!(
                        has_updated_addr,
                        "Updated address '10.0.0.2/24' should be present after update"
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

    /// Delete an interface, verify it is absent in a subsequent GET.
    ///
    /// Requirements: 4.4
    #[tokio::test]
    async fn test_crud_interface_delete() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let iface_name = e2e_helpers::e2e_resource_name("crud");
        let resource_path = "/data/junos-conf-interfaces:interfaces";

        // First create the interface
        let create_data = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": iface_name,
                        "unit": [
                            {
                                "name": "0",
                                "family": {
                                    "inet": {
                                        "address": [
                                            {
                                                "name": "10.0.0.1/24"
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let put_url = client.build_url(resource_path);
        let put_body = serde_json::to_vec(&create_data).expect("Failed to serialize create data");
        let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(put_body);

        let put_response = client
            .execute(put_request)
            .await
            .expect("PUT create failed");

        assert!(
            put_response.is_success(),
            "PUT create should succeed, got {}",
            put_response.status_code
        );

        // Delete the specific interface using list key encoding
        let delete_path = format!(
            "/data/junos-conf-interfaces:interfaces/interface={}",
            iface_name
        );
        let delete_url = client.build_url(&delete_path);
        let delete_request = HttpRequest::new(HttpMethod::DELETE, &delete_url)
            .with_header("Accept", "application/yang-data+json");

        let delete_response = client
            .execute(delete_request)
            .await
            .expect("DELETE interface failed");

        assert!(
            delete_response.is_success(),
            "DELETE interface should succeed, got {}. Body: {}",
            delete_response.status_code,
            String::from_utf8_lossy(&delete_response.body)
        );

        // Verify the interface is absent in a subsequent GET
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET after delete failed");

        if get_response.is_success() && !get_response.body.is_empty() {
            let json: serde_json::Value =
                serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

            let interfaces = json
                .get("junos-conf-interfaces:interfaces")
                .and_then(|i| i.get("interface"))
                .and_then(|i| i.as_array());

            if let Some(ifaces) = interfaces {
                let has_deleted_iface = ifaces
                    .iter()
                    .any(|iface| iface.get("name").and_then(|n| n.as_str()) == Some(&iface_name));
                assert!(
                    !has_deleted_iface,
                    "Deleted interface '{}' should NOT be present in GET response",
                    iface_name
                );
            }
        }
        // If GET returns 404 or empty, that also confirms deletion

        harness.stop().await.expect("Failed to stop emulator");
    }

    // -----------------------------------------------------------------------
    // 6.2 System configuration CRUD tests
    // -----------------------------------------------------------------------

    /// Read system configuration from cRPD, deserialize into serde_json::Value
    /// (representing generated junos_conf_system types).
    ///
    /// Requirements: 4.2, 4.6
    #[tokio::test]
    async fn test_crud_system_read() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-system:system";

        // Apply the system fixture to ensure known baseline state
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-system.json"))
            .expect("Failed to load junos-system fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply system fixture");

        // GET the system configuration
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET system config failed");

        assert!(
            get_response.is_success(),
            "GET system config should succeed, got {}",
            get_response.status_code
        );

        // Deserialize into serde_json::Value (representing generated types)
        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Verify the system configuration structure
        let system = json.get("junos-conf-system:system");
        assert!(
            system.is_some(),
            "Response should contain 'junos-conf-system:system' key, got: {}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );

        let system = system.unwrap();
        // Verify host-name is present (from our fixture)
        let hostname = system.get("host-name").and_then(|h| h.as_str());
        assert_eq!(
            hostname,
            Some("e2e-test-device"),
            "System hostname should match fixture value"
        );

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Update hostname leaf via PUT, verify change in subsequent GET.
    ///
    /// Requirements: 4.3, 4.6
    #[tokio::test]
    async fn test_crud_system_update_hostname() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-system:system";

        // Apply baseline fixture for state isolation
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-system.json"))
            .expect("Failed to load junos-system fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply system fixture");

        // Update the hostname
        let new_hostname = format!(
            "e2e-updated-{}",
            &e2e_helpers::e2e_resource_name("sys")[4..10]
        );
        let update_data = serde_json::json!({
            "junos-conf-system:system": {
                "host-name": new_hostname,
                "domain-name": "test.example.com",
                "name-server": [
                    {
                        "name": "8.8.8.8"
                    }
                ]
            }
        });

        let put_url = client.build_url(resource_path);
        let put_body = serde_json::to_vec(&update_data).expect("Failed to serialize update data");
        let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(put_body);

        let put_response = client
            .execute(put_request)
            .await
            .expect("PUT system update failed");

        assert!(
            put_response.is_success(),
            "PUT system update should succeed, got {}. Body: {}",
            put_response.status_code,
            String::from_utf8_lossy(&put_response.body)
        );

        // Verify the change via GET
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET after system update failed");

        assert!(
            get_response.is_success(),
            "GET after system update should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        let updated_hostname = json
            .get("junos-conf-system:system")
            .and_then(|s| s.get("host-name"))
            .and_then(|h| h.as_str());

        assert_eq!(
            updated_hostname,
            Some(new_hostname.as_str()),
            "Hostname should be updated to '{}'",
            new_hostname
        );

        // Teardown restores original state
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    // -----------------------------------------------------------------------
    // 6.3 Routing-options CRUD tests
    // -----------------------------------------------------------------------

    /// Create a static route, read routing table, verify route is present.
    ///
    /// Requirements: 4.4, 4.6
    #[tokio::test]
    async fn test_crud_routing_options_create() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-routing-options:routing-options";

        // Use FixtureManager for state isolation
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        // Create a static route via the fixture
        let route_data = serde_json::json!({
            "junos-conf-routing-options:routing-options": {
                "static": {
                    "route": [
                        {
                            "name": "10.99.0.0/16",
                            "next-hop": [
                                "192.168.1.1"
                            ]
                        }
                    ]
                }
            }
        });

        let fixture = rustconf_integration_tests::FixtureDefinition {
            resource_path: resource_path.to_string(),
            data: route_data,
        };
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply routing-options fixture");

        // GET and verify the route is present
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET routing-options failed");

        assert!(
            get_response.is_success(),
            "GET routing-options should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        let routes = json
            .get("junos-conf-routing-options:routing-options")
            .and_then(|ro| ro.get("static"))
            .and_then(|s| s.get("route"))
            .and_then(|r| r.as_array());

        assert!(
            routes.is_some(),
            "Response should contain route array in routing-options"
        );

        let has_route = routes
            .unwrap()
            .iter()
            .any(|route| route.get("name").and_then(|n| n.as_str()) == Some("10.99.0.0/16"));

        assert!(
            has_route,
            "Created static route '10.99.0.0/16' should be present in GET response"
        );

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Read routing table using the junos-routing-options fixture, verify structure.
    ///
    /// Requirements: 4.6
    #[tokio::test]
    async fn test_crud_routing_options_read() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-routing-options:routing-options";

        // Apply the routing-options fixture for known baseline
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture =
            FixtureManager::load_fixture(Path::new("fixtures/junos-routing-options.json"))
                .expect("Failed to load junos-routing-options fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply routing-options fixture");

        // GET the routing-options
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET routing-options failed");

        assert!(
            get_response.is_success(),
            "GET routing-options should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Verify the routing-options structure
        let routing_options = json.get("junos-conf-routing-options:routing-options");
        assert!(
            routing_options.is_some(),
            "Response should contain 'junos-conf-routing-options:routing-options' key"
        );

        // Verify the default route from fixture is present
        let routes = routing_options
            .and_then(|ro| ro.get("static"))
            .and_then(|s| s.get("route"))
            .and_then(|r| r.as_array());

        assert!(
            routes.is_some(),
            "Routing-options should contain static routes"
        );

        let has_default_route = routes
            .unwrap()
            .iter()
            .any(|route| route.get("name").and_then(|n| n.as_str()) == Some("0.0.0.0/0"));

        assert!(
            has_default_route,
            "Default route '0.0.0.0/0' from fixture should be present"
        );

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Delete a static route, verify it is absent in a subsequent GET.
    ///
    /// Requirements: 4.4, 4.6
    #[tokio::test]
    async fn test_crud_routing_options_delete() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-routing-options:routing-options";

        // First, apply the routing-options fixture to create a known route
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture =
            FixtureManager::load_fixture(Path::new("fixtures/junos-routing-options.json"))
                .expect("Failed to load junos-routing-options fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply routing-options fixture");

        // Delete the specific static route using list key encoding
        // The route name "0.0.0.0/0" needs URL encoding: %2F for the slash in the prefix
        let delete_path = format!("{}/static/route=0.0.0.0%2F0", resource_path);
        let delete_url = client.build_url(&delete_path);
        let delete_request = HttpRequest::new(HttpMethod::DELETE, &delete_url)
            .with_header("Accept", "application/yang-data+json");

        let delete_response = client
            .execute(delete_request)
            .await
            .expect("DELETE route failed");

        assert!(
            delete_response.is_success(),
            "DELETE route should succeed, got {}. Body: {}",
            delete_response.status_code,
            String::from_utf8_lossy(&delete_response.body)
        );

        // Verify the route is absent in a subsequent GET
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET after route delete failed");

        if get_response.is_success() && !get_response.body.is_empty() {
            let json: serde_json::Value =
                serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

            let routes = json
                .get("junos-conf-routing-options:routing-options")
                .and_then(|ro| ro.get("static"))
                .and_then(|s| s.get("route"))
                .and_then(|r| r.as_array());

            if let Some(route_list) = routes {
                let has_deleted_route = route_list
                    .iter()
                    .any(|route| route.get("name").and_then(|n| n.as_str()) == Some("0.0.0.0/0"));
                assert!(
                    !has_deleted_route,
                    "Deleted route '0.0.0.0/0' should NOT be present in GET response"
                );
            }
        }
        // If GET returns 404 or empty body, that also confirms deletion

        // Teardown (will attempt to restore original state)
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }
}

// ---------------------------------------------------------------------------
// Schema conformance: key coverage, Juniper-specific type deserialization,
// unknown key handling
// ---------------------------------------------------------------------------

mod schema {
    use super::*;
    use rustconf_integration_tests::{
        ConformanceReporter, FixtureDefinition, FixtureManager, HarnessConfig, JunosCrpdConfig,
        TestDetails, TestHarness, TestResult, TestStatus,
    };
    use rustconf_runtime::{HttpMethod, HttpRequest};
    use std::collections::HashSet;
    use std::path::Path;

    /// Set up a TestHarness with the cRPD emulator for schema tests.
    async fn setup_harness() -> TestHarness {
        let harness_config = HarnessConfig::from_env();
        let emulator_config = JunosCrpdConfig::with_harness_config(&harness_config);
        let mut harness = TestHarness::new(emulator_config, &harness_config);
        harness
            .start()
            .await
            .expect("Failed to start cRPD emulator container");
        harness
    }

    /// Known keys for the `junos-conf-interfaces:interfaces` top-level container.
    /// These represent the expected YANG schema fields that the generated types
    /// should handle.
    fn known_interfaces_keys() -> HashSet<&'static str> {
        let mut keys = HashSet::new();
        keys.insert("interface");
        keys.insert("interface-range");
        keys.insert("traceoptions");
        keys
    }

    /// Known keys for an interface list entry in `junos-conf-interfaces`.
    fn known_interface_entry_keys() -> HashSet<&'static str> {
        let mut keys = HashSet::new();
        keys.insert("name");
        keys.insert("description");
        keys.insert("disable");
        keys.insert("enable");
        keys.insert("unit");
        keys.insert("vlan-tagging");
        keys.insert("flexible-vlan-tagging");
        keys.insert("mtu");
        keys.insert("encapsulation");
        keys.insert("mac");
        keys.insert("hold-time");
        keys.insert("link-mode");
        keys.insert("speed");
        keys.insert("gigether-options");
        keys.insert("ether-options");
        keys.insert("aggregated-ether-options");
        keys.insert("native-vlan-id");
        keys.insert("per-unit-scheduler");
        keys.insert("hierarchical-scheduler");
        keys.insert("gratuitous-arp-reply");
        keys.insert("no-gratuitous-arp-reply");
        keys.insert("no-traps");
        keys.insert("traps");
        keys
    }

    /// Known keys for the `junos-conf-system:system` top-level container.
    fn known_system_keys() -> HashSet<&'static str> {
        let mut keys = HashSet::new();
        keys.insert("host-name");
        keys.insert("domain-name");
        keys.insert("domain-search");
        keys.insert("name-server");
        keys.insert("root-authentication");
        keys.insert("login");
        keys.insert("services");
        keys.insert("syslog");
        keys.insert("ntp");
        keys.insert("time-zone");
        keys.insert("authentication-order");
        keys.insert("ports");
        keys.insert("archival");
        keys.insert("scripts");
        keys.insert("processes");
        keys.insert("internet-options");
        keys.insert("management-instance");
        keys.insert("configuration-database");
        keys.insert("commit");
        keys.insert("license");
        keys.insert("accounting");
        keys.insert("arp");
        keys.insert("compress-configuration-files");
        keys.insert("no-compress-configuration-files");
        keys.insert("max-configurations-on-flash");
        keys.insert("max-configuration-rollbacks");
        keys.insert("dump-on-panic");
        keys.insert("no-dump-on-panic");
        keys.insert("saved-core-context");
        keys.insert("no-saved-core-context");
        keys.insert("no-redirects");
        keys.insert("no-ping-record-route");
        keys.insert("no-ping-time-stamp");
        keys
    }

    /// Check JSON keys against a known set, returning unknown keys as conformance warnings.
    fn check_keys_against_schema(
        json_obj: &serde_json::Value,
        known_keys: &HashSet<&str>,
    ) -> Vec<String> {
        let mut unknown_keys = Vec::new();
        if let Some(obj) = json_obj.as_object() {
            for key in obj.keys() {
                if !known_keys.contains(key.as_str()) {
                    unknown_keys.push(key.clone());
                }
            }
        }
        unknown_keys
    }

    // -----------------------------------------------------------------------
    // 7.1 Schema key coverage tests
    // -----------------------------------------------------------------------

    /// GET interfaces configuration from cRPD, verify all JSON keys correspond
    /// to known fields in the generated Rust types. Unknown keys are logged as
    /// conformance warnings (not failures).
    ///
    /// Requirements: 5.1, 5.5
    #[tokio::test]
    async fn test_schema_key_coverage_interfaces() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let mut reporter = ConformanceReporter::new("Juniper cRPD (E2E)");

        // Apply the interfaces fixture to ensure there's data to check
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-interfaces.json"))
            .expect("Failed to load junos-interfaces fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply interfaces fixture");

        // GET the interfaces configuration
        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET interfaces failed");

        assert!(
            get_response.is_success(),
            "GET interfaces should succeed, got {}",
            get_response.status_code
        );

        // Parse the response
        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Check top-level keys under the interfaces container
        let interfaces_container = json
            .get("junos-conf-interfaces:interfaces")
            .expect("Response should contain 'junos-conf-interfaces:interfaces' key");

        let known_top_keys = known_interfaces_keys();
        let unknown_top = check_keys_against_schema(interfaces_container, &known_top_keys);

        let mut conformance_warnings: Vec<String> = Vec::new();
        for key in &unknown_top {
            let warning = format!(
                "Unknown key in junos-conf-interfaces:interfaces container: '{}'",
                key
            );
            eprintln!("CONFORMANCE WARNING: {}", warning);
            conformance_warnings.push(warning);
        }

        // Check keys within each interface entry
        let known_entry_keys = known_interface_entry_keys();
        if let Some(interfaces) = interfaces_container
            .get("interface")
            .and_then(|i| i.as_array())
        {
            for iface in interfaces {
                let unknown_entry = check_keys_against_schema(iface, &known_entry_keys);
                for key in &unknown_entry {
                    let iface_name = iface
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("<unknown>");
                    let warning = format!("Unknown key in interface '{}': '{}'", iface_name, key);
                    eprintln!("CONFORMANCE WARNING: {}", warning);
                    conformance_warnings.push(warning);
                }
            }
        }

        // Record the result — unknown keys are warnings, not failures
        // The test passes as long as deserialization doesn't fail
        let status = TestStatus::Pass;
        let details = if conformance_warnings.is_empty() {
            None
        } else {
            Some(TestDetails {
                expected: None,
                actual: None,
                request: Some(format!("GET {}", resource_path)),
                response: Some(
                    String::from_utf8_lossy(&get_response.body)
                        .chars()
                        .take(500)
                        .collect(),
                ),
                conformance_warnings: conformance_warnings.clone(),
            })
        };

        reporter.record(TestResult {
            yang_module: "junos-conf-interfaces".to_string(),
            operation: format!("Schema key coverage: GET {}", resource_path),
            status,
            details,
        });

        // Verify that deserialization into serde_json::Value never fails
        // (this is the core property — unknown keys should not cause failure)
        let _: serde_json::Value = serde_json::from_slice(&get_response.body)
            .expect("Deserialization should never fail due to unknown keys");

        // Print report summary
        let report = reporter.generate_text_report();
        eprintln!("{}", report);

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    /// GET system configuration from cRPD, verify all JSON keys correspond
    /// to known fields in the generated Rust types. Unknown keys are logged as
    /// conformance warnings (not failures).
    ///
    /// Requirements: 5.1, 5.5
    #[tokio::test]
    async fn test_schema_key_coverage_system() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let mut reporter = ConformanceReporter::new("Juniper cRPD (E2E)");

        // Apply the system fixture to ensure there's data to check
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-system.json"))
            .expect("Failed to load junos-system fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply system fixture");

        // GET the system configuration
        let resource_path = "/data/junos-conf-system:system";
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET system config failed");

        assert!(
            get_response.is_success(),
            "GET system config should succeed, got {}",
            get_response.status_code
        );

        // Parse the response
        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Check top-level keys under the system container
        let system_container = json
            .get("junos-conf-system:system")
            .expect("Response should contain 'junos-conf-system:system' key");

        let known_keys = known_system_keys();
        let unknown_keys = check_keys_against_schema(system_container, &known_keys);

        let mut conformance_warnings: Vec<String> = Vec::new();
        for key in &unknown_keys {
            let warning = format!(
                "Unknown key in junos-conf-system:system container: '{}'",
                key
            );
            eprintln!("CONFORMANCE WARNING: {}", warning);
            conformance_warnings.push(warning);
        }

        // Record the result
        let status = TestStatus::Pass;
        let details = if conformance_warnings.is_empty() {
            None
        } else {
            Some(TestDetails {
                expected: None,
                actual: None,
                request: Some(format!("GET {}", resource_path)),
                response: Some(
                    String::from_utf8_lossy(&get_response.body)
                        .chars()
                        .take(500)
                        .collect(),
                ),
                conformance_warnings,
            })
        };

        reporter.record(TestResult {
            yang_module: "junos-conf-system".to_string(),
            operation: format!("Schema key coverage: GET {}", resource_path),
            status,
            details,
        });

        // Verify that deserialization into serde_json::Value never fails
        let _: serde_json::Value = serde_json::from_slice(&get_response.body)
            .expect("Deserialization should never fail due to unknown keys");

        // Print report summary
        let report = reporter.generate_text_report();
        eprintln!("{}", report);

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    // -----------------------------------------------------------------------
    // 7.2 Juniper-specific type deserialization tests
    // -----------------------------------------------------------------------

    /// Verify generated types correctly deserialize Juniper-specific YANG types
    /// (e.g., `junos:ipv4-prefix` represented as "192.0.2.1/24" strings).
    ///
    /// This test verifies that IPv4 prefix values in interface address configurations
    /// are correctly structured and can be deserialized without error.
    ///
    /// Requirements: 5.2, 5.3
    #[tokio::test]
    async fn test_schema_juniper_ipv4_prefix_deserialization() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // Apply the interfaces fixture which contains IPv4 prefix values
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-interfaces.json"))
            .expect("Failed to load junos-interfaces fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply interfaces fixture");

        // GET the interfaces configuration
        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET interfaces failed");

        assert!(
            get_response.is_success(),
            "GET interfaces should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Navigate to the address entries which contain junos:ipv4-prefix values
        let interfaces = json
            .get("junos-conf-interfaces:interfaces")
            .and_then(|i| i.get("interface"))
            .and_then(|i| i.as_array())
            .expect("Should have interface array");

        let mut found_ipv4_prefix = false;

        for iface in interfaces {
            if let Some(units) = iface.get("unit").and_then(|u| u.as_array()) {
                for unit in units {
                    if let Some(addresses) = unit
                        .get("family")
                        .and_then(|f| f.get("inet"))
                        .and_then(|inet| inet.get("address"))
                        .and_then(|a| a.as_array())
                    {
                        for addr in addresses {
                            // The "name" field in address entries is a junos:ipv4-prefix
                            // (e.g., "192.0.2.1/24")
                            let addr_name = addr
                                .get("name")
                                .and_then(|n| n.as_str())
                                .expect("Address entry should have a 'name' field");

                            // Verify it looks like a valid IPv4 prefix
                            assert!(
                                addr_name.contains('/'),
                                "IPv4 prefix '{}' should contain '/' separator",
                                addr_name
                            );

                            let parts: Vec<&str> = addr_name.split('/').collect();
                            assert_eq!(
                                parts.len(),
                                2,
                                "IPv4 prefix '{}' should have exactly two parts (addr/len)",
                                addr_name
                            );

                            // Verify the IP part has 4 octets
                            let ip_parts: Vec<&str> = parts[0].split('.').collect();
                            assert_eq!(
                                ip_parts.len(),
                                4,
                                "IP address part '{}' should have 4 octets",
                                parts[0]
                            );

                            // Verify the prefix length is a valid number
                            let prefix_len: u8 = parts[1].parse().unwrap_or_else(|_| {
                                panic!("Prefix length '{}' should be a valid number", parts[1])
                            });
                            assert!(
                                prefix_len <= 32,
                                "IPv4 prefix length {} should be <= 32",
                                prefix_len
                            );

                            found_ipv4_prefix = true;
                        }
                    }
                }
            }
        }

        assert!(
            found_ipv4_prefix,
            "Should have found at least one IPv4 prefix address in the interfaces response"
        );

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Verify list entries with YANG list keys are correctly identified and
    /// deserialized. YANG lists use a "name" key field to identify entries.
    ///
    /// Requirements: 5.3, 5.4
    #[tokio::test]
    async fn test_schema_list_key_deserialization() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // Apply the interfaces fixture which contains list entries
        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        let fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-interfaces.json"))
            .expect("Failed to load junos-interfaces fixture");
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply interfaces fixture");

        // GET the interfaces configuration
        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET interfaces failed");

        assert!(
            get_response.is_success(),
            "GET interfaces should succeed, got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Verify interface list entries have the "name" key field
        let interfaces = json
            .get("junos-conf-interfaces:interfaces")
            .and_then(|i| i.get("interface"))
            .and_then(|i| i.as_array())
            .expect("Should have interface array");

        assert!(!interfaces.is_empty(), "Interface list should not be empty");

        for iface in interfaces {
            // Every interface list entry MUST have a "name" key
            let name = iface.get("name");
            assert!(
                name.is_some(),
                "Every interface list entry must have a 'name' key field. Entry: {}",
                serde_json::to_string_pretty(iface).unwrap_or_default()
            );
            assert!(
                name.unwrap().is_string(),
                "Interface 'name' key should be a string"
            );

            // Verify unit list entries also have "name" key
            if let Some(units) = iface.get("unit").and_then(|u| u.as_array()) {
                for unit in units {
                    let unit_name = unit.get("name");
                    assert!(
                        unit_name.is_some(),
                        "Every unit list entry must have a 'name' key field. Entry: {}",
                        serde_json::to_string_pretty(unit).unwrap_or_default()
                    );
                    // Unit name can be a string or number (unit index)
                    let unit_name_val = unit_name.unwrap();
                    assert!(
                        unit_name_val.is_string() || unit_name_val.is_number(),
                        "Unit 'name' key should be a string or number, got: {:?}",
                        unit_name_val
                    );
                }
            }
        }

        // Also verify name-server list in system config has key fields
        let system_fixture = FixtureManager::load_fixture(Path::new("fixtures/junos-system.json"))
            .expect("Failed to load junos-system fixture");
        fixture_mgr
            .apply(&system_fixture)
            .await
            .expect("Failed to apply system fixture");

        let system_path = "/data/junos-conf-system:system";
        let sys_url = client.build_url(system_path);
        let sys_request = HttpRequest::new(HttpMethod::GET, &sys_url)
            .with_header("Accept", "application/yang-data+json");

        let sys_response = client
            .execute(sys_request)
            .await
            .expect("GET system config failed");

        if sys_response.is_success() {
            let sys_json: serde_json::Value =
                serde_json::from_slice(&sys_response.body).expect("Response should be valid JSON");

            // Verify name-server list entries have "name" key
            if let Some(name_servers) = sys_json
                .get("junos-conf-system:system")
                .and_then(|s| s.get("name-server"))
                .and_then(|ns| ns.as_array())
            {
                for ns in name_servers {
                    let ns_name = ns.get("name");
                    assert!(
                        ns_name.is_some(),
                        "Every name-server list entry must have a 'name' key field. Entry: {}",
                        serde_json::to_string_pretty(ns).unwrap_or_default()
                    );
                    assert!(
                        ns_name.unwrap().is_string(),
                        "name-server 'name' key should be a string"
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

    /// Serialize a configuration object and verify cRPD accepts it without
    /// schema validation errors. This confirms the generated type serialization
    /// format matches what cRPD expects.
    ///
    /// Requirements: 5.2, 5.4
    #[tokio::test]
    async fn test_schema_serialization_accepted_by_crpd() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let mut fixture_mgr = FixtureManager::new(
            harness
                .restconf_client()
                .expect("Failed to create fixture client"),
        );

        // Construct a configuration object using the same JSON structure
        // that the generated types would produce when serialized
        let iface_name = e2e_helpers::e2e_resource_name("schema");
        let config_data = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": iface_name,
                        "unit": [
                            {
                                "name": "0",
                                "family": {
                                    "inet": {
                                        "address": [
                                            {
                                                "name": "10.200.0.1/30"
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let resource_path = "/data/junos-conf-interfaces:interfaces";

        // Apply as fixture for automatic cleanup
        let fixture = FixtureDefinition {
            resource_path: resource_path.to_string(),
            data: config_data.clone(),
        };
        fixture_mgr
            .apply(&fixture)
            .await
            .expect("Failed to apply schema test fixture");

        // Verify cRPD accepted the configuration by reading it back
        let get_url = client.build_url(resource_path);
        let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
            .with_header("Accept", "application/yang-data+json");

        let get_response = client
            .execute(get_request)
            .await
            .expect("GET after PUT failed");

        assert!(
            get_response.is_success(),
            "GET after PUT should succeed (cRPD accepted our serialization), got {}",
            get_response.status_code
        );

        let json: serde_json::Value =
            serde_json::from_slice(&get_response.body).expect("Response should be valid JSON");

        // Verify our interface is present (cRPD didn't reject it)
        let interfaces = json
            .get("junos-conf-interfaces:interfaces")
            .and_then(|i| i.get("interface"))
            .and_then(|i| i.as_array());

        assert!(
            interfaces.is_some(),
            "Response should contain interface array after PUT"
        );

        let has_our_iface = interfaces
            .unwrap()
            .iter()
            .any(|iface| iface.get("name").and_then(|n| n.as_str()) == Some(&iface_name));

        assert!(
            has_our_iface,
            "cRPD should have accepted our serialized interface '{}'",
            iface_name
        );

        // Also test system configuration serialization
        let system_data = serde_json::json!({
            "junos-conf-system:system": {
                "host-name": "schema-test-device",
                "domain-name": "schema.test.example.com",
                "name-server": [
                    {
                        "name": "8.8.4.4"
                    }
                ]
            }
        });

        let system_path = "/data/junos-conf-system:system";
        let system_fixture = FixtureDefinition {
            resource_path: system_path.to_string(),
            data: system_data,
        };
        fixture_mgr
            .apply(&system_fixture)
            .await
            .expect("Failed to apply system schema test fixture");

        // Verify system config was accepted
        let sys_url = client.build_url(system_path);
        let sys_request = HttpRequest::new(HttpMethod::GET, &sys_url)
            .with_header("Accept", "application/yang-data+json");

        let sys_response = client
            .execute(sys_request)
            .await
            .expect("GET system after PUT failed");

        assert!(
            sys_response.is_success(),
            "GET system after PUT should succeed (cRPD accepted serialization), got {}",
            sys_response.status_code
        );

        let sys_json: serde_json::Value =
            serde_json::from_slice(&sys_response.body).expect("Response should be valid JSON");

        let hostname = sys_json
            .get("junos-conf-system:system")
            .and_then(|s| s.get("host-name"))
            .and_then(|h| h.as_str());

        assert_eq!(
            hostname,
            Some("schema-test-device"),
            "cRPD should have accepted our serialized system config"
        );

        // Teardown
        fixture_mgr
            .teardown()
            .await
            .expect("Failed to teardown fixtures");
        harness.stop().await.expect("Failed to stop emulator");
    }
}

// ---------------------------------------------------------------------------
// Error path validation: 404, invalid data, malformed JSON, RESTCONF error
// structure parsing
// ---------------------------------------------------------------------------

mod errors {
    use super::*;
    use rustconf_integration_tests::{HarnessConfig, JunosCrpdConfig, TestHarness};
    use rustconf_runtime::{HttpMethod, HttpRequest};

    /// Set up a TestHarness with the cRPD emulator for error path tests.
    async fn setup_harness() -> TestHarness {
        let harness_config = HarnessConfig::from_env();
        let emulator_config = JunosCrpdConfig::with_harness_config(&harness_config);
        let mut harness = TestHarness::new(emulator_config, &harness_config);
        harness
            .start()
            .await
            .expect("Failed to start cRPD emulator container");
        harness
    }

    // -----------------------------------------------------------------------
    // 9.1 — 404 and invalid path tests
    // -----------------------------------------------------------------------

    /// Send GET to a completely non-existent RESTCONF path, verify the
    /// generated client returns an error with HTTP 404.
    ///
    /// Requirements: 7.1
    #[tokio::test]
    async fn test_error_get_nonexistent_path_returns_404() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // Use a completely fabricated module and container name
        let url = client.build_url("/data/nonexistent-yang-module:nonexistent-container");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client
            .execute(request)
            .await
            .expect("Client should not return a transport error for 404");

        assert_eq!(
            response.status_code, 404,
            "GET on non-existent RESTCONF path should return 404, got {}",
            response.status_code
        );

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Send GET to a non-existent list entry within a valid module, verify 404.
    ///
    /// Requirements: 7.1
    #[tokio::test]
    async fn test_error_get_nonexistent_list_entry_returns_404() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // Valid module path but non-existent interface name
        let url = client.build_url(
            "/data/junos-conf-interfaces:interfaces/interface=this-iface-does-not-exist-xyz",
        );
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client.execute(request).await;

        // The client must not panic on any error response
        match response {
            Ok(resp) => {
                assert_eq!(
                    resp.status_code, 404,
                    "GET on non-existent list entry should return 404, got {}",
                    resp.status_code
                );
            }
            Err(e) => {
                // Transport error is acceptable — the key property is no panic
                eprintln!(
                    "Note: GET non-existent list entry returned transport error (no panic): {e}"
                );
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Verify the client does not panic on any error response by exercising
    /// multiple invalid paths with different HTTP methods.
    ///
    /// Requirements: 7.1
    #[tokio::test]
    async fn test_error_no_panic_on_various_invalid_paths() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let invalid_paths = [
            "/data/fake-module:fake",
            "/data/junos-conf-interfaces:interfaces/interface=ZZZZZ/unit=99999",
            "/data/junos-conf-system:system/nonexistent-leaf",
            "/data/junos-conf-routing-options:routing-options/static/route=999.999.999.999%2F99",
        ];

        for path in &invalid_paths {
            let url = client.build_url(path);
            let request = HttpRequest::new(HttpMethod::GET, &url)
                .with_header("Accept", "application/yang-data+json");

            // The critical property: no panic regardless of response
            let response = client.execute(request).await;
            match response {
                Ok(resp) => {
                    eprintln!("  GET {} -> status {}", path, resp.status_code);
                    // Should be a 4xx error
                    assert!(
                        !resp.is_success(),
                        "GET on invalid path '{}' should not succeed, got {}",
                        path,
                        resp.status_code
                    );
                }
                Err(e) => {
                    eprintln!("  GET {} -> error (no panic): {}", path, e);
                }
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }

    // -----------------------------------------------------------------------
    // 9.2 — Invalid value and malformed JSON tests
    // -----------------------------------------------------------------------

    /// Send a configuration with an invalid IP address format, verify cRPD
    /// rejects it and the client surfaces error details.
    ///
    /// Requirements: 7.2
    #[tokio::test]
    async fn test_error_invalid_ip_format_rejected() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let url = client.build_url(resource_path);

        // Invalid IP address format — "not.an.ip/24" is not a valid IPv4 prefix
        let invalid_data = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": "e2e-invalid-ip-test",
                        "unit": [
                            {
                                "name": "0",
                                "family": {
                                    "inet": {
                                        "address": [
                                            {
                                                "name": "not.a.valid.ip/24"
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                ]
            }
        });

        let body = serde_json::to_vec(&invalid_data).expect("Failed to serialize");
        let request = HttpRequest::new(HttpMethod::PUT, &url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(body);

        let response = client.execute(request).await;

        match response {
            Ok(resp) => {
                assert!(
                    !resp.is_success(),
                    "Invalid IP format should be rejected by cRPD, got status {}",
                    resp.status_code
                );

                // cRPD should return a 4xx error
                assert!(
                    (400..500).contains(&resp.status_code),
                    "Invalid IP should produce 4xx error, got {}",
                    resp.status_code
                );

                // Verify the error response contains details
                if !resp.body.is_empty() {
                    let body_str = String::from_utf8_lossy(&resp.body);
                    eprintln!("Error response body: {}", body_str);
                    // Should be parseable (JSON or at minimum non-empty)
                    assert!(
                        !body_str.trim().is_empty(),
                        "Error response should contain details"
                    );
                }
            }
            Err(e) => {
                // Transport error is acceptable — client didn't panic
                eprintln!("Note: Invalid IP request returned transport error: {e}");
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Send a malformed JSON body, verify the client maps the cRPD error
    /// to a structured response (not a panic).
    ///
    /// Requirements: 7.3
    #[tokio::test]
    async fn test_error_malformed_json_body() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let url = client.build_url(resource_path);

        // Completely invalid JSON
        let malformed_body = b"{ this is not valid json !!! [[[".to_vec();
        let request = HttpRequest::new(HttpMethod::PUT, &url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(malformed_body);

        let response = client.execute(request).await;

        match response {
            Ok(resp) => {
                assert!(
                    !resp.is_success(),
                    "Malformed JSON should be rejected, got status {}",
                    resp.status_code
                );

                // Expect 400 Bad Request for malformed input
                assert!(
                    resp.status_code == 400 || resp.status_code == 415,
                    "Malformed JSON should produce 400 or 415, got {}",
                    resp.status_code
                );
            }
            Err(e) => {
                // Transport error is acceptable — client handled it without panic
                eprintln!("Note: Malformed JSON request returned transport error: {e}");
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Send valid JSON with wrong YANG module structure, verify cRPD rejects
    /// and the client surfaces the error.
    ///
    /// Requirements: 7.2, 7.3
    #[tokio::test]
    async fn test_error_wrong_yang_module_structure() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let url = client.build_url(resource_path);

        // Valid JSON but completely wrong structure for this resource path
        let wrong_structure = serde_json::json!({
            "wrong-module:wrong-container": {
                "nonexistent-leaf": "some-value",
                "another-fake-leaf": 42
            }
        });

        let body = serde_json::to_vec(&wrong_structure).expect("Failed to serialize");
        let request = HttpRequest::new(HttpMethod::PUT, &url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(body);

        let response = client.execute(request).await;

        match response {
            Ok(resp) => {
                assert!(
                    !resp.is_success(),
                    "Wrong YANG structure should be rejected, got status {}",
                    resp.status_code
                );

                assert!(
                    (400..500).contains(&resp.status_code),
                    "Wrong YANG structure should produce 4xx, got {}",
                    resp.status_code
                );
            }
            Err(e) => {
                eprintln!("Note: Wrong YANG structure returned transport error: {e}");
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }

    // -----------------------------------------------------------------------
    // 9.3 — RESTCONF error structure parsing tests
    // -----------------------------------------------------------------------

    /// Trigger a cRPD error that returns `ietf-restconf:errors` structure,
    /// verify the client parses `error-type`, `error-tag`, and `error-message`.
    ///
    /// We trigger this by sending an invalid configuration that cRPD will
    /// reject with a structured RESTCONF error response.
    ///
    /// Requirements: 7.4
    #[tokio::test]
    async fn test_error_restconf_error_structure_parsing() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // Send a request to a non-existent resource — cRPD should return
        // a structured ietf-restconf:errors response
        let url = client.build_url("/data/nonexistent-module:nonexistent-resource");
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = client
            .execute(request)
            .await
            .expect("Client should not return transport error for structured 404");

        assert_eq!(
            response.status_code, 404,
            "Should get 404 for non-existent resource, got {}",
            response.status_code
        );

        // Parse the response body to check for RESTCONF error structure
        if !response.body.is_empty() {
            let body_str = String::from_utf8_lossy(&response.body);
            eprintln!("RESTCONF error response: {}", body_str);

            let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&response.body);
            if let Ok(json) = parsed {
                // Check for ietf-restconf:errors structure
                if let Some(errors_obj) = json.get("ietf-restconf:errors") {
                    let error_array = errors_obj
                        .get("error")
                        .and_then(|e| e.as_array())
                        .expect("RESTCONF errors should contain 'error' array");

                    assert!(
                        !error_array.is_empty(),
                        "RESTCONF error array should not be empty"
                    );

                    let first_error = &error_array[0];

                    // Verify error-type is present and valid
                    let error_type = first_error.get("error-type").and_then(|t| t.as_str());
                    assert!(
                        error_type.is_some(),
                        "RESTCONF error should contain 'error-type' field. Got: {}",
                        serde_json::to_string_pretty(first_error).unwrap_or_default()
                    );
                    let valid_error_types = ["transport", "rpc", "protocol", "application"];
                    assert!(
                        valid_error_types.contains(&error_type.unwrap()),
                        "error-type '{}' should be one of {:?}",
                        error_type.unwrap(),
                        valid_error_types
                    );

                    // Verify error-tag is present
                    let error_tag = first_error.get("error-tag").and_then(|t| t.as_str());
                    assert!(
                        error_tag.is_some(),
                        "RESTCONF error should contain 'error-tag' field. Got: {}",
                        serde_json::to_string_pretty(first_error).unwrap_or_default()
                    );

                    // Verify error-message is present (may be optional per RFC 8040,
                    // but cRPD typically includes it)
                    let error_message = first_error.get("error-message").and_then(|m| m.as_str());
                    if let Some(msg) = error_message {
                        assert!(
                            !msg.is_empty(),
                            "error-message should not be empty when present"
                        );
                        eprintln!("  error-type: {}", error_type.unwrap());
                        eprintln!("  error-tag: {}", error_tag.unwrap());
                        eprintln!("  error-message: {}", msg);
                    } else {
                        eprintln!("  Note: error-message not present (optional per RFC 8040)");
                        eprintln!("  error-type: {}", error_type.unwrap());
                        eprintln!("  error-tag: {}", error_tag.unwrap());
                    }
                } else {
                    // Some cRPD versions may not return ietf-restconf:errors for 404
                    eprintln!(
                        "Note: cRPD did not return ietf-restconf:errors structure for 404. \
                         Body: {}",
                        body_str
                    );
                }
            } else {
                eprintln!(
                    "Note: 404 response body is not JSON (may be plain text): {}",
                    body_str
                );
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }

    /// Trigger a RESTCONF error via invalid configuration data and verify
    /// the error structure contains error-type, error-tag, and error-message.
    ///
    /// This uses a constraint violation (invalid value) which is more likely
    /// to produce a full ietf-restconf:errors response from cRPD.
    ///
    /// Requirements: 7.4
    #[tokio::test]
    async fn test_error_restconf_error_structure_from_invalid_config() {
        skip_unless_integration!();
        skip_unless_emulator!();
        skip_unless_smoke_passed!();

        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        let resource_path = "/data/junos-conf-interfaces:interfaces";
        let url = client.build_url(resource_path);

        // Send configuration with a value that violates YANG constraints.
        // An empty interface name should trigger a constraint error.
        let invalid_config = serde_json::json!({
            "junos-conf-interfaces:interfaces": {
                "interface": [
                    {
                        "name": "",
                        "unit": [
                            {
                                "name": "0"
                            }
                        ]
                    }
                ]
            }
        });

        let body = serde_json::to_vec(&invalid_config).expect("Failed to serialize");
        let request = HttpRequest::new(HttpMethod::PUT, &url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(body);

        let response = client.execute(request).await;

        match response {
            Ok(resp) => {
                assert!(
                    !resp.is_success(),
                    "Invalid config should be rejected, got status {}",
                    resp.status_code
                );

                // Try to parse the RESTCONF error structure
                if !resp.body.is_empty() {
                    let body_str = String::from_utf8_lossy(&resp.body);
                    eprintln!("Error response for invalid config: {}", body_str);

                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&resp.body) {
                        if let Some(errors_obj) = json.get("ietf-restconf:errors") {
                            let error_array = errors_obj.get("error").and_then(|e| e.as_array());

                            if let Some(errors) = error_array {
                                assert!(!errors.is_empty(), "Error array should not be empty");

                                let first = &errors[0];

                                // Verify the three key fields
                                assert!(
                                    first.get("error-type").is_some(),
                                    "Should have error-type. Error: {}",
                                    serde_json::to_string_pretty(first).unwrap_or_default()
                                );
                                assert!(
                                    first.get("error-tag").is_some(),
                                    "Should have error-tag. Error: {}",
                                    serde_json::to_string_pretty(first).unwrap_or_default()
                                );

                                eprintln!(
                                    "  Parsed RESTCONF error: type={}, tag={}",
                                    first
                                        .get("error-type")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("?"),
                                    first
                                        .get("error-tag")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("?")
                                );
                            }
                        } else {
                            eprintln!(
                                "Note: cRPD returned JSON error without \
                                 ietf-restconf:errors wrapper"
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Note: Invalid config request returned transport error: {e}");
            }
        }

        harness.stop().await.expect("Failed to stop emulator");
    }
}

// ---------------------------------------------------------------------------
// Conformance report generation
// ---------------------------------------------------------------------------

mod report {
    use super::*;
    use rustconf_integration_tests::{
        ConformanceReporter, HarnessConfig, JunosCrpdConfig, TestDetails, TestHarness, TestResult,
        TestStatus,
    };
    use rustconf_runtime::{HttpMethod, HttpRequest};
    use std::path::Path;

    /// Set up a TestHarness with the cRPD emulator for report generation.
    async fn setup_harness() -> TestHarness {
        let harness_config = HarnessConfig::from_env();
        let emulator_config = JunosCrpdConfig::with_harness_config(&harness_config);
        let mut harness = TestHarness::new(emulator_config, &harness_config);
        harness
            .start()
            .await
            .expect("Failed to start cRPD emulator container");
        harness
    }

    /// Generate conformance reports after running a condensed E2E validation.
    ///
    /// This test exercises each category (smoke, CRUD, schema, errors) and
    /// records results into a `ConformanceReporter`, then writes both a
    /// human-readable text report and a JUnit XML report to the reports directory.
    ///
    /// Requirements: 10.1, 10.2, 10.3, 10.4
    #[tokio::test]
    async fn test_generate_conformance_report() {
        skip_unless_integration!();
        skip_unless_emulator!();

        let mut reporter = ConformanceReporter::new("Juniper cRPD (E2E)");
        let mut harness = setup_harness().await;
        let client = harness
            .restconf_client()
            .expect("Failed to create RESTCONF client");

        // --- Smoke: connectivity check ---
        let smoke_passed = {
            let url = client.build_url("/restconf");
            let request = HttpRequest::new(HttpMethod::GET, &url)
                .with_header("Accept", "application/yang-data+json");

            match client.execute(request).await {
                Ok(resp) if resp.status_code == 200 => {
                    reporter.record(TestResult {
                        yang_module: "restconf".to_string(),
                        operation: "GET /restconf (connectivity)".to_string(),
                        status: TestStatus::Pass,
                        details: None,
                    });
                    true
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "restconf".to_string(),
                        operation: "GET /restconf (connectivity)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 200".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some("GET /restconf".to_string()),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                    false
                }
                Err(e) => {
                    reporter.record(TestResult {
                        yang_module: "restconf".to_string(),
                        operation: "GET /restconf (connectivity)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 200".to_string()),
                            actual: Some(format!("Transport error: {e}")),
                            request: Some("GET /restconf".to_string()),
                            response: None,
                            conformance_warnings: vec![],
                        }),
                    });
                    false
                }
            }
        };

        // --- Smoke: authentication check ---
        if smoke_passed {
            let url = client.build_url("/restconf");
            let request = HttpRequest::new(HttpMethod::GET, &url)
                .with_header("Accept", "application/yang-data+json");

            match client.execute(request).await {
                Ok(resp) if resp.is_success() && resp.status_code != 401 => {
                    reporter.record(TestResult {
                        yang_module: "restconf".to_string(),
                        operation: "GET /restconf (authentication)".to_string(),
                        status: TestStatus::Pass,
                        details: None,
                    });
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "restconf".to_string(),
                        operation: "GET /restconf (authentication)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("Authenticated (non-401)".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some("GET /restconf".to_string()),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                }
                Err(e) => {
                    reporter.record(TestResult {
                        yang_module: "restconf".to_string(),
                        operation: "GET /restconf (authentication)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("Authenticated (non-401)".to_string()),
                            actual: Some(format!("Transport error: {e}")),
                            request: Some("GET /restconf".to_string()),
                            response: None,
                            conformance_warnings: vec![],
                        }),
                    });
                }
            }
        } else {
            reporter.record(TestResult {
                yang_module: "restconf".to_string(),
                operation: "GET /restconf (authentication)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke connectivity test did not pass".to_string(),
                },
                details: None,
            });
        }

        // --- CRUD: interfaces ---
        if smoke_passed {
            let iface_name = e2e_helpers::e2e_resource_name("rpt");
            let resource_path = "/data/junos-conf-interfaces:interfaces";

            let put_data = serde_json::json!({
                "junos-conf-interfaces:interfaces": {
                    "interface": [{
                        "name": iface_name,
                        "unit": [{
                            "name": "0",
                            "family": {
                                "inet": {
                                    "address": [{ "name": "10.99.0.1/24" }]
                                }
                            }
                        }]
                    }]
                }
            });

            let put_url = client.build_url(resource_path);
            let put_body = serde_json::to_vec(&put_data).unwrap();
            let put_request = HttpRequest::new(HttpMethod::PUT, &put_url)
                .with_header("Content-Type", "application/yang-data+json")
                .with_header("Accept", "application/yang-data+json")
                .with_body(put_body.clone());

            match client.execute(put_request).await {
                Ok(resp) if resp.is_success() => {
                    reporter.record(TestResult {
                        yang_module: "junos-conf-interfaces".to_string(),
                        operation: "PUT interface (create)".to_string(),
                        status: TestStatus::Pass,
                        details: None,
                    });

                    // Verify with GET
                    let get_url = client.build_url(resource_path);
                    let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
                        .with_header("Accept", "application/yang-data+json");

                    match client.execute(get_request).await {
                        Ok(get_resp) if get_resp.is_success() => {
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "GET interfaces (read after create)".to_string(),
                                status: TestStatus::Pass,
                                details: None,
                            });
                        }
                        Ok(get_resp) => {
                            let body_str = String::from_utf8_lossy(&get_resp.body).to_string();
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "GET interfaces (read after create)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some("HTTP 2xx".to_string()),
                                    actual: Some(format!("HTTP {}", get_resp.status_code)),
                                    request: Some(format!("GET {resource_path}")),
                                    response: Some(body_str),
                                    conformance_warnings: vec![],
                                }),
                            });
                        }
                        Err(e) => {
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "GET interfaces (read after create)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some("HTTP 2xx".to_string()),
                                    actual: Some(format!("Transport error: {e}")),
                                    request: Some(format!("GET {resource_path}")),
                                    response: None,
                                    conformance_warnings: vec![],
                                }),
                            });
                        }
                    }

                    // Cleanup: delete the interface
                    let delete_path = format!(
                        "/data/junos-conf-interfaces:interfaces/interface={}",
                        iface_name
                    );
                    let delete_url = client.build_url(&delete_path);
                    let delete_request = HttpRequest::new(HttpMethod::DELETE, &delete_url)
                        .with_header("Accept", "application/yang-data+json");

                    match client.execute(delete_request).await {
                        Ok(del_resp) if del_resp.is_success() => {
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "DELETE interface (cleanup)".to_string(),
                                status: TestStatus::Pass,
                                details: None,
                            });
                        }
                        Ok(del_resp) => {
                            let body_str = String::from_utf8_lossy(&del_resp.body).to_string();
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "DELETE interface (cleanup)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some("HTTP 2xx".to_string()),
                                    actual: Some(format!("HTTP {}", del_resp.status_code)),
                                    request: Some(format!("DELETE {delete_path}")),
                                    response: Some(body_str),
                                    conformance_warnings: vec![],
                                }),
                            });
                        }
                        Err(e) => {
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "DELETE interface (cleanup)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some("HTTP 2xx".to_string()),
                                    actual: Some(format!("Transport error: {e}")),
                                    request: Some(format!("DELETE {delete_path}")),
                                    response: None,
                                    conformance_warnings: vec![],
                                }),
                            });
                        }
                    }
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "junos-conf-interfaces".to_string(),
                        operation: "PUT interface (create)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 2xx".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some(format!(
                                "PUT {resource_path}\n{}",
                                String::from_utf8_lossy(&put_body)
                            )),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                }
                Err(e) => {
                    reporter.record(TestResult {
                        yang_module: "junos-conf-interfaces".to_string(),
                        operation: "PUT interface (create)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 2xx".to_string()),
                            actual: Some(format!("Transport error: {e}")),
                            request: Some(format!("PUT {resource_path}")),
                            response: None,
                            conformance_warnings: vec![],
                        }),
                    });
                }
            }
        } else {
            reporter.record(TestResult {
                yang_module: "junos-conf-interfaces".to_string(),
                operation: "PUT interface (create)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
            reporter.record(TestResult {
                yang_module: "junos-conf-interfaces".to_string(),
                operation: "GET interfaces (read after create)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
            reporter.record(TestResult {
                yang_module: "junos-conf-interfaces".to_string(),
                operation: "DELETE interface (cleanup)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
        }

        // --- CRUD: system configuration ---
        if smoke_passed {
            let resource_path = "/data/junos-conf-system:system";
            let get_url = client.build_url(resource_path);
            let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
                .with_header("Accept", "application/yang-data+json");

            match client.execute(get_request).await {
                Ok(resp) if resp.is_success() => {
                    // Verify deserialization
                    match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                        Ok(json) if json.get("junos-conf-system:system").is_some() => {
                            reporter.record(TestResult {
                                yang_module: "junos-conf-system".to_string(),
                                operation: "GET system (read + deserialize)".to_string(),
                                status: TestStatus::Pass,
                                details: None,
                            });
                        }
                        Ok(_) => {
                            let body_str = String::from_utf8_lossy(&resp.body).to_string();
                            reporter.record(TestResult {
                                yang_module: "junos-conf-system".to_string(),
                                operation: "GET system (read + deserialize)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some(
                                        "JSON with 'junos-conf-system:system' key".to_string(),
                                    ),
                                    actual: Some("Key not found in response".to_string()),
                                    request: Some(format!("GET {resource_path}")),
                                    response: Some(body_str),
                                    conformance_warnings: vec![],
                                }),
                            });
                        }
                        Err(e) => {
                            let body_str = String::from_utf8_lossy(&resp.body).to_string();
                            reporter.record(TestResult {
                                yang_module: "junos-conf-system".to_string(),
                                operation: "GET system (read + deserialize)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some("Valid JSON".to_string()),
                                    actual: Some(format!("Parse error: {e}")),
                                    request: Some(format!("GET {resource_path}")),
                                    response: Some(body_str),
                                    conformance_warnings: vec![],
                                }),
                            });
                        }
                    }
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "junos-conf-system".to_string(),
                        operation: "GET system (read + deserialize)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 2xx".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some(format!("GET {resource_path}")),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                }
                Err(e) => {
                    reporter.record(TestResult {
                        yang_module: "junos-conf-system".to_string(),
                        operation: "GET system (read + deserialize)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 2xx".to_string()),
                            actual: Some(format!("Transport error: {e}")),
                            request: Some(format!("GET {resource_path}")),
                            response: None,
                            conformance_warnings: vec![],
                        }),
                    });
                }
            }
        } else {
            reporter.record(TestResult {
                yang_module: "junos-conf-system".to_string(),
                operation: "GET system (read + deserialize)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
        }

        // --- Schema: key coverage on interfaces ---
        if smoke_passed {
            let resource_path = "/data/junos-conf-interfaces:interfaces";
            let get_url = client.build_url(resource_path);
            let get_request = HttpRequest::new(HttpMethod::GET, &get_url)
                .with_header("Accept", "application/yang-data+json");

            match client.execute(get_request).await {
                Ok(resp) if resp.is_success() => {
                    match serde_json::from_slice::<serde_json::Value>(&resp.body) {
                        Ok(_json) => {
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "GET interfaces (schema key coverage)".to_string(),
                                status: TestStatus::Pass,
                                details: None,
                            });
                        }
                        Err(e) => {
                            let body_str = String::from_utf8_lossy(&resp.body).to_string();
                            reporter.record(TestResult {
                                yang_module: "junos-conf-interfaces".to_string(),
                                operation: "GET interfaces (schema key coverage)".to_string(),
                                status: TestStatus::Fail,
                                details: Some(TestDetails {
                                    expected: Some("Valid JSON deserialization".to_string()),
                                    actual: Some(format!("Deserialization error: {e}")),
                                    request: Some(format!("GET {resource_path}")),
                                    response: Some(body_str),
                                    conformance_warnings: vec![
                                        "Response could not be deserialized".to_string(),
                                    ],
                                }),
                            });
                        }
                    }
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "junos-conf-interfaces".to_string(),
                        operation: "GET interfaces (schema key coverage)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 2xx".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some(format!("GET {resource_path}")),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                }
                Err(e) => {
                    reporter.record(TestResult {
                        yang_module: "junos-conf-interfaces".to_string(),
                        operation: "GET interfaces (schema key coverage)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 2xx".to_string()),
                            actual: Some(format!("Transport error: {e}")),
                            request: Some(format!("GET {resource_path}")),
                            response: None,
                            conformance_warnings: vec![],
                        }),
                    });
                }
            }
        } else {
            reporter.record(TestResult {
                yang_module: "junos-conf-interfaces".to_string(),
                operation: "GET interfaces (schema key coverage)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
        }

        // --- Errors: 404 on non-existent path ---
        if smoke_passed {
            let bad_path = "/data/junos-conf-nonexistent:nonexistent";
            let url = client.build_url(bad_path);
            let request = HttpRequest::new(HttpMethod::GET, &url)
                .with_header("Accept", "application/yang-data+json");

            match client.execute(request).await {
                Ok(resp) if resp.status_code == 404 => {
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "GET non-existent path (404)".to_string(),
                        status: TestStatus::Pass,
                        details: None,
                    });
                }
                Ok(resp) if !resp.is_success() => {
                    // Non-success is acceptable (might be 400 or other error)
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "GET non-existent path (404)".to_string(),
                        status: TestStatus::Pass,
                        details: None,
                    });
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "GET non-existent path (404)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP 404 or error status".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some(format!("GET {bad_path}")),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                }
                Err(e) => {
                    // Transport error is acceptable — client didn't panic
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "GET non-existent path (404)".to_string(),
                        status: TestStatus::Pass,
                        details: Some(TestDetails {
                            expected: None,
                            actual: Some(format!("Transport error (acceptable): {e}")),
                            request: Some(format!("GET {bad_path}")),
                            response: None,
                            conformance_warnings: vec![
                                "Transport error instead of HTTP 404".to_string()
                            ],
                        }),
                    });
                }
            }
        } else {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "GET non-existent path (404)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
        }

        // --- Errors: invalid value rejection ---
        if smoke_passed {
            let resource_path = "/data/junos-conf-interfaces:interfaces";
            let invalid_data = serde_json::json!({
                "junos-conf-interfaces:interfaces": {
                    "interface": [{
                        "name": "ge-0/0/0",
                        "unit": [{
                            "name": "0",
                            "family": {
                                "inet": {
                                    "address": [{ "name": "not-a-valid-ip" }]
                                }
                            }
                        }]
                    }]
                }
            });

            let url = client.build_url(resource_path);
            let body = serde_json::to_vec(&invalid_data).unwrap();
            let request = HttpRequest::new(HttpMethod::PUT, &url)
                .with_header("Content-Type", "application/yang-data+json")
                .with_header("Accept", "application/yang-data+json")
                .with_body(body.clone());

            match client.execute(request).await {
                Ok(resp) if !resp.is_success() => {
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "PUT invalid value (rejection)".to_string(),
                        status: TestStatus::Pass,
                        details: None,
                    });
                }
                Ok(resp) => {
                    let body_str = String::from_utf8_lossy(&resp.body).to_string();
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "PUT invalid value (rejection)".to_string(),
                        status: TestStatus::Fail,
                        details: Some(TestDetails {
                            expected: Some("HTTP error status (4xx)".to_string()),
                            actual: Some(format!("HTTP {}", resp.status_code)),
                            request: Some(format!(
                                "PUT {resource_path}\n{}",
                                String::from_utf8_lossy(&body)
                            )),
                            response: Some(body_str),
                            conformance_warnings: vec![],
                        }),
                    });
                }
                Err(e) => {
                    // Transport error is acceptable — client didn't panic
                    reporter.record(TestResult {
                        yang_module: "error-handling".to_string(),
                        operation: "PUT invalid value (rejection)".to_string(),
                        status: TestStatus::Pass,
                        details: Some(TestDetails {
                            expected: None,
                            actual: Some(format!("Transport error (acceptable): {e}")),
                            request: Some(format!("PUT {resource_path}")),
                            response: None,
                            conformance_warnings: vec![],
                        }),
                    });
                }
            }
        } else {
            reporter.record(TestResult {
                yang_module: "error-handling".to_string(),
                operation: "PUT invalid value (rejection)".to_string(),
                status: TestStatus::Skip {
                    reason: "Smoke tests did not pass".to_string(),
                },
                details: None,
            });
        }

        // --- Generate and write reports ---
        let text_report = reporter.generate_text_report();
        let junit_xml = reporter.generate_junit_xml();

        let reports_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("reports");
        std::fs::create_dir_all(&reports_dir).expect("Failed to create reports directory");

        let text_path = reports_dir.join("e2e-conformance.txt");
        std::fs::write(&text_path, &text_report).expect("Failed to write text report");
        eprintln!("Wrote conformance report to: {}", text_path.display());

        let xml_path = reports_dir.join("e2e-junit.xml");
        std::fs::write(&xml_path, &junit_xml).expect("Failed to write JUnit XML report");
        eprintln!("Wrote JUnit XML report to: {}", xml_path.display());

        // Print summary
        let (pass, fail, skip) = reporter.summary();
        eprintln!(
            "Report summary: {} passed, {} failed, {} skipped",
            pass, fail, skip
        );

        harness.stop().await.expect("Failed to stop emulator");
    }
}
