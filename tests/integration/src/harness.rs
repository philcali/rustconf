//! Test harness for managing emulator container lifecycle and RESTCONF client access.
//!
//! [`TestHarness`] orchestrates the full lifecycle of a containerized RESTCONF emulator:
//! starting the container, waiting for the health check endpoint, providing a configured
//! RESTCONF client, and stopping/removing the container on teardown.

use std::time::Duration;

use base64::Engine as _;
use log::{debug, info, warn};
use testcontainers::{
    core::IntoContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
};

use rustconf_runtime::reqwest_adapter::ReqwestTransport;
use rustconf_runtime::{HttpRequest, RequestInterceptor, RestconfClient, RpcError};

use crate::config::HarnessConfig;
use crate::emulators::EmulatorConfig;
use crate::error::HarnessError;

/// Integration test harness that manages an emulator container and provides RESTCONF access.
///
/// # Lifecycle
///
/// 1. Create with [`TestHarness::new`] — stores configuration, does not start anything.
/// 2. Call [`TestHarness::start`] — starts the container, waits for health check.
/// 3. Use [`TestHarness::restconf_client`] to get a configured RESTCONF client.
/// 4. Call [`TestHarness::stop`] — stops and removes the container.
pub struct TestHarness {
    config: Box<dyn EmulatorConfig>,
    container: Option<ContainerAsync<GenericImage>>,
    client: Option<reqwest::Client>,
    base_url: String,
    health_timeout: Duration,
    skip_tls_verify: bool,
}

impl TestHarness {
    /// Create a new harness from an emulator config and harness config.
    ///
    /// Does not start the emulator — call [`start`](TestHarness::start) to begin.
    pub fn new(config: impl EmulatorConfig + 'static, harness_config: &HarnessConfig) -> Self {
        Self {
            config: Box::new(config),
            container: None,
            client: None,
            base_url: String::new(),
            health_timeout: harness_config.health_timeout,
            skip_tls_verify: harness_config.skip_tls_verify,
        }
    }

    /// Start the emulator container and wait for the health check to pass.
    ///
    /// This method:
    /// 1. Creates and starts a container from the emulator config image
    /// 2. Maps the RESTCONF port to a random host port
    /// 3. Computes the RESTCONF base URL from the mapped port
    /// 4. Builds a `reqwest::Client` for health checking
    /// 5. Polls the health check endpoint until it returns 200 or the timeout expires
    pub async fn start(&mut self) -> Result<(), HarnessError> {
        let image_name = self.config.image_name();
        let (name, tag) = parse_image_name(image_name);

        let port = self.config.restconf_port();
        let env_vars = self.config.container_env();

        info!("Starting container from image {}:{}", name, tag);

        // Build the image with exposed port
        let image = GenericImage::new(name, tag).with_exposed_port(port.tcp());

        // Apply environment variables via ImageExt (returns ContainerRequest)
        // Then start the container via AsyncRunner.
        let container = if env_vars.is_empty() {
            image.start().await.map_err(|e| {
                HarnessError::StartupFailed(format!("Failed to start container: {e}"))
            })?
        } else {
            let mut request = image.with_env_var(&env_vars[0].0, &env_vars[0].1);
            for (key, value) in env_vars.iter().skip(1) {
                request = request.with_env_var(key, value);
            }
            request.start().await.map_err(|e| {
                HarnessError::StartupFailed(format!("Failed to start container: {e}"))
            })?
        };

        // Get the mapped host port and host address
        let mapped_port = container
            .get_host_port_ipv4(port)
            .await
            .map_err(|e| HarnessError::StartupFailed(format!("Failed to get mapped port: {e}")))?;

        let host = container
            .get_host()
            .await
            .map_err(|e| HarnessError::StartupFailed(format!("Failed to get host: {e}")))?;

        // Compute the base URL
        let scheme = if self.config.uses_tls() {
            "https"
        } else {
            "http"
        };
        let base_path = self.config.restconf_base_path();
        self.base_url = format!("{scheme}://{host}:{mapped_port}{base_path}");

        info!("Container started, RESTCONF base URL: {}", self.base_url);

        // Build a reqwest client for health checks
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(self.skip_tls_verify)
            .build()
            .map_err(|e| {
                HarnessError::StartupFailed(format!("Failed to build HTTP client: {e}"))
            })?;

        // Store the container before the health check loop
        self.container = Some(container);
        self.client = Some(http_client.clone());

        // Health check polling loop
        let health_url = {
            let health_path = self.config.health_check_path();
            format!("{scheme}://{host}:{mapped_port}{health_path}")
        };

        info!(
            "Waiting for health check at {} (timeout: {:?})",
            health_url, self.health_timeout
        );

        let poll_interval = Duration::from_secs(2);
        let result = tokio::time::timeout(self.health_timeout, async {
            loop {
                match http_client.get(&health_url).send().await {
                    Ok(resp) if resp.status().as_u16() == 200 => {
                        info!("Health check passed");
                        return Ok(());
                    }
                    Ok(resp) => {
                        debug!(
                            "Health check returned status {}, retrying...",
                            resp.status()
                        );
                    }
                    Err(e) => {
                        debug!("Health check failed: {}, retrying...", e);
                    }
                }
                tokio::time::sleep(poll_interval).await;
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!("Health check timed out after {:?}", self.health_timeout);
                Err(HarnessError::HealthCheckTimeout(self.health_timeout))
            }
        }
    }

    /// Stop and remove the emulator container.
    ///
    /// After this call, [`is_running`](TestHarness::is_running) returns `false` and
    /// the container resources are cleaned up.
    pub async fn stop(&mut self) -> Result<(), HarnessError> {
        if let Some(container) = self.container.take() {
            info!("Stopping container...");
            container.stop().await.map_err(|e| {
                HarnessError::ContainerError(format!("Failed to stop container: {e}"))
            })?;

            container.rm().await.map_err(|e| {
                HarnessError::ContainerError(format!("Failed to remove container: {e}"))
            })?;

            info!("Container stopped and removed");
        }

        self.client = None;
        Ok(())
    }

    /// Returns `true` if the emulator container is currently running.
    pub fn is_running(&self) -> bool {
        self.container.is_some()
    }

    /// Returns the RESTCONF base URL (e.g., `https://127.0.0.1:32789/restconf`).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a configured [`RestconfClient`] pointing at the running emulator.
    ///
    /// The client uses Basic authentication with credentials from the emulator config
    /// and respects the `skip_tls_verify` setting.
    pub fn restconf_client(&self) -> Result<RestconfClient<ReqwestTransport>, HarnessError> {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(self.skip_tls_verify)
            .build()
            .map_err(|e| {
                HarnessError::ConfigError(format!("Failed to build RESTCONF HTTP client: {e}"))
            })?;

        let transport = ReqwestTransport::with_client(http_client);

        let (username, password) = self.config.credentials();
        let auth_interceptor = BasicAuthInterceptor::new(username, password);

        let client = RestconfClient::new(&self.base_url, transport)
            .map_err(|e| {
                HarnessError::ConfigError(format!("Failed to create RESTCONF client: {e}"))
            })?
            .with_interceptor(auth_interceptor);

        Ok(client)
    }
}

/// HTTP Basic authentication interceptor for RESTCONF requests.
///
/// Adds an `Authorization: Basic <encoded>` header to every outgoing request,
/// where `<encoded>` is the base64-encoded `username:password` string.
pub struct BasicAuthInterceptor {
    encoded_credentials: String,
}

impl BasicAuthInterceptor {
    /// Create a new interceptor with the given username and password.
    pub fn new(username: &str, password: &str) -> Self {
        let credentials = format!("{username}:{password}");
        let encoded_credentials =
            base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        Self {
            encoded_credentials,
        }
    }
}

impl RequestInterceptor for BasicAuthInterceptor {
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
        request.headers.push((
            "Authorization".to_string(),
            format!("Basic {}", self.encoded_credentials),
        ));
        Ok(())
    }
}

/// Parse an image name into (name, tag) parts.
///
/// If the image name contains a `:`, it is split into name and tag.
/// Otherwise, the tag defaults to `"latest"`.
fn parse_image_name(image: &str) -> (&str, &str) {
    match image.rsplit_once(':') {
        Some((name, tag)) if !name.is_empty() && !tag.is_empty() => (name, tag),
        _ => (image, "latest"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_name_with_tag() {
        assert_eq!(parse_image_name("crpd:23.4R1.10"), ("crpd", "23.4R1.10"));
    }

    #[test]
    fn test_parse_image_name_latest() {
        assert_eq!(parse_image_name("crpd:latest"), ("crpd", "latest"));
    }

    #[test]
    fn test_parse_image_name_no_tag() {
        assert_eq!(parse_image_name("crpd"), ("crpd", "latest"));
    }

    #[test]
    fn test_parse_image_name_with_registry() {
        assert_eq!(
            parse_image_name("sysrepo/netopeer2:latest"),
            ("sysrepo/netopeer2", "latest")
        );
    }

    #[test]
    fn test_basic_auth_interceptor() {
        let interceptor = BasicAuthInterceptor::new("root", "Juniper1");
        let mut request = HttpRequest::new(
            rustconf_runtime::HttpMethod::GET,
            "https://localhost:3000/restconf",
        );

        interceptor.intercept(&mut request).unwrap();

        assert_eq!(request.headers.len(), 1);
        let (name, value) = &request.headers[0];
        assert_eq!(name, "Authorization");

        // "root:Juniper1" base64-encoded
        let expected_encoded = base64::engine::general_purpose::STANDARD.encode(b"root:Juniper1");
        assert_eq!(value, &format!("Basic {expected_encoded}"));
    }

    #[test]
    fn test_harness_new_initial_state() {
        use crate::emulators::JunosCrpdConfig;

        let harness_config = HarnessConfig::default();
        let harness = TestHarness::new(JunosCrpdConfig::new(), &harness_config);

        assert!(!harness.is_running());
        assert!(harness.base_url().is_empty());
    }
}
