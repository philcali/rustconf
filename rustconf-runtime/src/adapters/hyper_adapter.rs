//! Hyper-based HTTP transport adapter.

use crate::{HttpMethod, HttpRequest, HttpResponse, HttpTransport, RpcError};
use async_trait::async_trait;
use hyper::client::{Client, HttpConnector};
use hyper::{Body, Request, Uri};
use hyper_tls::HttpsConnector;

/// HTTP transport implementation using hyper.
///
/// This adapter uses the hyper library for HTTP communication,
/// which is a low-level, fast HTTP implementation.
#[derive(Clone)]
pub struct HyperTransport {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl HyperTransport {
    /// Create a new hyper transport with default settings.
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);
        Self { client }
    }

    /// Create a new hyper transport with a custom client.
    pub fn with_client(client: Client<HttpsConnector<HttpConnector>>) -> Self {
        Self { client }
    }
}

impl Default for HyperTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpTransport for HyperTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Parse URI
        let uri: Uri = request
            .url
            .parse()
            .map_err(|e| RpcError::TransportError(format!("Invalid URL: {}", e)))?;

        // Convert HttpMethod to hyper::Method
        let method = match request.method {
            HttpMethod::GET => hyper::Method::GET,
            HttpMethod::POST => hyper::Method::POST,
            HttpMethod::PUT => hyper::Method::PUT,
            HttpMethod::PATCH => hyper::Method::PATCH,
            HttpMethod::DELETE => hyper::Method::DELETE,
            HttpMethod::OPTIONS => hyper::Method::OPTIONS,
            HttpMethod::HEAD => hyper::Method::HEAD,
        };

        // Build hyper request
        let mut req_builder = Request::builder().method(method).uri(uri);

        // Add headers
        for (name, value) in &request.headers {
            req_builder = req_builder.header(name, value);
        }

        // Add body
        let body = if let Some(body_bytes) = request.body {
            Body::from(body_bytes)
        } else {
            Body::empty()
        };

        let hyper_request = req_builder
            .body(body)
            .map_err(|e| RpcError::TransportError(format!("Failed to build request: {}", e)))?;

        // Execute request
        let response = self
            .client
            .request(hyper_request)
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
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .map_err(|e| RpcError::TransportError(e.to_string()))?
            .to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body: body_bytes,
        })
    }
}
