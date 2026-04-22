//! Emulator configuration abstractions and vendor-specific implementations.
//!
//! The [`EmulatorConfig`] trait abstracts vendor-specific details (container image, ports,
//! credentials, YANG model paths, RESTCONF endpoint configuration) so that the test harness
//! can work with any RESTCONF-capable emulator without hard-coding vendor specifics.
//!
//! Concrete implementations are provided for:
//! - [`JunosCrpdConfig`] — Juniper cRPD container
//! - [`NetopeerConfig`] — sysrepo/Netopeer2 container (IETF reference)

pub mod crpd;
pub mod netopeer2;

pub use crpd::JunosCrpdConfig;
pub use netopeer2::NetopeerConfig;

use std::path::Path;

/// Configuration for a RESTCONF emulator.
///
/// Each supported emulator implements this trait to provide vendor-specific defaults
/// for container image, networking, authentication, and YANG model locations.
/// Implementations may accept an optional [`HarnessConfig`](crate::HarnessConfig) to
/// allow overriding defaults at runtime.
pub trait EmulatorConfig: Send + Sync {
    /// Container image name (e.g., `"crpd:latest"`).
    fn image_name(&self) -> &str;

    /// RESTCONF port inside the container.
    fn restconf_port(&self) -> u16;

    /// Default credentials as `(username, password)`.
    fn credentials(&self) -> (&str, &str);

    /// Path prefix for RESTCONF endpoints (e.g., `"/restconf"`).
    fn restconf_base_path(&self) -> &str;

    /// Whether the emulator uses TLS for RESTCONF.
    fn uses_tls(&self) -> bool;

    /// Directory containing vendor YANG models for this emulator.
    fn yang_model_dir(&self) -> &Path;

    /// Health check URL path (e.g., `"/.well-known/host-meta"` or `"/restconf"`).
    fn health_check_path(&self) -> &str;

    /// Vendor identifier for conformance reports.
    fn vendor_name(&self) -> &str;

    /// Environment variables to set on the container.
    fn container_env(&self) -> Vec<(String, String)>;
}
