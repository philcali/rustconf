//! HTTP transport abstraction and RESTCONF client implementation.

use crate::error::RpcError;
use async_trait::async_trait;

/// HTTP methods supported by RESTCONF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD,
}

impl HttpMethod {
    /// Convert to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::HEAD => "HEAD",
        }
    }
}

/// HTTP request structure.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Create a new HTTP request.
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Add a header to the request.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the request body.
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

/// HTTP response structure.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new HTTP response.
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Check if the response status indicates success (2xx).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// Get a header value by name (case-insensitive).
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }
}

/// Trait for HTTP transport implementations.
///
/// This trait abstracts over different HTTP client libraries (reqwest, hyper, etc.)
/// allowing users to choose their preferred transport or implement custom transports.
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Execute an HTTP request and return the response.
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>;
}

/// Trait for request interceptors.
///
/// Interceptors can modify requests before they are sent, useful for adding
/// authentication, logging, or other cross-cutting concerns.
pub trait RequestInterceptor: Send + Sync {
    /// Intercept and potentially modify a request before it is sent.
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError>;
}

/// RESTCONF client that uses a pluggable HTTP transport.
///
/// This client provides the foundation for generated RESTCONF operations.
/// It handles base URL management, request interceptors, and delegates
/// actual HTTP execution to the provided transport implementation.
pub struct RestconfClient<T: HttpTransport> {
    base_url: String,
    transport: T,
    interceptors: Vec<Box<dyn RequestInterceptor>>,
}

impl<T: HttpTransport> RestconfClient<T> {
    /// Create a new RESTCONF client with the given base URL and transport.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the RESTCONF server (e.g., "https://device.example.com")
    /// * `transport` - The HTTP transport implementation to use
    ///
    /// # Errors
    ///
    /// Returns an error if the base URL is invalid.
    pub fn new(base_url: impl Into<String>, transport: T) -> Result<Self, RpcError> {
        let base_url = base_url.into();

        // Basic validation of base URL
        if base_url.is_empty() {
            return Err(RpcError::ConfigurationError(
                "Base URL cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            base_url,
            transport,
            interceptors: Vec::new(),
        })
    }

    /// Add a request interceptor to the client.
    ///
    /// Interceptors are called in the order they are added.
    pub fn with_interceptor(mut self, interceptor: impl RequestInterceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }

    /// Get the base URL of this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Execute an HTTP request through this client.
    ///
    /// This method applies all registered interceptors before delegating
    /// to the underlying transport.
    pub async fn execute(&self, mut request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Apply interceptors
        for interceptor in &self.interceptors {
            interceptor.intercept(&mut request)?;
        }

        // Execute through transport
        self.transport.execute(request).await
    }

    /// Build a full URL by combining the base URL with a path.
    pub fn build_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }
}
