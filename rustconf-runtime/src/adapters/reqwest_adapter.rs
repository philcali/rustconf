//! Reqwest-based HTTP transport adapter.

use crate::{HttpMethod, HttpRequest, HttpResponse, HttpTransport, RpcError};
use async_trait::async_trait;

/// HTTP transport implementation using reqwest.
///
/// This adapter uses the reqwest library for HTTP communication,
/// which is a high-level, ergonomic HTTP client built on hyper.
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    /// Create a new reqwest transport with default settings.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Create a new reqwest transport with a custom client.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Convert HttpMethod to reqwest::Method
        let method = match request.method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::PATCH => reqwest::Method::PATCH,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
            HttpMethod::HEAD => reqwest::Method::HEAD,
        };

        // Build reqwest request
        let mut req_builder = self.client.request(method, &request.url);

        // Add headers
        for (name, value) in &request.headers {
            req_builder = req_builder.header(name, value);
        }

        // Add body if present
        if let Some(body) = request.body {
            req_builder = req_builder.body(body);
        }

        // Execute request
        let response = req_builder
            .send()
            .await
            .map_err(|e| RpcError::TransportError(e.to_string()))?;

        // Extract status code
        let status_code = response.status().as_u16();

        // Extract headers
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();

        // Extract body
        let body = response
            .bytes()
            .await
            .map_err(|e| RpcError::TransportError(e.to_string()))?
            .to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }
}
