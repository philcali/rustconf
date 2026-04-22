//! Test fixture management for applying and restoring emulator configuration state.
//!
//! [`FixtureManager`] handles the lifecycle of test fixtures: loading fixture definitions
//! from JSON files, applying them to the emulator via RESTCONF PUT, saving the original
//! state for rollback, and restoring that state on teardown.
//!
//! Fixtures follow RFC 7951 JSON encoding and are defined as JSON files with a
//! `resource_path` and `data` field.

use std::path::Path;

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use rustconf_runtime::reqwest_adapter::ReqwestTransport;
use rustconf_runtime::{HttpMethod, HttpRequest, RestconfClient};

use crate::error::HarnessError;

/// A fixture definition describing configuration data to apply to an emulator.
///
/// Fixture files are JSON documents with two fields:
/// - `resource_path`: the RESTCONF resource path (e.g., `/data/ietf-interfaces:interfaces`)
/// - `data`: the JSON payload to PUT at that path, following RFC 7951 encoding
///
/// # Example JSON
///
/// ```json
/// {
///   "resource_path": "/data/ietf-interfaces:interfaces",
///   "data": {
///     "ietf-interfaces:interfaces": {
///       "interface": [{ "name": "ge-0/0/0", "enabled": true }]
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixtureDefinition {
    /// RESTCONF resource path (e.g., `/data/ietf-interfaces:interfaces`).
    pub resource_path: String,
    /// JSON data to PUT at the resource path, following RFC 7951 encoding.
    pub data: serde_json::Value,
}

/// Tracks an applied fixture so it can be rolled back during teardown.
#[derive(Debug)]
struct AppliedFixture {
    /// RESTCONF path where the fixture was applied.
    resource_path: String,
    /// Original data at the resource path before the fixture was applied.
    /// `None` means the resource did not exist (teardown should DELETE).
    original_data: Option<serde_json::Value>,
}

/// Manages test fixture lifecycle: apply, track, and teardown.
///
/// `FixtureManager` wraps a [`RestconfClient`] and provides methods to:
/// - Load fixture definitions from JSON files
/// - Apply fixtures (saving original state for rollback)
/// - Tear down all applied fixtures (restoring original state)
///
/// Shared fixtures can be applied once and used across multiple tests before
/// calling [`teardown`](FixtureManager::teardown).
pub struct FixtureManager {
    client: RestconfClient<ReqwestTransport>,
    applied_fixtures: Vec<AppliedFixture>,
}

impl FixtureManager {
    /// Create a new fixture manager backed by the given RESTCONF client.
    pub fn new(client: RestconfClient<ReqwestTransport>) -> Self {
        Self {
            client,
            applied_fixtures: Vec::new(),
        }
    }

    /// Load a fixture definition from a JSON file.
    ///
    /// The file must contain a JSON object with `resource_path` (string) and `data` (object)
    /// fields. See [`FixtureDefinition`] for the expected format.
    pub fn load_fixture(path: &Path) -> Result<FixtureDefinition, HarnessError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            HarnessError::FixtureApplyFailed(format!(
                "Failed to read fixture file {}: {e}",
                path.display()
            ))
        })?;

        let fixture: FixtureDefinition = serde_json::from_str(&content).map_err(|e| {
            HarnessError::FixtureApplyFailed(format!(
                "Failed to parse fixture file {}: {e}",
                path.display()
            ))
        })?;

        Ok(fixture)
    }

    /// Apply a fixture to the emulator.
    ///
    /// This method:
    /// 1. GETs the current state at the fixture's resource path (saved for rollback)
    /// 2. PUTs the fixture data at the resource path
    ///
    /// If the GET returns a 404, the resource is treated as non-existent and teardown
    /// will DELETE it. If the PUT fails, a [`HarnessError::FixtureApplyFailed`] is returned.
    pub async fn apply(&mut self, fixture: &FixtureDefinition) -> Result<(), HarnessError> {
        info!("Applying fixture at {}", fixture.resource_path);

        // 1. GET current state for rollback
        let original_data = self.get_resource(&fixture.resource_path).await?;

        debug!(
            "Saved original state at {} (exists: {})",
            fixture.resource_path,
            original_data.is_some()
        );

        // 2. PUT the fixture data
        self.put_resource(&fixture.resource_path, &fixture.data)
            .await
            .map_err(|e| {
                HarnessError::FixtureApplyFailed(format!(
                    "Failed to PUT fixture at {}: {e}",
                    fixture.resource_path
                ))
            })?;

        info!("Fixture applied at {}", fixture.resource_path);

        // Track for teardown
        self.applied_fixtures.push(AppliedFixture {
            resource_path: fixture.resource_path.clone(),
            original_data,
        });

        Ok(())
    }

    /// Tear down all applied fixtures, restoring original state.
    ///
    /// Fixtures are restored in reverse order (LIFO). For each applied fixture:
    /// - If original data existed, it is restored via PUT
    /// - If the resource did not exist before, it is removed via DELETE
    ///
    /// Teardown errors are logged as warnings but do not stop the process.
    /// The first error encountered is returned after all fixtures have been processed.
    pub async fn teardown(&mut self) -> Result<(), HarnessError> {
        info!(
            "Tearing down {} applied fixture(s)",
            self.applied_fixtures.len()
        );

        // Drain in reverse order so the most recently applied fixture is restored first.
        let fixtures: Vec<AppliedFixture> = self.applied_fixtures.drain(..).rev().collect();
        let mut first_error: Option<HarnessError> = None;

        for fixture in fixtures {
            let result = match &fixture.original_data {
                Some(data) => {
                    debug!("Restoring original data at {}", fixture.resource_path);
                    self.put_resource(&fixture.resource_path, data)
                        .await
                        .map_err(|e| {
                            HarnessError::FixtureTeardownFailed(format!(
                                "Failed to restore {} : {e}",
                                fixture.resource_path
                            ))
                        })
                }
                None => {
                    debug!(
                        "Deleting resource at {} (did not exist before)",
                        fixture.resource_path
                    );
                    self.delete_resource(&fixture.resource_path)
                        .await
                        .map_err(|e| {
                            HarnessError::FixtureTeardownFailed(format!(
                                "Failed to delete {}: {e}",
                                fixture.resource_path
                            ))
                        })
                }
            };

            if let Err(e) = result {
                warn!("Fixture teardown error: {e}");
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }

        match first_error {
            Some(e) => Err(e),
            None => {
                info!("All fixtures torn down successfully");
                Ok(())
            }
        }
    }

    /// Returns the number of currently applied (tracked) fixtures.
    pub fn applied_count(&self) -> usize {
        self.applied_fixtures.len()
    }

    // ── Internal helpers ──────────────────────────────────────────────

    /// GET a RESTCONF resource, returning `None` if it does not exist (404).
    async fn get_resource(
        &self,
        resource_path: &str,
    ) -> Result<Option<serde_json::Value>, HarnessError> {
        let url = self.client.build_url(resource_path);
        let request = HttpRequest::new(HttpMethod::GET, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = self.client.execute(request).await.map_err(|e| {
            HarnessError::FixtureApplyFailed(format!("GET {resource_path} failed: {e}"))
        })?;

        if response.status_code == 404 {
            return Ok(None);
        }

        if !response.is_success() {
            return Err(HarnessError::FixtureApplyFailed(format!(
                "GET {resource_path} returned status {}",
                response.status_code
            )));
        }

        let value: serde_json::Value = serde_json::from_slice(&response.body).map_err(|e| {
            HarnessError::FixtureApplyFailed(format!(
                "Failed to parse GET response for {resource_path}: {e}"
            ))
        })?;

        Ok(Some(value))
    }

    /// PUT a JSON payload to a RESTCONF resource path.
    async fn put_resource(
        &self,
        resource_path: &str,
        data: &serde_json::Value,
    ) -> Result<(), HarnessError> {
        let url = self.client.build_url(resource_path);
        let body = serde_json::to_vec(data).map_err(|e| {
            HarnessError::FixtureApplyFailed(format!("Failed to serialize fixture data: {e}"))
        })?;

        let request = HttpRequest::new(HttpMethod::PUT, &url)
            .with_header("Content-Type", "application/yang-data+json")
            .with_header("Accept", "application/yang-data+json")
            .with_body(body);

        let response = self.client.execute(request).await.map_err(|e| {
            HarnessError::FixtureApplyFailed(format!("PUT {resource_path} failed: {e}"))
        })?;

        if !response.is_success() {
            let body_str = String::from_utf8_lossy(&response.body);
            return Err(HarnessError::FixtureApplyFailed(format!(
                "PUT {resource_path} returned status {}: {body_str}",
                response.status_code
            )));
        }

        Ok(())
    }

    /// DELETE a RESTCONF resource.
    async fn delete_resource(&self, resource_path: &str) -> Result<(), HarnessError> {
        let url = self.client.build_url(resource_path);
        let request = HttpRequest::new(HttpMethod::DELETE, &url)
            .with_header("Accept", "application/yang-data+json");

        let response = self.client.execute(request).await.map_err(|e| {
            HarnessError::FixtureTeardownFailed(format!("DELETE {resource_path} failed: {e}"))
        })?;

        // 404 on delete is acceptable — resource may already be gone.
        if !response.is_success() && response.status_code != 404 {
            let body_str = String::from_utf8_lossy(&response.body);
            return Err(HarnessError::FixtureTeardownFailed(format!(
                "DELETE {resource_path} returned status {}: {body_str}",
                response.status_code
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_fixture_valid() {
        let json = r#"{
            "resource_path": "/data/ietf-interfaces:interfaces",
            "data": {
                "ietf-interfaces:interfaces": {
                    "interface": [{"name": "ge-0/0/0", "enabled": true}]
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let fixture = FixtureManager::load_fixture(file.path()).unwrap();
        assert_eq!(fixture.resource_path, "/data/ietf-interfaces:interfaces");
        assert!(fixture.data.is_object());
    }

    #[test]
    fn test_load_fixture_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"not json").unwrap();

        let result = FixtureManager::load_fixture(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            HarnessError::FixtureApplyFailed(msg) => {
                assert!(msg.contains("Failed to parse fixture file"));
            }
            other => panic!("Expected FixtureApplyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn test_load_fixture_missing_file() {
        let result = FixtureManager::load_fixture(Path::new("/nonexistent/fixture.json"));
        assert!(result.is_err());
        match result.unwrap_err() {
            HarnessError::FixtureApplyFailed(msg) => {
                assert!(msg.contains("Failed to read fixture file"));
            }
            other => panic!("Expected FixtureApplyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn test_load_fixture_missing_fields() {
        let json = r#"{"resource_path": "/data/test"}"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let result = FixtureManager::load_fixture(file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            HarnessError::FixtureApplyFailed(msg) => {
                assert!(msg.contains("Failed to parse fixture file"));
            }
            other => panic!("Expected FixtureApplyFailed, got: {other:?}"),
        }
    }

    #[test]
    fn test_fixture_definition_roundtrip() {
        let fixture = FixtureDefinition {
            resource_path: "/data/test:config".to_string(),
            data: serde_json::json!({"test:config": {"enabled": true}}),
        };

        let json = serde_json::to_string(&fixture).unwrap();
        let loaded: FixtureDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(fixture, loaded);
    }
}
