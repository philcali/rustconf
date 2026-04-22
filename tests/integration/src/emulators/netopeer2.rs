//! Netopeer2 emulator configuration.
//!
//! Provides [`NetopeerConfig`], an [`EmulatorConfig`] implementation for the
//! sysrepo/netopeer2 container. Netopeer2 is a NETCONF/RESTCONF server that
//! serves IETF standard YANG models, making it useful as a lightweight
//! conformance target for RESTCONF protocol validation.

use std::path::{Path, PathBuf};

use crate::config::HarnessConfig;
use crate::emulators::EmulatorConfig;

/// Default container image for Netopeer2.
const DEFAULT_IMAGE: &str = "sysrepo/netopeer2:latest";

/// Default RESTCONF port inside the Netopeer2 container.
const DEFAULT_PORT: u16 = 6443;

/// Default username for Netopeer2 authentication.
const DEFAULT_USERNAME: &str = "admin";

/// Default password for Netopeer2 authentication.
const DEFAULT_PASSWORD: &str = "admin";

/// Default RESTCONF base path.
const DEFAULT_BASE_PATH: &str = "/restconf";

/// Default health check path.
const DEFAULT_HEALTH_CHECK_PATH: &str = "/restconf";

/// Emulator configuration for sysrepo/Netopeer2.
///
/// Netopeer2 exposes a RESTCONF API over TLS on port 6443 by default.
/// It ships with IETF standard YANG models (ietf-interfaces, ietf-system, etc.),
/// making it the primary target for lightweight conformance testing that does not
/// require a vendor-specific emulator.
///
/// # Overrides
///
/// When constructed with [`NetopeerConfig::with_harness_config`], fields from the
/// [`HarnessConfig`] take precedence over defaults.
#[derive(Debug, Clone)]
pub struct NetopeerConfig {
    image: String,
    port: u16,
    username: String,
    password: String,
    base_path: String,
    tls: bool,
    yang_dir: PathBuf,
    health_path: String,
}

impl Default for NetopeerConfig {
    fn default() -> Self {
        Self {
            image: DEFAULT_IMAGE.to_string(),
            port: DEFAULT_PORT,
            username: DEFAULT_USERNAME.to_string(),
            password: DEFAULT_PASSWORD.to_string(),
            base_path: DEFAULT_BASE_PATH.to_string(),
            tls: true,
            yang_dir: PathBuf::from("yang/ietf"),
            health_path: DEFAULT_HEALTH_CHECK_PATH.to_string(),
        }
    }
}

impl NetopeerConfig {
    /// Create a new Netopeer2 config with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a Netopeer2 config, applying overrides from a [`HarnessConfig`].
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

impl EmulatorConfig for NetopeerConfig {
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
        "Netopeer2"
    }

    fn container_env(&self) -> Vec<(String, String)> {
        // Netopeer2 may need NETOPEER2_SETUP to run initial configuration.
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_netopeer_config() {
        let config = NetopeerConfig::new();
        assert_eq!(config.image_name(), "sysrepo/netopeer2:latest");
        assert_eq!(config.restconf_port(), 6443);
        assert_eq!(config.credentials(), ("admin", "admin"));
        assert_eq!(config.restconf_base_path(), "/restconf");
        assert!(config.uses_tls());
        assert_eq!(config.yang_model_dir(), Path::new("yang/ietf"));
        assert_eq!(config.health_check_path(), "/restconf");
        assert_eq!(config.vendor_name(), "Netopeer2");
        assert!(config.container_env().is_empty());
    }

    #[test]
    fn test_netopeer_config_with_harness_overrides() {
        let harness = HarnessConfig {
            container_image: Some("netopeer2:custom".to_string()),
            restconf_port: Some(9443),
            username: Some("root".to_string()),
            password: Some("topsecret".to_string()),
            ..HarnessConfig::default()
        };

        let config = NetopeerConfig::with_harness_config(&harness);
        assert_eq!(config.image_name(), "netopeer2:custom");
        assert_eq!(config.restconf_port(), 9443);
        assert_eq!(config.credentials(), ("root", "topsecret"));
        // Non-overridden fields keep defaults
        assert_eq!(config.restconf_base_path(), "/restconf");
        assert!(config.uses_tls());
        assert_eq!(config.yang_model_dir(), Path::new("yang/ietf"));
    }

    #[test]
    fn test_netopeer_config_partial_overrides() {
        let harness = HarnessConfig {
            restconf_port: Some(7443),
            ..HarnessConfig::default()
        };

        let config = NetopeerConfig::with_harness_config(&harness);
        assert_eq!(config.image_name(), "sysrepo/netopeer2:latest");
        assert_eq!(config.restconf_port(), 7443);
        assert_eq!(config.credentials(), ("admin", "admin"));
    }
}
