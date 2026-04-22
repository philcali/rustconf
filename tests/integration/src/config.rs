//! Runtime configuration for the integration test harness.
//!
//! Configuration can be loaded from environment variables via [`HarnessConfig::from_env`]
//! or from a TOML file with env var overrides via [`HarnessConfig::from_file`].

use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::error::HarnessError;

/// Default health check timeout in seconds.
const DEFAULT_HEALTH_TIMEOUT_SECS: u64 = 120;

/// Default per-test timeout in seconds.
const DEFAULT_TEST_TIMEOUT_SECS: u64 = 30;

/// Runtime configuration for the integration test harness.
///
/// Controls which emulator to use, how to connect to it, and timeout behavior.
/// All fields with `Option` values fall back to emulator-specific defaults when `None`.
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Emulator type identifier (e.g., "crpd", "netopeer2").
    pub emulator_type: String,
    /// Override container image name (e.g., "crpd:23.4R1.10").
    pub container_image: Option<String>,
    /// Override RESTCONF port inside the container.
    pub restconf_port: Option<u16>,
    /// Override username for authentication.
    pub username: Option<String>,
    /// Override password for authentication.
    pub password: Option<String>,
    /// How long to wait for the emulator health check to pass.
    pub health_timeout: Duration,
    /// Maximum duration for a single test operation.
    pub test_timeout: Duration,
    /// Whether to skip TLS certificate verification.
    pub skip_tls_verify: bool,
    /// Override the full RESTCONF base URL (scheme + host + port + path prefix).
    pub base_url: Option<String>,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            emulator_type: String::from("crpd"),
            container_image: None,
            restconf_port: None,
            username: None,
            password: None,
            health_timeout: Duration::from_secs(DEFAULT_HEALTH_TIMEOUT_SECS),
            test_timeout: Duration::from_secs(DEFAULT_TEST_TIMEOUT_SECS),
            skip_tls_verify: false,
            base_url: None,
        }
    }
}

impl HarnessConfig {
    /// Load configuration from environment variables.
    ///
    /// Supported variables:
    /// - `RUSTCONF_EMULATOR_TYPE` — emulator type (default: "crpd")
    /// - `RUSTCONF_CONTAINER_IMAGE` — container image override
    /// - `RUSTCONF_RESTCONF_PORT` — RESTCONF port override
    /// - `RUSTCONF_USERNAME` — authentication username
    /// - `RUSTCONF_PASSWORD` — authentication password
    /// - `RUSTCONF_HEALTH_TIMEOUT_SECS` — health check timeout in seconds
    /// - `RUSTCONF_TEST_TIMEOUT_SECS` — per-test timeout in seconds
    /// - `RUSTCONF_SKIP_TLS_VERIFY` — set to "1" or "true" to skip TLS verification
    /// - `RUSTCONF_BASE_URL` — full RESTCONF base URL override
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("RUSTCONF_EMULATOR_TYPE") {
            if !val.is_empty() {
                config.emulator_type = val;
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_CONTAINER_IMAGE") {
            if !val.is_empty() {
                config.container_image = Some(val);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_RESTCONF_PORT") {
            if let Ok(port) = val.parse::<u16>() {
                config.restconf_port = Some(port);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_USERNAME") {
            if !val.is_empty() {
                config.username = Some(val);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_PASSWORD") {
            if !val.is_empty() {
                config.password = Some(val);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_HEALTH_TIMEOUT_SECS") {
            if let Ok(secs) = val.parse::<u64>() {
                config.health_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_TEST_TIMEOUT_SECS") {
            if let Ok(secs) = val.parse::<u64>() {
                config.test_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_SKIP_TLS_VERIFY") {
            config.skip_tls_verify = val == "1" || val.eq_ignore_ascii_case("true");
        }

        if let Ok(val) = std::env::var("RUSTCONF_BASE_URL") {
            if !val.is_empty() {
                config.base_url = Some(val);
            }
        }

        config
    }

    /// Load configuration from a TOML file, then apply environment variable overrides.
    ///
    /// The TOML file should have the following structure:
    ///
    /// ```toml
    /// [emulator]
    /// type = "crpd"
    /// image = "crpd:23.4R1.10"
    /// restconf_port = 3000
    /// username = "root"
    /// password = "Juniper1"
    /// skip_tls_verify = true
    /// base_url = "https://localhost:3000/restconf"
    ///
    /// [timeouts]
    /// health_check_secs = 120
    /// test_timeout_secs = 30
    /// ```
    ///
    /// Environment variables always take precedence over file values.
    pub fn from_file(path: &Path) -> Result<Self, HarnessError> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            HarnessError::ConfigError(format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            ))
        })?;

        let file_config: TomlConfig = toml::from_str(&contents).map_err(|e| {
            HarnessError::ConfigError(format!(
                "Failed to parse config file {}: {}",
                path.display(),
                e
            ))
        })?;

        let mut config = Self::default();

        // Apply file values
        if let Some(ref emulator) = file_config.emulator {
            if let Some(ref t) = emulator.r#type {
                config.emulator_type = t.clone();
            }
            if let Some(ref img) = emulator.image {
                config.container_image = Some(img.clone());
            }
            if let Some(port) = emulator.restconf_port {
                config.restconf_port = Some(port);
            }
            if let Some(ref u) = emulator.username {
                config.username = Some(u.clone());
            }
            if let Some(ref p) = emulator.password {
                config.password = Some(p.clone());
            }
            if let Some(skip) = emulator.skip_tls_verify {
                config.skip_tls_verify = skip;
            }
            if let Some(ref url) = emulator.base_url {
                config.base_url = Some(url.clone());
            }
        }

        if let Some(ref timeouts) = file_config.timeouts {
            if let Some(secs) = timeouts.health_check_secs {
                config.health_timeout = Duration::from_secs(secs);
            }
            if let Some(secs) = timeouts.test_timeout_secs {
                config.test_timeout = Duration::from_secs(secs);
            }
        }

        // Apply env var overrides on top of file values
        config.apply_env_overrides();

        Ok(config)
    }

    /// Apply environment variable overrides to an already-populated config.
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("RUSTCONF_EMULATOR_TYPE") {
            if !val.is_empty() {
                self.emulator_type = val;
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_CONTAINER_IMAGE") {
            if !val.is_empty() {
                self.container_image = Some(val);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_RESTCONF_PORT") {
            if let Ok(port) = val.parse::<u16>() {
                self.restconf_port = Some(port);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_USERNAME") {
            if !val.is_empty() {
                self.username = Some(val);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_PASSWORD") {
            if !val.is_empty() {
                self.password = Some(val);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_HEALTH_TIMEOUT_SECS") {
            if let Ok(secs) = val.parse::<u64>() {
                self.health_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_TEST_TIMEOUT_SECS") {
            if let Ok(secs) = val.parse::<u64>() {
                self.test_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_SKIP_TLS_VERIFY") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                self.skip_tls_verify = true;
            }
        }

        if let Ok(val) = std::env::var("RUSTCONF_BASE_URL") {
            if !val.is_empty() {
                self.base_url = Some(val);
            }
        }
    }
}

/// Internal deserialization model for the TOML config file.
#[derive(Debug, Deserialize)]
struct TomlConfig {
    emulator: Option<TomlEmulator>,
    timeouts: Option<TomlTimeouts>,
}

#[derive(Debug, Deserialize)]
struct TomlEmulator {
    r#type: Option<String>,
    image: Option<String>,
    restconf_port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    skip_tls_verify: Option<bool>,
    base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlTimeouts {
    health_check_secs: Option<u64>,
    test_timeout_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = HarnessConfig::default();
        assert_eq!(config.emulator_type, "crpd");
        assert_eq!(config.health_timeout, Duration::from_secs(120));
        assert_eq!(config.test_timeout, Duration::from_secs(30));
        assert!(!config.skip_tls_verify);
        assert!(config.container_image.is_none());
        assert!(config.restconf_port.is_none());
        assert!(config.username.is_none());
        assert!(config.password.is_none());
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_from_file_basic() {
        let toml_content = r#"
[emulator]
type = "netopeer2"
image = "netopeer2:latest"
restconf_port = 6443
username = "admin"
password = "secret"
skip_tls_verify = true
base_url = "https://localhost:6443/restconf"

[timeouts]
health_check_secs = 60
test_timeout_secs = 15
"#;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.toml");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        // Clear any env vars that might interfere
        for var in &[
            "RUSTCONF_EMULATOR_TYPE",
            "RUSTCONF_CONTAINER_IMAGE",
            "RUSTCONF_RESTCONF_PORT",
            "RUSTCONF_USERNAME",
            "RUSTCONF_PASSWORD",
            "RUSTCONF_HEALTH_TIMEOUT_SECS",
            "RUSTCONF_TEST_TIMEOUT_SECS",
            "RUSTCONF_SKIP_TLS_VERIFY",
            "RUSTCONF_BASE_URL",
        ] {
            std::env::remove_var(var);
        }

        let config = HarnessConfig::from_file(&file_path).unwrap();
        assert_eq!(config.emulator_type, "netopeer2");
        assert_eq!(config.container_image.as_deref(), Some("netopeer2:latest"));
        assert_eq!(config.restconf_port, Some(6443));
        assert_eq!(config.username.as_deref(), Some("admin"));
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert!(config.skip_tls_verify);
        assert_eq!(
            config.base_url.as_deref(),
            Some("https://localhost:6443/restconf")
        );
        assert_eq!(config.health_timeout, Duration::from_secs(60));
        assert_eq!(config.test_timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_from_file_missing_file() {
        let result = HarnessConfig::from_file(Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HarnessError::ConfigError(_)));
    }

    #[test]
    fn test_from_file_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("bad.toml");
        std::fs::write(&file_path, "this is not [valid toml =").unwrap();

        let result = HarnessConfig::from_file(&file_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HarnessError::ConfigError(_)));
    }
}
