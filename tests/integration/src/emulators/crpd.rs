//! Juniper cRPD emulator configuration.
//!
//! Provides [`JunosCrpdConfig`], an [`EmulatorConfig`] implementation with sensible defaults
//! for the Juniper cRPD container. All defaults can be overridden via [`HarnessConfig`].

use std::path::{Path, PathBuf};

use crate::config::HarnessConfig;
use crate::emulators::EmulatorConfig;

/// Default container image for Juniper cRPD.
const DEFAULT_IMAGE: &str = "crpd:latest";

/// Default RESTCONF port inside the cRPD container.
const DEFAULT_PORT: u16 = 3000;

/// Default username for cRPD authentication.
const DEFAULT_USERNAME: &str = "root";

/// Default password for cRPD authentication.
const DEFAULT_PASSWORD: &str = "Juniper1";

/// Default RESTCONF base path.
const DEFAULT_BASE_PATH: &str = "/restconf";

/// Default health check path.
const DEFAULT_HEALTH_CHECK_PATH: &str = "/restconf";

/// Emulator configuration for Juniper cRPD.
///
/// cRPD (containerized routing protocol daemon) exposes a RESTCONF API over TLS
/// on port 3000 by default. Authentication uses HTTP Basic with `root` / `Juniper1`.
///
/// # Overrides
///
/// When constructed with [`JunosCrpdConfig::with_harness_config`], fields from the
/// [`HarnessConfig`] take precedence over defaults. This allows CI or developer
/// environments to point at a custom image tag, alternate port, or different credentials.
#[derive(Debug, Clone)]
pub struct JunosCrpdConfig {
    image: String,
    port: u16,
    username: String,
    password: String,
    base_path: String,
    tls: bool,
    yang_dir: PathBuf,
    health_path: String,
}

impl Default for JunosCrpdConfig {
    fn default() -> Self {
        Self {
            image: DEFAULT_IMAGE.to_string(),
            port: DEFAULT_PORT,
            username: DEFAULT_USERNAME.to_string(),
            password: DEFAULT_PASSWORD.to_string(),
            base_path: DEFAULT_BASE_PATH.to_string(),
            tls: true,
            yang_dir: PathBuf::from("yang/juniper"),
            health_path: DEFAULT_HEALTH_CHECK_PATH.to_string(),
        }
    }
}

impl JunosCrpdConfig {
    /// Create a new cRPD config with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a cRPD config, applying overrides from a [`HarnessConfig`].
    ///
    /// Only `Some` fields in the harness config override the defaults.
    pub fn with_harness_config(harness: &HarnessConfig) -> Self {
        let mut config = Self::default();

        if let Some(ref image) = harness.container_image {
            config.image = image.clone();
        }
        if let Some(port) = harness.restconf_port {
            config.port = port;
        }
        if let Some(ref username) = harness.username {
            config.username = username.clone();
        }
        if let Some(ref password) = harness.password {
            config.password = password.clone();
        }

        config
    }
}

impl EmulatorConfig for JunosCrpdConfig {
    fn image_name(&self) -> &str {
        &self.image
    }

    fn restconf_port(&self) -> u16 {
        self.port
    }

    fn credentials(&self) -> (&str, &str) {
        (&self.username, &self.password)
    }

    fn restconf_base_path(&self) -> &str {
        &self.base_path
    }

    fn uses_tls(&self) -> bool {
        self.tls
    }

    fn yang_model_dir(&self) -> &Path {
        &self.yang_dir
    }

    fn health_check_path(&self) -> &str {
        &self.health_path
    }

    fn vendor_name(&self) -> &str {
        "Juniper cRPD"
    }

    fn container_env(&self) -> Vec<(String, String)> {
        // cRPD doesn't require extra env vars by default.
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_crpd_config() {
        let config = JunosCrpdConfig::new();
        assert_eq!(config.image_name(), "crpd:latest");
        assert_eq!(config.restconf_port(), 3000);
        assert_eq!(config.credentials(), ("root", "Juniper1"));
        assert_eq!(config.restconf_base_path(), "/restconf");
        assert!(config.uses_tls());
        assert_eq!(config.yang_model_dir(), Path::new("yang/juniper"));
        assert_eq!(config.health_check_path(), "/restconf");
        assert_eq!(config.vendor_name(), "Juniper cRPD");
        assert!(config.container_env().is_empty());
    }

    #[test]
    fn test_crpd_config_with_harness_overrides() {
        let harness = HarnessConfig {
            container_image: Some("crpd:23.4R1.10".to_string()),
            restconf_port: Some(8443),
            username: Some("admin".to_string()),
            password: Some("secret".to_string()),
            ..HarnessConfig::default()
        };

        let config = JunosCrpdConfig::with_harness_config(&harness);
        assert_eq!(config.image_name(), "crpd:23.4R1.10");
        assert_eq!(config.restconf_port(), 8443);
        assert_eq!(config.credentials(), ("admin", "secret"));
        // Non-overridden fields keep defaults
        assert_eq!(config.restconf_base_path(), "/restconf");
        assert!(config.uses_tls());
    }

    #[test]
    fn test_crpd_config_partial_overrides() {
        let harness = HarnessConfig {
            container_image: Some("crpd:custom".to_string()),
            ..HarnessConfig::default()
        };

        let config = JunosCrpdConfig::with_harness_config(&harness);
        assert_eq!(config.image_name(), "crpd:custom");
        // Everything else stays default
        assert_eq!(config.restconf_port(), 3000);
        assert_eq!(config.credentials(), ("root", "Juniper1"));
    }
}
