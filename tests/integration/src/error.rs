//! Unified error type for the integration test harness.

use std::time::Duration;

use rustconf_runtime::RpcError;

/// Unified error type covering all failure modes in the integration test harness.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// Emulator container failed to start.
    #[error("Emulator startup failed: {0}")]
    StartupFailed(String),

    /// Health check did not pass within the configured timeout.
    #[error("Health check timed out after {0:?}")]
    HealthCheckTimeout(Duration),

    /// Emulator container crashed during a test session.
    #[error("Emulator crashed during test: {0}")]
    EmulatorCrashed(String),

    /// Failed to apply a test fixture to the emulator.
    #[error("Fixture apply failed: {0}")]
    FixtureApplyFailed(String),

    /// Failed to tear down (restore) a test fixture on the emulator.
    #[error("Fixture teardown failed: {0}")]
    FixtureTeardownFailed(String),

    /// Code generation from YANG models failed.
    #[error("Code generation failed: {0}")]
    CodegenFailed(String),

    /// RESTCONF protocol-level error from the emulator or generated client.
    #[error("RESTCONF error: {0}")]
    RestconfError(String),

    /// Container orchestration error (start, stop, networking).
    #[error("Container error: {0}")]
    ContainerError(String),

    /// Configuration loading or validation error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// A test operation exceeded the configured per-test timeout.
    #[error("Test timeout after {0:?}")]
    TestTimeout(Duration),

    /// Underlying I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<RpcError> for HarnessError {
    fn from(err: RpcError) -> Self {
        HarnessError::RestconfError(err.to_string())
    }
}
