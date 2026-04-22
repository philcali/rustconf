//! Shared test helpers for integration tests.
//!
//! Provides CI gating functions that allow integration tests to skip gracefully
//! when the required environment is not available, without failing the build.

/// Environment variable that gates integration test execution.
const INTEGRATION_TEST_ENV: &str = "RUSTCONF_INTEGRATION_TEST";

/// Skip the calling test unless `RUSTCONF_INTEGRATION_TEST=1` is set.
///
/// When the environment variable is absent or set to any value other than `"1"`,
/// the test returns early (skipped). This keeps `cargo test` green in environments
/// that don't have an emulator container image available.
///
/// # Usage
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_get_interfaces() {
///     skip_unless_integration!();
///     // ... test body that requires a live emulator
/// }
/// ```
#[macro_export]
macro_rules! skip_unless_integration {
    () => {
        if !common::is_integration_enabled() {
            eprintln!(
                "Skipping integration test: set {}=1 to enable",
                "RUSTCONF_INTEGRATION_TEST"
            );
            return;
        }
    };
}

/// Check whether integration tests are enabled via the environment variable.
///
/// Returns `true` when `RUSTCONF_INTEGRATION_TEST` is set to `"1"`.
pub fn is_integration_enabled() -> bool {
    std::env::var(INTEGRATION_TEST_ENV)
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Skip the calling test unless a container runtime is available and the
/// emulator container image can be reached.
///
/// This performs a lightweight check by running `docker info` (or `podman info`)
/// to verify the container runtime is responsive. It does **not** pull or start
/// the emulator image — that's the harness's job.
///
/// # Usage
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_requires_emulator() {
///     skip_unless_integration!();
///     skip_unless_emulator!();
///     // ... test body that requires a running emulator
/// }
/// ```
#[macro_export]
macro_rules! skip_unless_emulator {
    () => {
        if !common::is_emulator_available() {
            eprintln!("Skipping test: no container runtime available (docker/podman)");
            return;
        }
    };
}

/// Check whether a container runtime (Docker or Podman) is available.
///
/// Tries `docker info` first, then falls back to `podman info`. Returns `true`
/// if either command exits successfully, indicating the runtime is responsive
/// and the current user has permission to manage containers.
pub fn is_emulator_available() -> bool {
    // Try docker first
    if let Ok(output) = std::process::Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        if output.success() {
            return true;
        }
    }

    // Fall back to podman
    if let Ok(output) = std::process::Command::new("podman")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        if output.success() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mutex to serialize tests that mutate the shared process environment.
    /// Without this, parallel test threads race on `RUSTCONF_INTEGRATION_TEST`.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_is_integration_enabled_when_unset() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let original = std::env::var(INTEGRATION_TEST_ENV).ok();
        std::env::remove_var(INTEGRATION_TEST_ENV);

        assert!(!is_integration_enabled());

        // Restore
        if let Some(val) = original {
            std::env::set_var(INTEGRATION_TEST_ENV, val);
        }
    }

    #[test]
    fn test_is_integration_enabled_when_set_to_1() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let original = std::env::var(INTEGRATION_TEST_ENV).ok();
        std::env::set_var(INTEGRATION_TEST_ENV, "1");

        assert!(is_integration_enabled());

        // Restore
        match original {
            Some(val) => std::env::set_var(INTEGRATION_TEST_ENV, val),
            None => std::env::remove_var(INTEGRATION_TEST_ENV),
        }
    }

    #[test]
    fn test_is_integration_enabled_when_set_to_other() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let original = std::env::var(INTEGRATION_TEST_ENV).ok();
        std::env::set_var(INTEGRATION_TEST_ENV, "yes");

        assert!(!is_integration_enabled());

        // Restore
        match original {
            Some(val) => std::env::set_var(INTEGRATION_TEST_ENV, val),
            None => std::env::remove_var(INTEGRATION_TEST_ENV),
        }
    }

    #[test]
    fn test_is_integration_enabled_when_empty() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let original = std::env::var(INTEGRATION_TEST_ENV).ok();
        std::env::set_var(INTEGRATION_TEST_ENV, "");

        assert!(!is_integration_enabled());

        // Restore
        match original {
            Some(val) => std::env::set_var(INTEGRATION_TEST_ENV, val),
            None => std::env::remove_var(INTEGRATION_TEST_ENV),
        }
    }
}
