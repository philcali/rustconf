//! HTTP transport abstraction and RESTCONF client implementation.

use crate::error::{RpcError, ServerError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// HTTP methods supported by RESTCONF.
///
/// Represents the standard HTTP methods used in RESTCONF operations.
/// All methods are serializable for potential logging or debugging purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    /// HTTP GET method - retrieve a resource
    GET,
    /// HTTP POST method - create a resource or invoke an operation
    POST,
    /// HTTP PUT method - replace a resource
    PUT,
    /// HTTP PATCH method - partially update a resource
    PATCH,
    /// HTTP DELETE method - remove a resource
    DELETE,
    /// HTTP OPTIONS method - query supported methods
    OPTIONS,
    /// HTTP HEAD method - retrieve headers only
    HEAD,
}

impl HttpMethod {
    /// Convert to string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::HttpMethod;
    ///
    /// assert_eq!(HttpMethod::GET.as_str(), "GET");
    /// assert_eq!(HttpMethod::POST.as_str(), "POST");
    /// ```
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
///
/// Represents an HTTP request with method, URL, headers, and optional body.
/// This structure is used to communicate with RESTCONF servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// The HTTP method for this request
    pub method: HttpMethod,
    /// The full URL for this request
    pub url: String,
    /// HTTP headers as name-value pairs
    pub headers: Vec<(String, String)>,
    /// Optional request body as raw bytes
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Create a new HTTP request.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::new(HttpMethod::GET, "https://example.com/api");
    /// assert_eq!(request.url, "https://example.com/api");
    /// assert_eq!(request.method, HttpMethod::GET);
    /// ```
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Add a header to the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::new(HttpMethod::GET, "https://example.com/api")
    ///     .with_header("Content-Type", "application/json");
    /// ```
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the request body.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{HttpRequest, HttpMethod};
    ///
    /// let body = b"{\"key\": \"value\"}".to_vec();
    /// let request = HttpRequest::new(HttpMethod::POST, "https://example.com/api")
    ///     .with_body(body);
    /// ```
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

/// HTTP response structure.
///
/// Represents an HTTP response with status code, headers, and body.
/// This structure is returned by transport implementations after executing requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code (e.g., 200, 404, 500)
    pub status_code: u16,
    /// HTTP headers as name-value pairs
    pub headers: Vec<(String, String)>,
    /// Response body as raw bytes
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new HTTP response.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::HttpResponse;
    ///
    /// let response = HttpResponse::new(200);
    /// assert_eq!(response.status_code, 200);
    /// assert!(response.is_success());
    /// ```
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Check if the response status indicates success (2xx).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::HttpResponse;
    ///
    /// assert!(HttpResponse::new(200).is_success());
    /// assert!(HttpResponse::new(201).is_success());
    /// assert!(!HttpResponse::new(404).is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// Get a header value by name (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::HttpResponse;
    ///
    /// let mut response = HttpResponse::new(200);
    /// response.headers.push(("Content-Type".to_string(), "application/json".to_string()));
    /// assert_eq!(response.get_header("content-type"), Some("application/json"));
    /// ```
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
///
/// # Examples
///
/// Implementing a custom transport:
///
/// ```no_run
/// use rustconf_runtime::{HttpTransport, HttpRequest, HttpResponse, RpcError};
/// use async_trait::async_trait;
///
/// struct MyCustomTransport {
///     // Your transport state
/// }
///
/// #[async_trait]
/// impl HttpTransport for MyCustomTransport {
///     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
///         // Your custom HTTP implementation
///         todo!()
///     }
/// }
/// ```
///
/// Using a transport with RestconfClient:
///
/// ```no_run
/// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, RpcError};
/// # use async_trait::async_trait;
/// # struct MyTransport;
/// # #[async_trait]
/// # impl HttpTransport for MyTransport {
/// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
/// #         todo!()
/// #     }
/// # }
/// # async fn example() -> Result<(), RpcError> {
/// let transport = MyTransport;
/// let client = RestconfClient::new("https://device.example.com", transport)?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Execute an HTTP request and return the response.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request to execute
    ///
    /// # Returns
    ///
    /// Returns the HTTP response on success, or an RpcError on failure.
    ///
    /// # Errors
    ///
    /// This method should return:
    /// - `RpcError::TransportError` for network or connection failures
    /// - `RpcError::HttpError` for HTTP-level errors (4xx, 5xx status codes)
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>;
}

/// Trait for request interceptors.
///
/// Interceptors can modify requests before they are sent, useful for adding
/// authentication, logging, or other cross-cutting concerns.
///
/// # Examples
///
/// Adding authentication headers:
///
/// ```
/// use rustconf_runtime::{RequestInterceptor, HttpRequest, RpcError};
///
/// struct AuthInterceptor {
///     token: String,
/// }
///
/// impl RequestInterceptor for AuthInterceptor {
///     fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
///         request.headers.push((
///             "Authorization".to_string(),
///             format!("Bearer {}", self.token)
///         ));
///         Ok(())
///     }
/// }
/// ```
///
/// Using with RestconfClient:
///
/// ```no_run
/// # use rustconf_runtime::{RestconfClient, RequestInterceptor, HttpRequest, HttpTransport, HttpResponse, RpcError};
/// # use async_trait::async_trait;
/// # struct MyTransport;
/// # #[async_trait]
/// # impl HttpTransport for MyTransport {
/// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
/// #         todo!()
/// #     }
/// # }
/// # struct AuthInterceptor { token: String }
/// # impl RequestInterceptor for AuthInterceptor {
/// #     fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
/// #         Ok(())
/// #     }
/// # }
/// # fn example() -> Result<(), RpcError> {
/// let transport = MyTransport;
/// let auth = AuthInterceptor { token: "secret".to_string() };
/// let client = RestconfClient::new("https://device.example.com", transport)?
///     .with_interceptor(auth);
/// # Ok(())
/// # }
/// ```
pub trait RequestInterceptor: Send + Sync {
    /// Intercept and potentially modify a request before it is sent.
    ///
    /// # Arguments
    ///
    /// * `request` - Mutable reference to the request to be modified
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if interception succeeds, or an `RpcError` if it fails.
    ///
    /// # Errors
    ///
    /// Return an error if the request cannot be properly intercepted
    /// (e.g., authentication token is expired or invalid).
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError>;
}

/// RESTCONF client that uses a pluggable HTTP transport.
///
/// This client provides the foundation for generated RESTCONF operations.
/// It handles base URL management, request interceptors, and delegates
/// actual HTTP execution to the provided transport implementation.
///
/// # Examples
///
/// Basic usage with a custom transport:
///
/// ```no_run
/// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, RpcError};
/// # use async_trait::async_trait;
/// # struct MyTransport;
/// # #[async_trait]
/// # impl HttpTransport for MyTransport {
/// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
/// #         todo!()
/// #     }
/// # }
/// # fn example() -> Result<(), RpcError> {
/// let transport = MyTransport;
/// let client = RestconfClient::new("https://device.example.com", transport)?;
/// # Ok(())
/// # }
/// ```
///
/// Using with interceptors for authentication:
///
/// ```no_run
/// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, RpcError, RequestInterceptor};
/// # use async_trait::async_trait;
/// # struct MyTransport;
/// # #[async_trait]
/// # impl HttpTransport for MyTransport {
/// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
/// #         todo!()
/// #     }
/// # }
/// # struct AuthInterceptor { token: String }
/// # impl RequestInterceptor for AuthInterceptor {
/// #     fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
/// #         Ok(())
/// #     }
/// # }
/// # fn example() -> Result<(), RpcError> {
/// let transport = MyTransport;
/// let auth = AuthInterceptor { token: "secret".to_string() };
/// let client = RestconfClient::new("https://device.example.com", transport)?
///     .with_interceptor(auth);
/// # Ok(())
/// # }
/// ```
///
/// Making requests:
///
/// ```no_run
/// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, HttpMethod, RpcError};
/// # use async_trait::async_trait;
/// # struct MyTransport;
/// # #[async_trait]
/// # impl HttpTransport for MyTransport {
/// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
/// #         todo!()
/// #     }
/// # }
/// # async fn example() -> Result<(), RpcError> {
/// # let transport = MyTransport;
/// let client = RestconfClient::new("https://device.example.com", transport)?;
/// let url = client.build_url("/restconf/data/device");
/// let request = HttpRequest::new(HttpMethod::GET, url);
/// let response = client.execute(request).await?;
/// # Ok(())
/// # }
/// ```
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
    /// Interceptors are called in the order they are added, allowing you to
    /// chain multiple interceptors for different concerns (auth, logging, etc.).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, RpcError, RequestInterceptor};
    /// # use async_trait::async_trait;
    /// # struct MyTransport;
    /// # #[async_trait]
    /// # impl HttpTransport for MyTransport {
    /// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
    /// #         todo!()
    /// #     }
    /// # }
    /// # struct AuthInterceptor;
    /// # impl RequestInterceptor for AuthInterceptor {
    /// #     fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// # struct LoggingInterceptor;
    /// # impl RequestInterceptor for LoggingInterceptor {
    /// #     fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// # fn example() -> Result<(), RpcError> {
    /// # let transport = MyTransport;
    /// let client = RestconfClient::new("https://device.example.com", transport)?
    ///     .with_interceptor(AuthInterceptor)
    ///     .with_interceptor(LoggingInterceptor);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_interceptor(mut self, interceptor: impl RequestInterceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }

    /// Get the base URL of this client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, RpcError};
    /// # use async_trait::async_trait;
    /// # struct MyTransport;
    /// # #[async_trait]
    /// # impl HttpTransport for MyTransport {
    /// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
    /// #         todo!()
    /// #     }
    /// # }
    /// # fn example() -> Result<(), RpcError> {
    /// # let transport = MyTransport;
    /// let client = RestconfClient::new("https://device.example.com", transport)?;
    /// assert_eq!(client.base_url(), "https://device.example.com");
    /// # Ok(())
    /// # }
    /// ```
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Execute an HTTP request through this client.
    ///
    /// This method applies all registered interceptors before delegating
    /// to the underlying transport.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request to execute
    ///
    /// # Returns
    ///
    /// Returns the HTTP response on success, or an RpcError on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any interceptor fails
    /// - The transport fails to execute the request
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, HttpMethod, RpcError};
    /// # use async_trait::async_trait;
    /// # struct MyTransport;
    /// # #[async_trait]
    /// # impl HttpTransport for MyTransport {
    /// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
    /// #         Ok(HttpResponse::new(200))
    /// #     }
    /// # }
    /// # async fn example() -> Result<(), RpcError> {
    /// # let transport = MyTransport;
    /// let client = RestconfClient::new("https://device.example.com", transport)?;
    /// let request = HttpRequest::new(HttpMethod::GET, "https://device.example.com/api");
    /// let response = client.execute(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, mut request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Apply interceptors
        for interceptor in &self.interceptors {
            interceptor.intercept(&mut request)?;
        }

        // Execute through transport
        self.transport.execute(request).await
    }

    /// Build a full URL by combining the base URL with a path.
    ///
    /// This method handles trailing/leading slashes automatically.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpResponse, RpcError};
    /// # use async_trait::async_trait;
    /// # struct MyTransport;
    /// # #[async_trait]
    /// # impl HttpTransport for MyTransport {
    /// #     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {
    /// #         todo!()
    /// #     }
    /// # }
    /// # fn example() -> Result<(), RpcError> {
    /// # let transport = MyTransport;
    /// let client = RestconfClient::new("https://device.example.com", transport)?;
    /// let url = client.build_url("/restconf/data/device");
    /// assert_eq!(url, "https://device.example.com/restconf/data/device");
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }
}

/// Server request structure.
///
/// Represents an incoming HTTP request on the server side with method, path,
/// headers, and optional body. This structure is used by server-side handlers
/// to process RESTCONF requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRequest {
    /// The HTTP method for this request
    pub method: HttpMethod,
    /// The request path (without base URL)
    pub path: String,
    /// HTTP headers as name-value pairs
    pub headers: Vec<(String, String)>,
    /// Optional request body as raw bytes
    pub body: Option<Vec<u8>>,
}

impl ServerRequest {
    /// Create a new server request.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{ServerRequest, HttpMethod};
    ///
    /// let request = ServerRequest::new(HttpMethod::GET, "/restconf/data/interfaces");
    /// assert_eq!(request.path, "/restconf/data/interfaces");
    /// assert_eq!(request.method, HttpMethod::GET);
    /// ```
    pub fn new(method: HttpMethod, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// Add a header to the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{ServerRequest, HttpMethod};
    ///
    /// let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/restart")
    ///     .with_header("Content-Type", "application/json");
    /// ```
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the request body.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{ServerRequest, HttpMethod};
    ///
    /// let body = b"{\"delay\": 5}".to_vec();
    /// let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/restart")
    ///     .with_body(body);
    /// ```
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Get a header value by name (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{ServerRequest, HttpMethod};
    ///
    /// let request = ServerRequest::new(HttpMethod::GET, "/restconf/data/interfaces")
    ///     .with_header("Accept", "application/json");
    /// assert_eq!(request.get_header("accept"), Some("application/json"));
    /// ```
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }
}

/// Server response structure.
///
/// Represents an HTTP response to be sent from the server with status code,
/// headers, and body. This structure is returned by server-side handlers
/// after processing RESTCONF requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    /// HTTP status code (e.g., 200, 404, 500)
    pub status_code: u16,
    /// HTTP headers as name-value pairs
    pub headers: Vec<(String, String)>,
    /// Response body as raw bytes
    pub body: Vec<u8>,
}

impl ServerResponse {
    /// Create a new server response.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::ServerResponse;
    ///
    /// let response = ServerResponse::new(200);
    /// assert_eq!(response.status_code, 200);
    /// assert!(response.is_success());
    /// ```
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a success response with JSON body.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::ServerResponse;
    ///
    /// let body = b"{\"status\": \"ok\"}".to_vec();
    /// let response = ServerResponse::json(200, body);
    /// assert_eq!(response.get_header("content-type"), Some("application/json"));
    /// ```
    pub fn json(status_code: u16, body: Vec<u8>) -> Self {
        Self {
            status_code,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
        }
    }

    /// Create an error response from a ServerError.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::{ServerResponse, ServerError};
    ///
    /// let error = ServerError::NotFound("Resource not found".to_string());
    /// let response = ServerResponse::from_error(error);
    /// assert_eq!(response.status_code, 404);
    /// ```
    pub fn from_error(error: ServerError) -> Self {
        let status_code = error.status_code();
        let body = error.to_restconf_error().into_bytes();
        Self::json(status_code, body)
    }

    /// Add a header to the response.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::ServerResponse;
    ///
    /// let response = ServerResponse::new(200)
    ///     .with_header("Cache-Control", "no-cache");
    /// ```
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the response body.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::ServerResponse;
    ///
    /// let body = b"{\"result\": \"success\"}".to_vec();
    /// let response = ServerResponse::new(200)
    ///     .with_body(body);
    /// ```
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Check if the response status indicates success (2xx).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::ServerResponse;
    ///
    /// assert!(ServerResponse::new(200).is_success());
    /// assert!(ServerResponse::new(201).is_success());
    /// assert!(!ServerResponse::new(404).is_success());
    /// ```
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// Get a header value by name (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf_runtime::ServerResponse;
    ///
    /// let response = ServerResponse::new(200)
    ///     .with_header("Content-Type", "application/json");
    /// assert_eq!(response.get_header("content-type"), Some("application/json"));
    /// ```
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }
}

/// Trait for server-side HTTP transport implementations.
///
/// This trait abstracts over different server frameworks (axum, actix-web, warp, etc.)
/// allowing users to choose their preferred framework or implement custom server transports.
/// It mirrors the client-side HttpTransport pattern for consistency.
///
/// # Examples
///
/// Implementing a custom server transport:
///
/// ```no_run
/// use rustconf_runtime::{ServerTransport, ServerRequest, ServerResponse, ServerError};
/// use async_trait::async_trait;
///
/// struct MyServerTransport {
///     // Your server state
/// }
///
/// #[async_trait]
/// impl ServerTransport for MyServerTransport {
///     async fn serve<F>(
///         &self,
///         handler: F,
///         bind_addr: impl Into<String> + Send,
///     ) -> Result<(), ServerError>
///     where
///         F: Fn(ServerRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = ServerResponse> + Send>> + Send + Sync + 'static,
///     {
///         // Your custom server implementation
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait ServerTransport: Send + Sync {
    /// Start the server and begin accepting requests.
    ///
    /// This method starts the server on the specified bind address and routes
    /// incoming requests to the provided handler function.
    ///
    /// # Arguments
    ///
    /// * `handler` - A function that processes ServerRequest and returns ServerResponse
    /// * `bind_addr` - The address to bind the server to (e.g., "127.0.0.1:8080")
    ///
    /// # Returns
    ///
    /// Returns Ok(()) when the server shuts down gracefully, or a ServerError on failure.
    ///
    /// # Errors
    ///
    /// This method should return:
    /// - `ServerError::InternalError` for server startup or binding failures
    /// - `ServerError::InternalError` for unexpected runtime errors
    async fn serve<F>(
        &self,
        handler: F,
        bind_addr: impl Into<String> + Send,
    ) -> Result<(), ServerError>
    where
        F: Fn(
                ServerRequest,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ServerResponse> + Send>>
            + Send
            + Sync
            + 'static;
}
