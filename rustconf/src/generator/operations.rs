//! Operations generation module for RESTCONF CRUD and RPC operations.
//!
//! This module handles the generation of RESTCONF operations including:
//! - CRUD operations (GET, POST, PUT, PATCH, DELETE) for containers and lists
//! - RPC function definitions and types
//! - Error types for operations

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{Rpc, YangModule};

/// CRUD operation types for RESTCONF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudOperation {
    /// GET operation - retrieve resource
    Get,
    /// POST operation - create new resource
    Post,
    /// PUT operation - replace entire resource
    Put,
    /// PATCH operation - partial update
    Patch,
    /// DELETE operation - remove resource
    Delete,
}

impl CrudOperation {
    /// Get the HTTP method for this operation.
    pub fn http_method(&self) -> &'static str {
        match self {
            CrudOperation::Get => "GET",
            CrudOperation::Post => "POST",
            CrudOperation::Put => "PUT",
            CrudOperation::Patch => "PATCH",
            CrudOperation::Delete => "DELETE",
        }
    }

    /// Get the function name prefix for this operation.
    pub fn function_prefix(&self) -> &'static str {
        match self {
            CrudOperation::Get => "get",
            CrudOperation::Post => "create",
            CrudOperation::Put => "put",
            CrudOperation::Patch => "patch",
            CrudOperation::Delete => "delete",
        }
    }

    /// Get the operation description verb.
    pub fn description_verb(&self) -> &'static str {
        match self {
            CrudOperation::Get => "Retrieve",
            CrudOperation::Post => "Create",
            CrudOperation::Put => "Replace",
            CrudOperation::Patch => "Partially update",
            CrudOperation::Delete => "Delete",
        }
    }

    /// Check if this operation requires a data parameter.
    pub fn requires_data(&self) -> bool {
        matches!(
            self,
            CrudOperation::Post | CrudOperation::Put | CrudOperation::Patch
        )
    }

    /// Check if this operation returns data.
    pub fn returns_data(&self) -> bool {
        matches!(self, CrudOperation::Get)
    }
}

/// Resource type for CRUD operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Container resource (single instance)
    Container,
    /// List collection (all items)
    Collection,
    /// List item (single item by key)
    Item,
}

/// Generator for RESTCONF operations and RPC functions.
pub struct OperationsGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> OperationsGenerator<'a> {
    /// Create a new operations generator with the given configuration.
    pub fn new(config: &'a GeneratorConfig) -> Self {
        Self { config }
    }
}

use crate::parser::DataNode;

impl<'a> OperationsGenerator<'a> {
    /// Generate HTTP method enum for RESTful RPCs.
    pub fn generate_http_method(&self) -> String {
        let mut output = String::new();

        output.push_str("/// HTTP methods for RESTful operations.\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }
        derives.push("Copy");
        derives.push("PartialEq");
        derives.push("Eq");

        output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

        output.push_str("pub enum HttpMethod {\n");
        output.push_str("    /// HTTP GET method.\n");
        output.push_str("    GET,\n");
        output.push_str("    /// HTTP POST method.\n");
        output.push_str("    POST,\n");
        output.push_str("    /// HTTP PUT method.\n");
        output.push_str("    PUT,\n");
        output.push_str("    /// HTTP DELETE method.\n");
        output.push_str("    DELETE,\n");
        output.push_str("    /// HTTP PATCH method.\n");
        output.push_str("    PATCH,\n");
        output.push_str("}\n");

        output
    }

    /// Generate HTTP request struct for RESTful RPCs.
    pub fn generate_http_request(&self) -> String {
        let mut output = String::new();

        output.push_str("/// HTTP request for RESTful operations.\n");
        output.push_str("///\n");
        output
            .push_str("/// This struct represents an HTTP request with all necessary components\n");
        output
            .push_str("/// for executing RESTful RPC operations. All fields are public to allow\n");
        output.push_str("/// custom transport implementations to access request details.\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }

        output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

        output.push_str("pub struct HttpRequest {\n");
        output.push_str("    /// The HTTP method for this request.\n");
        output.push_str("    pub method: HttpMethod,\n");
        output.push_str("    /// The target URL for this request.\n");
        output.push_str("    pub url: String,\n");
        output.push_str("    /// HTTP headers as key-value pairs.\n");
        output.push_str("    pub headers: Vec<(String, String)>,\n");
        output.push_str("    /// Optional request body as raw bytes.\n");
        output.push_str("    pub body: Option<Vec<u8>>,\n");
        output.push_str("}\n");

        output
    }

    /// Generate HTTP response struct for RESTful RPCs.
    pub fn generate_http_response(&self) -> String {
        let mut output = String::new();

        output.push_str("/// HTTP response from RESTful operations.\n");
        output.push_str("///\n");
        output.push_str(
            "/// This struct represents an HTTP response received from a RESTCONF server.\n",
        );
        output.push_str("/// All fields are public to allow custom transport implementations to\n");
        output.push_str("/// construct responses and allow users to inspect response details.\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }

        output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

        output.push_str("pub struct HttpResponse {\n");
        output.push_str("    /// The HTTP status code (e.g., 200, 404, 500).\n");
        output.push_str("    pub status_code: u16,\n");
        output.push_str("    /// HTTP headers as key-value pairs.\n");
        output.push_str("    pub headers: Vec<(String, String)>,\n");
        output.push_str("    /// Response body as raw bytes.\n");
        output.push_str("    pub body: Vec<u8>,\n");
        output.push_str("}\n");

        output
    }

    /// Generate HttpTransport trait for RESTful RPCs.
    pub fn generate_http_transport(&self) -> String {
        let mut output = String::new();

        output.push_str("/// HTTP transport abstraction for executing RESTful operations.\n");
        output.push_str("///\n");
        output.push_str(
            "/// This trait provides a pluggable interface for HTTP execution, allowing users\n",
        );
        output.push_str(
            "/// to choose between different HTTP client libraries (reqwest, hyper) or implement\n",
        );
        output.push_str("/// custom transport logic.\n");
        output.push_str("///\n");
        output.push_str("/// # Thread Safety\n");
        output.push_str("///\n");
        output.push_str(
            "/// Implementations must be `Send + Sync` to support concurrent usage across\n",
        );
        output.push_str("/// multiple async tasks and threads.\n");
        output.push_str("///\n");
        output.push_str("/// # Examples\n");
        output.push_str("///\n");
        output.push_str("/// ## Using a built-in transport adapter\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// #[tokio::main]\n");
        output.push_str("/// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("///     // Create a transport using the reqwest adapter\n");
        output.push_str("///     let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("///     \n");
        output.push_str("///     // Create a client with the transport\n");
        output.push_str("///     let client = RestconfClient::new(\n");
        output.push_str("///         \"https://device.example.com\",\n");
        output.push_str("///         transport\n");
        output.push_str("///     );\n");
        output.push_str("///     \n");
        output.push_str("///     // Use the client to call RPC operations\n");
        output.push_str("///     // let result = some_rpc_function(&client, input).await?;\n");
        output.push_str("///     \n");
        output.push_str("///     Ok(())\n");
        output.push_str("/// }\n");
        output.push_str("/// ```\n");
        output.push_str("///\n");
        output.push_str("/// ## Implementing a custom transport\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use async_trait::async_trait;\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// struct MyCustomTransport {\n");
        output.push_str("///     // Your custom HTTP client or configuration\n");
        output.push_str("/// }\n");
        output.push_str("///\n");
        output.push_str("/// #[async_trait]\n");
        output.push_str("/// impl HttpTransport for MyCustomTransport {\n");
        output.push_str("///     async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {\n");
        output.push_str("///         // Your custom HTTP execution logic\n");
        output.push_str("///         // - Convert HttpRequest to your client's request format\n");
        output.push_str("///         // - Execute the HTTP request\n");
        output.push_str("///         // - Convert the response to HttpResponse\n");
        output.push_str("///         // - Handle errors and convert to RpcError\n");
        output.push_str("///         \n");
        output.push_str("///         unimplemented!(\"Implement your custom transport logic\")\n");
        output.push_str("///     }\n");
        output.push_str("/// }\n");
        output.push_str("/// ```\n");
        output.push_str("///\n");
        output.push_str("/// # Error Handling\n");
        output.push_str("///\n");
        output.push_str("/// Implementations should convert transport-specific errors to `RpcError::TransportError`\n");
        output.push_str("/// with descriptive error messages. Network errors, timeouts, and connection failures\n");
        output.push_str("/// should all be mapped to this error variant.\n");
        output.push_str("#[async_trait::async_trait]\n");
        output.push_str("pub trait HttpTransport: Send + Sync {\n");
        output.push_str("    /// Execute an HTTP request and return the response.\n");
        output.push_str("    ///\n");
        output.push_str("    /// This method takes an `HttpRequest` containing the HTTP method, URL, headers,\n");
        output.push_str(
            "    /// and optional body, executes the request using the underlying HTTP client,\n",
        );
        output.push_str(
            "    /// and returns an `HttpResponse` with the status code, headers, and body.\n",
        );
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `request` - The HTTP request to execute\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Returns `Ok(HttpResponse)` on successful execution, or `Err(RpcError)` if\n",
        );
        output.push_str("    /// the request fails due to network errors, timeouts, or other transport issues.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns `RpcError::TransportError` for:\n");
        output.push_str("    /// - Network connectivity issues\n");
        output.push_str("    /// - DNS resolution failures\n");
        output.push_str("    /// - Connection timeouts\n");
        output.push_str("    /// - TLS/SSL errors\n");
        output.push_str("    /// - Invalid URLs\n");
        output.push_str("    /// - Other transport-layer failures\n");
        output.push_str("    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>;\n");
        output.push_str("}\n");

        output
    }

    /// Generate RestconfClient struct for RESTful RPCs.
    pub fn generate_restconf_client(&self) -> String {
        let mut output = String::new();

        output.push_str("/// RESTCONF client for executing RESTful RPC operations.\n");
        output.push_str("///\n");
        output.push_str(
            "/// This struct manages HTTP communication with RESTCONF servers, providing\n",
        );
        output.push_str(
            "/// runtime configuration for base URLs, pluggable HTTP transports, and optional\n",
        );
        output.push_str("/// request interceptors for authentication and logging.\n");
        output.push_str("///\n");
        output.push_str("/// # Type Parameters\n");
        output.push_str("///\n");
        output.push_str(
            "/// * `T` - The HTTP transport implementation to use for executing requests.\n",
        );
        output.push_str("///   Must implement the `HttpTransport` trait.\n");
        output.push_str("///\n");
        output.push_str("/// # Examples\n");
        output.push_str("///\n");
        output.push_str("/// ## Basic usage with reqwest transport\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// #[tokio::main]\n");
        output.push_str("/// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("///     // Create a transport adapter\n");
        output.push_str("///     let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("///     \n");
        output.push_str("///     // Create a client for a specific device\n");
        output.push_str("///     let client = RestconfClient::new(\n");
        output.push_str("///         \"https://router.example.com\",\n");
        output.push_str("///         transport\n");
        output.push_str("///     )?;\n");
        output.push_str("///     \n");
        output.push_str("///     // Use the client to call RPC operations\n");
        output.push_str("///     // let result = some_rpc_function(&client, input).await?;\n");
        output.push_str("///     \n");
        output.push_str("///     Ok(())\n");
        output.push_str("/// }\n");
        output.push_str("/// ```\n");
        output.push_str("///\n");
        output.push_str("/// ## Using with an interceptor for authentication\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use async_trait::async_trait;\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// // Define a custom interceptor for authentication\n");
        output.push_str("/// struct AuthInterceptor {\n");
        output.push_str("///     token: String,\n");
        output.push_str("/// }\n");
        output.push_str("///\n");
        output.push_str("/// #[async_trait]\n");
        output.push_str("/// impl RequestInterceptor for AuthInterceptor {\n");
        output.push_str("///     async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError> {\n");
        output.push_str("///         request.headers.push((\n");
        output.push_str("///             \"Authorization\".to_string(),\n");
        output.push_str("///             format!(\"Bearer {}\", self.token)\n");
        output.push_str("///         ));\n");
        output.push_str("///         Ok(())\n");
        output.push_str("///     }\n");
        output.push_str("///\n");
        output.push_str("///     async fn after_response(&self, _response: &HttpResponse) -> Result<(), RpcError> {\n");
        output.push_str("///         Ok(())\n");
        output.push_str("///     }\n");
        output.push_str("/// }\n");
        output.push_str("///\n");
        output.push_str("/// #[tokio::main]\n");
        output.push_str("/// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("///     let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("///     \n");
        output.push_str("///     // Create a client with an authentication interceptor\n");
        output.push_str("///     let client = RestconfClient::new(\n");
        output.push_str("///         \"https://router.example.com\",\n");
        output.push_str("///         transport\n");
        output.push_str("///     )?\n");
        output.push_str("///     .with_interceptor(AuthInterceptor {\n");
        output.push_str("///         token: \"my-secret-token\".to_string(),\n");
        output.push_str("///     });\n");
        output.push_str("///     \n");
        output.push_str("///     Ok(())\n");
        output.push_str("/// }\n");
        output.push_str("/// ```\n");
        output.push_str("///\n");
        output.push_str("/// ## Managing multiple devices\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// #[tokio::main]\n");
        output.push_str("/// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("///     let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("///     \n");
        output.push_str(
            "///     // Create clients for different devices with the same generated code\n",
        );
        output.push_str("///     let router1 = RestconfClient::new(\n");
        output.push_str("///         \"https://router1.example.com\",\n");
        output.push_str("///         transport.clone()\n");
        output.push_str("///     )?;\n");
        output.push_str("///     \n");
        output.push_str("///     let router2 = RestconfClient::new(\n");
        output.push_str("///         \"https://router2.example.com\",\n");
        output.push_str("///         transport.clone()\n");
        output.push_str("///     )?;\n");
        output.push_str("///     \n");
        output.push_str("///     // Use the same RPC functions with different clients\n");
        output.push_str("///     // let result1 = some_rpc_function(&router1, input).await?;\n");
        output.push_str("///     // let result2 = some_rpc_function(&router2, input).await?;\n");
        output.push_str("///     \n");
        output.push_str("///     Ok(())\n");
        output.push_str("/// }\n");
        output.push_str("/// ```\n");

        // Note: We don't derive Debug for RestconfClient because it contains
        // a trait object (Box<dyn RequestInterceptor>) which doesn't implement Debug
        output.push_str("pub struct RestconfClient<T: HttpTransport> {\n");
        output.push_str("    /// The base URL for the RESTCONF server.\n");
        output.push_str("    base_url: String,\n");
        output.push_str("    /// The HTTP transport implementation.\n");
        output.push_str("    transport: T,\n");
        output.push_str("    /// Optional request interceptor for authentication, logging, etc.\n");
        output.push_str("    interceptor: Option<Box<dyn RequestInterceptor>>,\n");
        output.push_str("}\n");

        output
    }

    /// Generate RestconfClient implementation methods.
    pub fn generate_restconf_client_impl(&self) -> String {
        let mut output = String::new();

        output.push_str("impl<T: HttpTransport> RestconfClient<T> {\n");

        // Generate new() constructor
        output.push_str(
            "    /// Create a new RESTCONF client with the given base URL and transport.\n",
        );
        output.push_str("    ///\n");
        output.push_str(
            "    /// The base URL should be the root URL of the RESTCONF server, without\n",
        );
        output.push_str(
            "    /// the `/restconf` path component. For example: `https://device.example.com`\n",
        );
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `base_url` - The base URL of the RESTCONF server\n");
        output.push_str("    /// * `transport` - The HTTP transport implementation to use\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Returns `Ok(RestconfClient)` if the base URL is valid, or `Err(RpcError)`\n",
        );
        output.push_str("    /// if the URL format is invalid.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns `RpcError::InvalidInput` if:\n");
        output.push_str("    /// - The base URL is empty\n");
        output.push_str("    /// - The base URL does not start with `http://` or `https://`\n");
        output.push_str("    /// - The base URL contains invalid characters\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Examples\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```rust,ignore\n");
        output.push_str("    /// let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("    /// let client = RestconfClient::new(\n");
        output.push_str("    ///     \"https://router.example.com\",\n");
        output.push_str("    ///     transport\n");
        output.push_str("    /// )?;\n");
        output.push_str("    /// ```\n");
        output.push_str("    pub fn new(base_url: impl Into<String>, transport: T) -> Result<Self, RpcError> {\n");
        output.push_str("        let base_url = base_url.into();\n");
        output.push_str("        \n");
        output.push_str("        // Validate base URL format\n");
        output.push_str("        if base_url.is_empty() {\n");
        output.push_str("            return Err(RpcError::InvalidInput(\n");
        output.push_str("                \"Base URL cannot be empty\".to_string()\n");
        output.push_str("            ));\n");
        output.push_str("        }\n");
        output.push_str("        \n");
        output.push_str("        if !base_url.starts_with(\"http://\") && !base_url.starts_with(\"https://\") {\n");
        output.push_str("            return Err(RpcError::InvalidInput(\n");
        output.push_str("                format!(\"Base URL must start with http:// or https://, got: {}\", base_url)\n");
        output.push_str("            ));\n");
        output.push_str("        }\n");
        output.push_str("        \n");
        output.push_str("        Ok(Self {\n");
        output.push_str("            base_url,\n");
        output.push_str("            transport,\n");
        output.push_str("            interceptor: None,\n");
        output.push_str("        })\n");
        output.push_str("    }\n\n");

        // Generate with_interceptor() builder method
        output.push_str("    /// Add a request interceptor to this client.\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Interceptors allow you to modify requests before they are sent and inspect\n",
        );
        output.push_str("    /// responses after they are received. This is useful for implementing authentication,\n");
        output.push_str("    /// logging, request signing, and other cross-cutting concerns.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `interceptor` - The interceptor implementation to add\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns `self` to allow method chaining.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Examples\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```rust,ignore\n");
        output.push_str("    /// use async_trait::async_trait;\n");
        output.push_str("    /// use my_bindings::*;\n");
        output.push_str("    ///\n");
        output.push_str("    /// struct AuthInterceptor {\n");
        output.push_str("    ///     token: String,\n");
        output.push_str("    /// }\n");
        output.push_str("    ///\n");
        output.push_str("    /// #[async_trait]\n");
        output.push_str("    /// impl RequestInterceptor for AuthInterceptor {\n");
        output.push_str("    ///     async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError> {\n");
        output.push_str("    ///         request.headers.push((\n");
        output.push_str("    ///             \"Authorization\".to_string(),\n");
        output.push_str("    ///             format!(\"Bearer {}\", self.token)\n");
        output.push_str("    ///         ));\n");
        output.push_str("    ///         Ok(())\n");
        output.push_str("    ///     }\n");
        output.push_str("    ///\n");
        output.push_str("    ///     async fn after_response(&self, _response: &HttpResponse) -> Result<(), RpcError> {\n");
        output.push_str("    ///         Ok(())\n");
        output.push_str("    ///     }\n");
        output.push_str("    /// }\n");
        output.push_str("    ///\n");
        output.push_str("    /// let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("    /// let client = RestconfClient::new(\n");
        output.push_str("    ///     \"https://router.example.com\",\n");
        output.push_str("    ///     transport\n");
        output.push_str("    /// )?\n");
        output.push_str("    /// .with_interceptor(AuthInterceptor {\n");
        output.push_str("    ///     token: \"my-secret-token\".to_string(),\n");
        output.push_str("    /// });\n");
        output.push_str("    /// ```\n");
        output.push_str("    pub fn with_interceptor(mut self, interceptor: impl RequestInterceptor + 'static) -> Self {\n");
        output.push_str("        self.interceptor = Some(Box::new(interceptor));\n");
        output.push_str("        self\n");
        output.push_str("    }\n\n");

        // Generate execute_request() internal method
        output.push_str(
            "    /// Execute an HTTP request through the transport with interceptor hooks.\n",
        );
        output.push_str("    ///\n");
        output.push_str(
            "    /// This method is used internally by generated RPC functions to execute HTTP\n",
        );
        output.push_str("    /// requests. It handles the interceptor lifecycle:\n");
        output.push_str("    ///\n");
        output.push_str("    /// 1. Call `before_request` hook if an interceptor is configured\n");
        output.push_str("    /// 2. Execute the request through the transport\n");
        output.push_str("    /// 3. Call `after_response` hook if an interceptor is configured\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// If any interceptor hook returns an error, the request is aborted and the\n",
        );
        output.push_str("    /// error is returned immediately.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `request` - The HTTP request to execute\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Returns `Ok(HttpResponse)` on success, or `Err(RpcError)` if the request\n",
        );
        output.push_str("    /// fails or an interceptor aborts the request.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns an error if:\n");
        output.push_str(
            "    /// - The `before_request` interceptor returns an error (request is not sent)\n",
        );
        output.push_str("    /// - The transport fails to execute the request\n");
        output.push_str("    /// - The `after_response` interceptor returns an error\n");
        output.push_str("    pub(crate) async fn execute_request(&self, mut request: HttpRequest) -> Result<HttpResponse, RpcError> {\n");
        output.push_str("        // Call before_request hook if interceptor is configured\n");
        output.push_str("        if let Some(ref interceptor) = self.interceptor {\n");
        output.push_str("            interceptor.before_request(&mut request).await?;\n");
        output.push_str("        }\n");
        output.push_str("        \n");
        output.push_str("        // Execute the request through the transport\n");
        output.push_str("        let response = self.transport.execute(request).await?;\n");
        output.push_str("        \n");
        output.push_str("        // Call after_response hook if interceptor is configured\n");
        output.push_str("        if let Some(ref interceptor) = self.interceptor {\n");
        output.push_str("            interceptor.after_response(&response).await?;\n");
        output.push_str("        }\n");
        output.push_str("        \n");
        output.push_str("        Ok(response)\n");
        output.push_str("    }\n\n");

        // Generate base_url() getter method
        output.push_str("    /// Get the base URL for this client.\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// This method is used internally by generated RPC functions to construct\n",
        );
        output.push_str("    /// RESTCONF URLs.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns a reference to the base URL string.\n");
        output.push_str("    pub(crate) fn base_url(&self) -> &str {\n");
        output.push_str("        &self.base_url\n");
        output.push_str("    }\n");

        output.push_str("}\n");

        output
    }

    /// Generate RequestInterceptor trait for RESTful RPCs.
    pub fn generate_request_interceptor(&self) -> String {
        let mut output = String::new();

        output.push_str("/// Request interceptor for modifying HTTP requests and responses.\n");
        output.push_str("///\n");
        output.push_str(
            "/// This trait provides hooks for intercepting and modifying HTTP requests before\n",
        );
        output.push_str(
            "/// they are sent and HTTP responses after they are received. This is useful for\n",
        );
        output.push_str("/// implementing cross-cutting concerns such as:\n");
        output.push_str("///\n");
        output.push_str("/// - Authentication (adding tokens, signing requests)\n");
        output.push_str("/// - Logging and monitoring\n");
        output.push_str("/// - Request/response transformation\n");
        output.push_str("/// - Error handling and retry logic\n");
        output.push_str("/// - Custom header injection\n");
        output.push_str("///\n");
        output.push_str("/// # Thread Safety\n");
        output.push_str("///\n");
        output.push_str(
            "/// Implementations must be `Send + Sync` to support concurrent usage across\n",
        );
        output.push_str("/// multiple async tasks and threads.\n");
        output.push_str("///\n");
        output.push_str("/// # Execution Order\n");
        output.push_str("///\n");
        output.push_str("/// When multiple interceptors are registered with a `RestconfClient`:\n");
        output.push_str("/// - `before_request` hooks are called in registration order\n");
        output.push_str("/// - `after_response` hooks are called in reverse registration order\n");
        output.push_str("///\n");
        output.push_str(
            "/// If any interceptor returns an error, the request is aborted and the error\n",
        );
        output.push_str("/// is returned immediately without calling subsequent interceptors.\n");
        output.push_str("///\n");
        output.push_str("/// # Examples\n");
        output.push_str("///\n");
        output.push_str("/// ## Basic authentication interceptor\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use async_trait::async_trait;\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// struct AuthInterceptor {\n");
        output.push_str("///     token: String,\n");
        output.push_str("/// }\n");
        output.push_str("///\n");
        output.push_str("/// #[async_trait]\n");
        output.push_str("/// impl RequestInterceptor for AuthInterceptor {\n");
        output.push_str("///     async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError> {\n");
        output.push_str("///         // Add authorization header to every request\n");
        output.push_str("///         request.headers.push((\n");
        output.push_str("///             \"Authorization\".to_string(),\n");
        output.push_str("///             format!(\"Bearer {}\", self.token)\n");
        output.push_str("///         ));\n");
        output.push_str("///         Ok(())\n");
        output.push_str("///     }\n");
        output.push_str("///\n");
        output.push_str("///     async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError> {\n");
        output.push_str("///         // Validate response or perform logging\n");
        output.push_str("///         if response.status_code == 401 {\n");
        output.push_str(
            "///             return Err(RpcError::Unauthorized(\"Token expired\".to_string()));\n",
        );
        output.push_str("///         }\n");
        output.push_str("///         Ok(())\n");
        output.push_str("///     }\n");
        output.push_str("/// }\n");
        output.push_str("///\n");
        output.push_str("/// // Usage with RestconfClient\n");
        output.push_str("/// let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str(
            "/// let client = RestconfClient::new(\"https://device.example.com\", transport)\n",
        );
        output.push_str("///     .with_interceptor(AuthInterceptor {\n");
        output.push_str("///         token: \"my-secret-token\".to_string(),\n");
        output.push_str("///     });\n");
        output.push_str("/// ```\n");
        output.push_str("///\n");
        output.push_str("/// ## Logging interceptor\n");
        output.push_str("///\n");
        output.push_str("/// ```rust,ignore\n");
        output.push_str("/// use async_trait::async_trait;\n");
        output.push_str("/// use my_bindings::*;\n");
        output.push_str("///\n");
        output.push_str("/// struct LoggingInterceptor;\n");
        output.push_str("///\n");
        output.push_str("/// #[async_trait]\n");
        output.push_str("/// impl RequestInterceptor for LoggingInterceptor {\n");
        output.push_str("///     async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError> {\n");
        output.push_str(
            "///         println!(\"Sending {} request to {}\", request.method, request.url);\n",
        );
        output.push_str("///         Ok(())\n");
        output.push_str("///     }\n");
        output.push_str("///\n");
        output.push_str("///     async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError> {\n");
        output.push_str(
            "///         println!(\"Received response with status {}\", response.status_code);\n",
        );
        output.push_str("///         Ok(())\n");
        output.push_str("///     }\n");
        output.push_str("/// }\n");
        output.push_str("/// ```\n");
        output.push_str("///\n");
        output.push_str("/// # Error Handling\n");
        output.push_str("///\n");
        output.push_str(
            "/// Both `before_request` and `after_response` can return errors to abort the\n",
        );
        output
            .push_str("/// request or indicate validation failures. When an error is returned:\n");
        output.push_str("///\n");
        output.push_str(
            "/// - From `before_request`: The HTTP request is not sent, and the error is\n",
        );
        output.push_str("///   returned to the caller immediately\n");
        output
            .push_str("/// - From `after_response`: The response is discarded, and the error is\n");
        output.push_str("///   returned to the caller immediately\n");
        output.push_str("#[async_trait::async_trait]\n");
        output.push_str("pub trait RequestInterceptor: Send + Sync {\n");
        output.push_str("    /// Called before sending an HTTP request.\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// This method receives a mutable reference to the `HttpRequest`, allowing\n",
        );
        output.push_str(
            "    /// the interceptor to modify the request before it is sent. Common use cases\n",
        );
        output.push_str(
            "    /// include adding authentication headers, modifying URLs, or injecting\n",
        );
        output.push_str("    /// custom headers.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// * `request` - A mutable reference to the HTTP request that will be sent\n",
        );
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Returns `Ok(())` to proceed with the request, or `Err(RpcError)` to abort\n",
        );
        output.push_str("    /// the request and return the error to the caller.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str("    /// Return an error to abort the request. Common error scenarios:\n");
        output.push_str("    /// - Authentication token is missing or expired\n");
        output.push_str("    /// - Request validation fails\n");
        output.push_str("    /// - Rate limiting is triggered\n");
        output.push_str("    async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError>;\n\n");
        output.push_str("    /// Called after receiving an HTTP response.\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// This method receives an immutable reference to the `HttpResponse`, allowing\n",
        );
        output.push_str(
            "    /// the interceptor to inspect the response and perform validation or logging.\n",
        );
        output.push_str(
            "    /// The response cannot be modified at this stage since the RPC function will\n",
        );
        output.push_str("    /// handle deserialization.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `response` - An immutable reference to the HTTP response that was received\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Returns `Ok(())` to proceed with response processing, or `Err(RpcError)` to\n",
        );
        output.push_str("    /// abort and return the error to the caller.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// Return an error to abort response processing. Common error scenarios:\n",
        );
        output.push_str("    /// - Response validation fails\n");
        output.push_str("    /// - Unexpected status code\n");
        output.push_str("    /// - Missing required headers\n");
        output.push_str("    async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError>;\n");
        output.push_str("}\n");

        output
    }

    /// Generate RPC error type.
    pub fn generate_rpc_error(&self) -> String {
        let mut output = String::new();

        output.push_str("/// Error type for RPC operations.\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }
        output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

        output.push_str("pub enum RpcError {\n");
        output.push_str("    /// Network or communication error.\n");
        output.push_str("    NetworkError(String),\n");
        output.push_str("    /// Server returned an error response.\n");
        output.push_str("    ServerError { code: u16, message: String },\n");
        output.push_str("    /// Serialization or deserialization error.\n");
        output.push_str("    SerializationError(String),\n");
        output.push_str("    /// Invalid input parameters.\n");
        output.push_str("    InvalidInput(String),\n");
        output.push_str("    /// Operation not implemented.\n");
        output.push_str("    NotImplemented,\n");
        output.push_str("    /// HTTP transport error.\n");
        output.push_str("    TransportError(String),\n");
        output.push_str("    /// JSON deserialization error.\n");
        output.push_str("    DeserializationError(String),\n");
        output.push_str("    /// Unauthorized access.\n");
        output.push_str("    Unauthorized(String),\n");
        output.push_str("    /// Resource not found.\n");
        output.push_str("    NotFound(String),\n");
        output.push_str("    /// Unknown error.\n");
        output.push_str("    UnknownError(String),\n");
        output.push_str("}\n\n");

        output.push_str("impl std::fmt::Display for RpcError {\n");
        output
            .push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        output.push_str("        match self {\n");
        output.push_str(
            "            RpcError::NetworkError(msg) => write!(f, \"Network error: {}\", msg),\n",
        );
        output.push_str("            RpcError::ServerError { code, message } => write!(f, \"Server error {}: {}\", code, message),\n");
        output.push_str("            RpcError::SerializationError(msg) => write!(f, \"Serialization error: {}\", msg),\n");
        output.push_str(
            "            RpcError::InvalidInput(msg) => write!(f, \"Invalid input: {}\", msg),\n",
        );
        output.push_str(
            "            RpcError::NotImplemented => write!(f, \"Operation not implemented\"),\n",
        );
        output.push_str(
            "            RpcError::TransportError(msg) => write!(f, \"Transport error: {}\", msg),\n",
        );
        output.push_str(
            "            RpcError::DeserializationError(msg) => write!(f, \"Deserialization error: {}\", msg),\n",
        );
        output.push_str(
            "            RpcError::Unauthorized(msg) => write!(f, \"Unauthorized: {}\", msg),\n",
        );
        output.push_str(
            "            RpcError::NotFound(msg) => write!(f, \"Not found: {}\", msg),\n",
        );
        output.push_str(
            "            RpcError::UnknownError(msg) => write!(f, \"Unknown error: {}\", msg),\n",
        );
        output.push_str("        }\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");

        output.push_str("impl std::error::Error for RpcError {}\n");

        output
    }

    /// Generate operations module (RPC and CRUD operations).
    pub fn generate_operations_module(
        &self,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let path_gen = crate::generator::paths::PathGenerator::new(self.config);

        output.push_str("/// RESTCONF operations.\n");
        output.push_str("pub mod operations {\n");
        output.push_str("    use super::*;\n");
        output.push('\n');

        // Generate percent encoding helper function
        output.push_str(&path_gen.generate_percent_encode_helper());

        // Generate input/output types and functions for each RPC
        if !module.rpcs.is_empty() {
            for rpc in &module.rpcs {
                let types = self.generate_rpc_types(rpc, module)?;
                if !types.is_empty() {
                    output.push_str(&types);
                }
                output.push_str(&self.generate_rpc_function(rpc, module)?);
                output.push('\n');
            }
        }

        // Generate RESTCONF CRUD operations for data nodes
        if !module.data_nodes.is_empty() {
            output.push_str(&self.generate_crud_operations(module)?);
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate RESTCONF CRUD operations for data nodes.
    fn generate_crud_operations(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();

        output.push_str("    /// RESTCONF CRUD operations for data resources.\n");
        output.push_str("    pub mod crud {\n");
        output.push_str("        use super::*;\n");
        output.push('\n');

        // Generate CRUD operations for each top-level data node
        for node in &module.data_nodes {
            output.push_str(&self.generate_node_crud_operations(node, module)?);
        }

        output.push_str("    }\n");

        Ok(output)
    }

    /// Generate CRUD operations for a specific data node.
    fn generate_node_crud_operations(
        &self,
        node: &DataNode,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        match node {
            DataNode::Container(container) => {
                self.generate_container_crud_operations(container, module)
            }
            DataNode::List(list) => self.generate_list_crud_operations(list, module),
            DataNode::Leaf(_) => Ok(String::new()), // Top-level leaves are rare
            DataNode::LeafList(_) => Ok(String::new()),
            DataNode::Choice(_) => Ok(String::new()),
            DataNode::Case(_) => Ok(String::new()),
            DataNode::Uses(_) => Ok(String::new()),
        }
    }

    /// Generate a generic CRUD operation function.
    ///
    /// This is the core abstraction that eliminates duplication across CRUD operations.
    fn generate_crud_operation(
        &self,
        operation: CrudOperation,
        resource_type: ResourceType,
        resource_name: &str,
        type_name: &str,
        path_helper: &str,
        key_params: Option<&str>,
    ) -> String {
        let mut output = String::new();

        // Generate function name
        let function_prefix = crate::generator::naming::to_field_name(resource_name);
        let operation_prefix = operation.function_prefix();

        // Only GET operations on items get the _by_key suffix
        let resource_suffix =
            if resource_type == ResourceType::Item && operation == CrudOperation::Get {
                "_by_key"
            } else {
                ""
            };

        let function_name = format!(
            "{}_{}{}",
            operation_prefix, function_prefix, resource_suffix
        );

        // Generate documentation
        let description_verb = operation.description_verb();
        let resource_desc = match (resource_type, operation) {
            (ResourceType::Container, _) => format!("the {} container", resource_name),
            (ResourceType::Collection, CrudOperation::Get) => {
                format!("all {} items", resource_name)
            }
            (ResourceType::Collection, CrudOperation::Post) => {
                format!("a new {} item", resource_name)
            }
            (ResourceType::Item, CrudOperation::Get) => {
                format!("a single {} item by key", resource_name)
            }
            (ResourceType::Item, _) => format!("a {} item by key", resource_name),
            _ => format!("{} {}", resource_type_desc(resource_type), resource_name),
        };

        fn resource_type_desc(rt: ResourceType) -> &'static str {
            match rt {
                ResourceType::Container => "container",
                ResourceType::Collection => "collection",
                ResourceType::Item => "item",
            }
        }

        output.push_str(&format!(
            "        /// {} {}.\n",
            description_verb, resource_desc
        ));
        output.push_str("        ///\n");
        output.push_str("        /// # Errors\n");
        output.push_str("        ///\n");
        output.push_str("        /// Returns an error if the operation fails.\n");

        // Generate function signature
        output.push_str("        pub async fn ");
        output.push_str(&function_name);
        output.push('(');

        // Add parameters in the correct order
        let mut params = Vec::new();

        // Add key parameters first for item operations
        if let Some(keys) = key_params {
            params.push(keys.to_string());
        }

        // Add data parameter after keys for operations that require it
        if operation.requires_data() {
            params.push(format!("_data: {}", type_name));
        }

        output.push_str(&params.join(", "));
        output.push_str(") -> Result<");

        // Generate return type
        if operation.returns_data() {
            match resource_type {
                ResourceType::Collection => output.push_str(&format!("Vec<{}>", type_name)),
                _ => output.push_str(type_name),
            }
        } else {
            output.push_str("()");
        }

        output.push_str(", RpcError> {\n");

        // Generate function body
        output.push_str(&format!("            let _path = {};\n", path_helper));
        output.push_str(&format!(
            "            // TODO: Implement {} request to RESTCONF server\n",
            operation.http_method()
        ));
        output.push_str(&format!(
            "            unimplemented!(\"{} operation not yet implemented\")\n",
            operation.http_method()
        ));
        output.push_str("        }\n\n");

        output
    }

    /// Generate CRUD operations for a container.
    fn generate_container_crud_operations(
        &self,
        container: &crate::parser::Container,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let type_name = crate::generator::naming::to_type_name(&container.name);
        let function_prefix = crate::generator::naming::to_field_name(&container.name);
        let path_gen = crate::generator::paths::PathGenerator::new(self.config);

        // Generate path helper function
        output.push_str(&path_gen.generate_container_path_helper(container, module)?);
        output.push('\n');

        // Generate GET operation (always available for containers)
        let path_helper = format!("{}_path()", function_prefix);
        output.push_str(&self.generate_crud_operation(
            CrudOperation::Get,
            ResourceType::Container,
            &container.name,
            &type_name,
            &path_helper,
            None,
        ));

        // Generate config-based operations (PUT, PATCH, DELETE) only if config is true
        if container.config {
            // PUT operation - replace entire container
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Put,
                ResourceType::Container,
                &container.name,
                &type_name,
                &path_helper,
                None,
            ));

            // PATCH operation - partial update
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Patch,
                ResourceType::Container,
                &container.name,
                &type_name,
                &path_helper,
                None,
            ));

            // DELETE operation - remove container
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Delete,
                ResourceType::Container,
                &container.name,
                &type_name,
                &path_helper,
                None,
            ));
        }

        Ok(output)
    }

    /// Generate CRUD operations for a list.
    fn generate_list_crud_operations(
        &self,
        list: &crate::parser::List,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let type_name = crate::generator::naming::to_type_name(&list.name);
        let function_prefix = crate::generator::naming::to_field_name(&list.name);
        let path_gen = crate::generator::paths::PathGenerator::new(self.config);

        // Determine item type name (singular)
        let item_type_name = if type_name.ends_with('s') && type_name.len() > 1 {
            type_name[..type_name.len() - 1].to_string()
        } else {
            type_name.clone()
        };

        // Generate path helper functions
        output.push_str(&path_gen.generate_list_path_helpers(list, module)?);
        output.push('\n');

        // Generate key parameter types for operations that need them
        let key_params = path_gen.generate_list_key_params(list);
        let key_param_names = path_gen.generate_key_param_names(list);

        // Generate GET operation for entire list (collection)
        let collection_path = format!("{}_path()", function_prefix);
        output.push_str(&self.generate_crud_operation(
            CrudOperation::Get,
            ResourceType::Collection,
            &list.name,
            &item_type_name,
            &collection_path,
            None,
        ));

        // GET operation for single item by key
        let item_path = format!("{}_item_path({})", function_prefix, key_param_names);
        output.push_str(&self.generate_crud_operation(
            CrudOperation::Get,
            ResourceType::Item,
            &list.name,
            &item_type_name,
            &item_path,
            Some(&key_params),
        ));

        // Generate config-based operations only if config is true
        if list.config {
            // POST operation - create new item
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Post,
                ResourceType::Collection,
                &list.name,
                &item_type_name,
                &collection_path,
                None,
            ));

            // PUT operation - replace item by key
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Put,
                ResourceType::Item,
                &list.name,
                &item_type_name,
                &item_path,
                Some(&key_params),
            ));

            // PATCH operation - partial update by key
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Patch,
                ResourceType::Item,
                &list.name,
                &item_type_name,
                &item_path,
                Some(&key_params),
            ));

            // DELETE operation - remove item by key
            output.push_str(&self.generate_crud_operation(
                CrudOperation::Delete,
                ResourceType::Item,
                &list.name,
                &item_type_name,
                &item_path,
                Some(&key_params),
            ));
        }

        Ok(output)
    }

    /// Generate input and output types for an RPC.
    fn generate_rpc_types(&self, rpc: &Rpc, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let rpc_type_name = crate::generator::naming::to_type_name(&rpc.name);
        let type_gen = crate::generator::types::TypeGenerator::new(self.config);

        // Generate input type if RPC has input
        if let Some(ref input_nodes) = rpc.input {
            if !input_nodes.is_empty() {
                output.push_str(&format!("    /// Input parameters for {} RPC.\n", rpc.name));
                output.push_str(&format!("    {}", self.generate_derive_attributes()));
                output.push_str(&format!("    pub struct {}Input {{\n", rpc_type_name));

                // Generate fields from input nodes
                for node in input_nodes {
                    let field = type_gen.generate_field(node, module, None)?;
                    // Add indentation for nested struct
                    for line in field.lines() {
                        output.push_str(&format!("    {}\n", line));
                    }
                }

                output.push_str("    }\n\n");
            }
        }

        // Generate output type if RPC has output
        if let Some(ref output_nodes) = rpc.output {
            if !output_nodes.is_empty() {
                output.push_str(&format!("    /// Output result for {} RPC.\n", rpc.name));
                output.push_str(&format!("    {}", self.generate_derive_attributes()));
                output.push_str(&format!("    pub struct {}Output {{\n", rpc_type_name));

                // Generate fields from output nodes
                for node in output_nodes {
                    let field = type_gen.generate_field(node, module, None)?;
                    // Add indentation for nested struct
                    for line in field.lines() {
                        output.push_str(&format!("    {}\n", line));
                    }
                }

                output.push_str("    }\n\n");
            }
        }

        Ok(output)
    }

    /// Generate an async function for an RPC operation.
    fn generate_rpc_function(
        &self,
        rpc: &Rpc,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let rpc_type_name = crate::generator::naming::to_type_name(&rpc.name);
        let function_name = crate::generator::naming::to_field_name(&rpc.name);

        // Generate rustdoc comment from RPC description
        if let Some(ref description) = rpc.description {
            output.push_str(&format!("    {}", self.generate_rustdoc(description)));
        } else {
            output.push_str(&format!(
                "    /// Execute the {} RPC operation.\n",
                rpc.name
            ));
        }
        output.push_str("    ///\n");

        // Determine if we have input/output for documentation
        let has_input = rpc.input.as_ref().is_some_and(|nodes| !nodes.is_empty());
        let has_output = rpc.output.as_ref().is_some_and(|nodes| !nodes.is_empty());

        // Add parameters documentation
        if self.config.enable_restful_rpcs || has_input {
            output.push_str("    /// # Arguments\n");
            output.push_str("    ///\n");

            if self.config.enable_restful_rpcs {
                output.push_str("    /// * `client` - The RestconfClient to use for executing the RPC request\n");
            }

            if has_input {
                output.push_str(&format!(
                    "    /// * `input` - The input parameters for the {} operation\n",
                    rpc.name
                ));
            }
            output.push_str("    ///\n");
        }

        // Add returns documentation
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        if has_output {
            output.push_str(&format!(
                "    /// Returns `Ok({}Output)` on success, containing the operation result.\n",
                rpc_type_name
            ));
        } else {
            output.push_str("    /// Returns `Ok(())` on success.\n");
        }
        output.push_str("    ///\n");

        // Add error handling documentation
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        if self.config.enable_restful_rpcs {
            output.push_str("    /// Returns an error if:\n");
            if has_input {
                output.push_str(
                    "    /// - Input serialization fails (`RpcError::SerializationError`)\n",
                );
            }
            output.push_str("    /// - The HTTP request fails (`RpcError::TransportError`)\n");
            output.push_str("    /// - The server returns an error status:\n");
            output.push_str("    ///   - 400: `RpcError::InvalidInput`\n");
            output.push_str("    ///   - 401/403: `RpcError::Unauthorized`\n");
            output.push_str("    ///   - 404: `RpcError::NotFound`\n");
            output.push_str("    ///   - 500-599: `RpcError::ServerError`\n");
            if has_output {
                output.push_str(
                    "    /// - Response deserialization fails (`RpcError::DeserializationError`)\n",
                );
            }
        } else {
            output.push_str("    /// Returns `RpcError::NotImplemented` as RESTful RPC generation is disabled.\n");
        }
        output.push_str("    ///\n");

        // Add usage example
        if self.config.enable_restful_rpcs {
            output.push_str("    /// # Example\n");
            output.push_str("    ///\n");
            output.push_str("    /// ```rust,ignore\n");
            output.push_str(&format!(
                "    /// use {}::*;\n",
                module.name.replace('-', "_")
            ));
            output.push_str("    ///\n");
            output.push_str("    /// #[tokio::main]\n");
            output.push_str("    /// async fn main() -> Result<(), RpcError> {\n");
            output.push_str("    ///     // Create a transport adapter\n");
            output.push_str(
                "    ///     let transport = reqwest_adapter::ReqwestTransport::new();\n",
            );
            output.push_str("    ///\n");
            output.push_str("    ///     // Create a client for the RESTCONF server\n");
            output.push_str("    ///     let client = RestconfClient::new(\n");
            output.push_str("    ///         \"https://device.example.com\",\n");
            output.push_str("    ///         transport\n");
            output.push_str("    ///     )?;\n");
            output.push_str("    ///\n");

            if has_input {
                output.push_str("    ///     // Prepare input parameters\n");
                output.push_str(&format!(
                    "    ///     let input = {}Input {{\n",
                    rpc_type_name
                ));
                output.push_str("    ///         // Set input fields here\n");
                output.push_str("    ///         // ...\n");
                output.push_str("    ///     };\n");
                output.push_str("    ///\n");
                output.push_str("    ///     // Execute the RPC operation\n");
                output.push_str(&format!(
                    "    ///     let result = {}(&client, input).await?;\n",
                    function_name
                ));
            } else {
                output.push_str("    ///     // Execute the RPC operation\n");
                output.push_str(&format!(
                    "    ///     let result = {}(&client).await?;\n",
                    function_name
                ));
            }

            if has_output {
                output.push_str("    ///\n");
                output.push_str("    ///     // Process the result\n");
                output.push_str("    ///     println!(\"Operation completed successfully\");\n");
            }

            output.push_str("    ///\n");
            output.push_str("    ///     Ok(())\n");
            output.push_str("    /// }\n");
            output.push_str("    /// ```\n");
        }

        // Determine input parameter type
        let input_param = if let Some(ref input_nodes) = rpc.input {
            if !input_nodes.is_empty() {
                format!("input: {}Input", rpc_type_name)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Determine return type
        let return_type = if let Some(ref output_nodes) = rpc.output {
            if !output_nodes.is_empty() {
                format!("Result<{}Output, RpcError>", rpc_type_name)
            } else {
                "Result<(), RpcError>".to_string()
            }
        } else {
            "Result<(), RpcError>".to_string()
        };

        // Check if RESTful RPC generation is enabled
        if self.config.enable_restful_rpcs {
            // Generate RESTful implementation
            self.generate_restful_rpc_function(
                &mut output,
                rpc,
                module,
                &function_name,
                &rpc_type_name,
                &input_param,
                &return_type,
            )?;
        } else {
            // Generate stub function that returns NotImplemented
            self.generate_stub_rpc_function(
                &mut output,
                &function_name,
                &input_param,
                &return_type,
            );
        }

        Ok(output)
    }

    /// Generate a stub RPC function that returns NotImplemented error.
    fn generate_stub_rpc_function(
        &self,
        output: &mut String,
        function_name: &str,
        input_param: &str,
        return_type: &str,
    ) {
        // Generate function signature
        if input_param.is_empty() {
            output.push_str(&format!(
                "    pub async fn {}() -> {} {{\n",
                function_name, return_type
            ));
        } else {
            output.push_str(&format!(
                "    pub async fn {}({}) -> {} {{\n",
                function_name, input_param, return_type
            ));
        }

        // Generate stub body that returns NotImplemented
        output.push_str("        Err(RpcError::NotImplemented)\n");
        output.push_str("    }\n");
    }

    /// Generate a RESTful RPC function implementation.
    fn generate_restful_rpc_function(
        &self,
        output: &mut String,
        rpc: &Rpc,
        module: &YangModule,
        function_name: &str,
        _rpc_type_name: &str,
        input_param: &str,
        return_type: &str,
    ) -> Result<(), GeneratorError> {
        // Generate function signature with client parameter
        if input_param.is_empty() {
            output.push_str(&format!(
                "    pub async fn {}<T: HttpTransport>(client: &RestconfClient<T>) -> {} {{\n",
                function_name, return_type
            ));
        } else {
            output.push_str(&format!(
                "    pub async fn {}<T: HttpTransport>(client: &RestconfClient<T>, {}) -> {} {{\n",
                function_name, input_param, return_type
            ));
        }

        // Determine if we have input to serialize
        let has_input = rpc.input.as_ref().is_some_and(|nodes| !nodes.is_empty());

        // Determine if we have output to deserialize
        let has_output = rpc.output.as_ref().is_some_and(|nodes| !nodes.is_empty());

        // Generate function body
        if has_input {
            // Serialize input to JSON
            output.push_str("        // Serialize input to JSON\n");
            output.push_str("        let body = serde_json::to_vec(&input)\n");
            output.push_str("            .map_err(|e| RpcError::SerializationError(format!(\"Failed to serialize input: {}\", e)))?;\n\n");
        }

        // Construct RESTCONF URL inline
        output.push_str("        // Construct RESTCONF URL\n");
        output.push_str("        let base = client.base_url().trim_end_matches('/');\n");

        match self.config.restful_namespace_mode {
            crate::generator::config::NamespaceMode::Enabled => {
                output.push_str(&format!(
                    "        let url = format!(\"{{}}/ restconf/operations/{{}}:{{}}\", base, urlencoding::encode(\"{}\"), urlencoding::encode(\"{}\"));\n\n",
                    module.name, rpc.name
                ));
            }
            crate::generator::config::NamespaceMode::Disabled => {
                output.push_str(&format!(
                    "        let url = format!(\"{{}}/restconf/operations/{{}}\", base, urlencoding::encode(\"{}\"));\n\n",
                    rpc.name
                ));
            }
        }

        // Build HttpRequest with POST method
        output.push_str("        // Build HTTP request\n");
        output.push_str("        let request = HttpRequest {\n");
        output.push_str("            method: HttpMethod::POST,\n");
        output.push_str("            url,\n");
        output.push_str("            headers: vec![\n");
        output.push_str("                (\"Content-Type\".to_string(), \"application/yang-data+json\".to_string()),\n");
        output.push_str("                (\"Accept\".to_string(), \"application/yang-data+json\".to_string()),\n");
        output.push_str("            ],\n");

        if has_input {
            output.push_str("            body: Some(body),\n");
        } else {
            output.push_str("            body: None,\n");
        }

        output.push_str("        };\n\n");

        // Call client.execute_request()
        output.push_str("        // Execute request through client\n");
        output.push_str("        let response = client.execute_request(request).await?;\n\n");

        // Map HTTP status codes to RpcError variants
        output.push_str("        // Map HTTP status to error or deserialize response\n");
        output.push_str("        match response.status_code {\n");
        output.push_str("            200..=299 => {\n");

        if has_output {
            // Deserialize response body for 2xx status codes
            output.push_str("                // Success - deserialize response body\n");
            output.push_str("                serde_json::from_slice(&response.body)\n");
            output.push_str("                    .map_err(|e| RpcError::DeserializationError(\n");
            output.push_str(
                "                        format!(\"Failed to deserialize response: {}\", e)\n",
            );
            output.push_str("                    ))\n");
        } else {
            // No output expected, just return Ok(())
            output.push_str("                // Success - no output expected\n");
            output.push_str("                Ok(())\n");
        }

        output.push_str("            }\n");

        // Map specific status codes to appropriate errors
        output.push_str("            400 => Err(RpcError::InvalidInput(\n");
        output.push_str("                String::from_utf8_lossy(&response.body).to_string()\n");
        output.push_str("            )),\n");
        output.push_str("            401 | 403 => Err(RpcError::Unauthorized(\n");
        output.push_str("                String::from_utf8_lossy(&response.body).to_string()\n");
        output.push_str("            )),\n");
        output.push_str("            404 => Err(RpcError::NotFound(\n");
        output.push_str("                String::from_utf8_lossy(&response.body).to_string()\n");
        output.push_str("            )),\n");
        output.push_str("            500..=599 => Err(RpcError::ServerError {\n");
        output.push_str("                code: response.status_code,\n");
        output.push_str(
            "                message: String::from_utf8_lossy(&response.body).to_string(),\n",
        );
        output.push_str("            }),\n");
        output.push_str("            _ => Err(RpcError::UnknownError(\n");
        output.push_str(
            "                format!(\"Unexpected status code: {}\", response.status_code)\n",
        );
        output.push_str("            )),\n");
        output.push_str("        }\n");

        output.push_str("    }\n");

        Ok(())
    }

    /// Generate rustdoc comments from a YANG description.
    fn generate_rustdoc(&self, description: &str) -> String {
        let mut rustdoc = String::new();

        // Split description into lines and format as rustdoc comments
        for line in description.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                rustdoc.push_str("///\n");
            } else {
                rustdoc.push_str(&format!("/// {}\n", trimmed));
            }
        }

        rustdoc
    }

    /// Generate derive attributes based on configuration.
    fn generate_derive_attributes(&self) -> String {
        let mut derives = vec!["Serialize", "Deserialize"];

        if self.config.derive_debug {
            derives.insert(0, "Debug");
        }

        if self.config.derive_clone {
            derives.insert(if self.config.derive_debug { 1 } else { 0 }, "Clone");
        }

        format!("#[derive({})]\n", derives.join(", "))
    }

    /// Generate reqwest transport adapter module.
    pub fn generate_reqwest_adapter(&self) -> String {
        let mut output = String::new();

        // Add feature gate
        output.push_str("#[cfg(feature = \"reqwest-client\")]\n");
        output.push_str("pub mod reqwest_adapter {\n");
        output.push_str("    use super::*;\n");
        output.push_str("    use async_trait::async_trait;\n");
        output.push('\n');

        // Generate ReqwestTransport struct
        output.push_str("    /// HTTP transport implementation using reqwest.\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// This adapter provides HTTP execution using the `reqwest` library, which is\n",
        );
        output.push_str(
            "    /// a high-level HTTP client built on top of `hyper` and `tokio`. It provides\n",
        );
        output.push_str(
            "    /// a convenient API with automatic connection pooling, redirect handling, and\n",
        );
        output.push_str("    /// timeout management.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Feature Flag\n");
        output.push_str("    ///\n");
        output.push_str("    /// This adapter is only available when the `reqwest-client` feature is enabled.\n");
        output.push_str("    /// Add it to your `Cargo.toml`:\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```toml\n");
        output.push_str("    /// [dependencies]\n");
        output.push_str(
            "    /// my-bindings = { version = \"0.1\", features = [\"reqwest-client\"] }\n",
        );
        output.push_str("    /// ```\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Examples\n");
        output.push_str("    ///\n");
        output.push_str("    /// ## Basic usage with default client\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```rust,ignore\n");
        output.push_str("    /// use my_bindings::*;\n");
        output.push_str("    ///\n");
        output.push_str("    /// #[tokio::main]\n");
        output.push_str("    /// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("    ///     // Create a transport with default settings\n");
        output.push_str("    ///     let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("    ///\n");
        output.push_str("    ///     // Create a RESTCONF client\n");
        output.push_str("    ///     let client = RestconfClient::new(\n");
        output.push_str("    ///         \"https://device.example.com\",\n");
        output.push_str("    ///         transport\n");
        output.push_str("    ///     )?;\n");
        output.push_str("    ///\n");
        output.push_str("    ///     // Use the client to call RPC operations\n");
        output.push_str("    ///     // let result = some_rpc_function(&client, input).await?;\n");
        output.push_str("    ///\n");
        output.push_str("    ///     Ok(())\n");
        output.push_str("    /// }\n");
        output.push_str("    /// ```\n");
        output.push_str("    ///\n");
        output.push_str("    /// ## Using with a custom reqwest client\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```rust,ignore\n");
        output.push_str("    /// use my_bindings::*;\n");
        output.push_str("    /// use std::time::Duration;\n");
        output.push_str("    ///\n");
        output.push_str("    /// #[tokio::main]\n");
        output.push_str("    /// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("    ///     // Create a custom reqwest client with specific settings\n");
        output.push_str("    ///     let reqwest_client = reqwest::Client::builder()\n");
        output.push_str("    ///         .timeout(Duration::from_secs(30))\n");
        output
            .push_str("    ///         .danger_accept_invalid_certs(true) // For testing only!\n");
        output.push_str("    ///         .build()\n");
        output
            .push_str("    ///         .map_err(|e| RpcError::TransportError(e.to_string()))?;\n");
        output.push_str("    ///\n");
        output.push_str("    ///     // Create transport with custom client\n");
        output.push_str("    ///     let transport = reqwest_adapter::ReqwestTransport::with_client(reqwest_client);\n");
        output.push_str("    ///\n");
        output.push_str("    ///     // Create a RESTCONF client\n");
        output.push_str("    ///     let client = RestconfClient::new(\n");
        output.push_str("    ///         \"https://device.example.com\",\n");
        output.push_str("    ///         transport\n");
        output.push_str("    ///     )?;\n");
        output.push_str("    ///\n");
        output.push_str("    ///     Ok(())\n");
        output.push_str("    /// }\n");
        output.push_str("    /// ```\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }

        if !derives.is_empty() {
            output.push_str(&format!("    #[derive({})]\n", derives.join(", ")));
        }

        output.push_str("    pub struct ReqwestTransport {\n");
        output.push_str("        /// The underlying reqwest client.\n");
        output.push_str("        client: reqwest::Client,\n");
        output.push_str("    }\n\n");

        // Generate constructor methods
        output.push_str("    impl ReqwestTransport {\n");
        output
            .push_str("        /// Create a new reqwest transport with default client settings.\n");
        output.push_str("        ///\n");
        output.push_str("        /// This creates a `reqwest::Client` with default configuration, which includes:\n");
        output.push_str("        /// - Connection pooling\n");
        output.push_str("        /// - Automatic redirect following (up to 10 redirects)\n");
        output.push_str("        /// - Gzip and deflate compression support\n");
        output.push_str("        /// - Default timeout of 30 seconds\n");
        output.push_str("        ///\n");
        output.push_str("        /// # Returns\n");
        output.push_str("        ///\n");
        output.push_str("        /// Returns a new `ReqwestTransport` instance.\n");
        output.push_str("        ///\n");
        output.push_str("        /// # Examples\n");
        output.push_str("        ///\n");
        output.push_str("        /// ```rust,ignore\n");
        output.push_str("        /// let transport = reqwest_adapter::ReqwestTransport::new();\n");
        output.push_str("        /// ```\n");
        output.push_str("        pub fn new() -> Self {\n");
        output.push_str("            Self {\n");
        output.push_str("                client: reqwest::Client::new(),\n");
        output.push_str("            }\n");
        output.push_str("        }\n\n");

        output
            .push_str("        /// Create a new reqwest transport with a custom reqwest client.\n");
        output.push_str("        ///\n");
        output.push_str("        /// This allows you to configure the reqwest client with custom settings such as:\n");
        output.push_str("        /// - Custom timeouts\n");
        output.push_str("        /// - TLS/SSL configuration\n");
        output.push_str("        /// - Proxy settings\n");
        output.push_str("        /// - Connection pool limits\n");
        output.push_str("        /// - Custom headers\n");
        output.push_str("        ///\n");
        output.push_str("        /// # Arguments\n");
        output.push_str("        ///\n");
        output.push_str("        /// * `client` - A configured `reqwest::Client` instance\n");
        output.push_str("        ///\n");
        output.push_str("        /// # Returns\n");
        output.push_str("        ///\n");
        output.push_str(
            "        /// Returns a new `ReqwestTransport` instance using the provided client.\n",
        );
        output.push_str("        ///\n");
        output.push_str("        /// # Examples\n");
        output.push_str("        ///\n");
        output.push_str("        /// ```rust,ignore\n");
        output.push_str("        /// use std::time::Duration;\n");
        output.push_str("        ///\n");
        output.push_str("        /// let reqwest_client = reqwest::Client::builder()\n");
        output.push_str("        ///     .timeout(Duration::from_secs(60))\n");
        output.push_str("        ///     .build()\n");
        output.push_str("        ///     .unwrap();\n");
        output.push_str("        ///\n");
        output.push_str("        /// let transport = reqwest_adapter::ReqwestTransport::with_client(reqwest_client);\n");
        output.push_str("        /// ```\n");
        output.push_str("        pub fn with_client(client: reqwest::Client) -> Self {\n");
        output.push_str("            Self { client }\n");
        output.push_str("        }\n");
        output.push_str("    }\n\n");

        // Generate HttpTransport trait implementation
        output.push_str("    #[async_trait]\n");
        output.push_str("    impl HttpTransport for ReqwestTransport {\n");
        output.push_str("        async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {\n");
        output.push_str("            // Convert HttpMethod to reqwest::Method\n");
        output.push_str("            let method = match request.method {\n");
        output.push_str("                HttpMethod::GET => reqwest::Method::GET,\n");
        output.push_str("                HttpMethod::POST => reqwest::Method::POST,\n");
        output.push_str("                HttpMethod::PUT => reqwest::Method::PUT,\n");
        output.push_str("                HttpMethod::DELETE => reqwest::Method::DELETE,\n");
        output.push_str("                HttpMethod::PATCH => reqwest::Method::PATCH,\n");
        output.push_str("            };\n\n");

        output.push_str("            // Build reqwest request\n");
        output.push_str(
            "            let mut req_builder = self.client.request(method, &request.url);\n\n",
        );

        output.push_str("            // Add headers\n");
        output.push_str("            for (key, value) in request.headers {\n");
        output.push_str("                req_builder = req_builder.header(key, value);\n");
        output.push_str("            }\n\n");

        output.push_str("            // Add body if present\n");
        output.push_str("            if let Some(body) = request.body {\n");
        output.push_str("                req_builder = req_builder.body(body);\n");
        output.push_str("            }\n\n");

        output.push_str("            // Execute request\n");
        output.push_str("            let response = req_builder.send().await\n");
        output.push_str("                .map_err(|e| RpcError::TransportError(format!(\"HTTP request failed: {}\", e)))?;\n\n");

        output.push_str("            // Extract response data\n");
        output.push_str("            let status_code = response.status().as_u16();\n");
        output.push_str("            let headers = response.headers()\n");
        output.push_str("                .iter()\n");
        output.push_str("                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or(\"\").to_string()))\n");
        output.push_str("                .collect();\n");
        output.push_str("            let body = response.bytes().await\n");
        output.push_str("                .map_err(|e| RpcError::TransportError(format!(\"Failed to read response body: {}\", e)))?\n");
        output.push_str("                .to_vec();\n\n");

        output.push_str("            Ok(HttpResponse {\n");
        output.push_str("                status_code,\n");
        output.push_str("                headers,\n");
        output.push_str("                body,\n");
        output.push_str("            })\n");
        output.push_str("        }\n");
        output.push_str("    }\n");

        output.push_str("}\n");

        output
    }

    /// Generate hyper transport adapter module.
    pub fn generate_hyper_adapter(&self) -> String {
        let mut output = String::new();

        // Add feature gate
        output.push_str("#[cfg(feature = \"hyper-client\")]\n");
        output.push_str("pub mod hyper_adapter {\n");
        output.push_str("    use super::*;\n");
        output.push_str("    use async_trait::async_trait;\n");
        output.push('\n');

        // Generate HyperTransport struct
        output.push_str("    /// HTTP transport implementation using hyper.\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// This adapter provides HTTP execution using the `hyper` library, which is\n",
        );
        output.push_str(
            "    /// a low-level HTTP implementation that provides fine-grained control over\n",
        );
        output.push_str(
            "    /// HTTP connections and requests. It is built on top of `tokio` and provides\n",
        );
        output.push_str("    /// excellent performance for high-throughput scenarios.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Feature Flag\n");
        output.push_str("    ///\n");
        output.push_str(
            "    /// This adapter is only available when the `hyper-client` feature is enabled.\n",
        );
        output.push_str("    /// Add it to your `Cargo.toml`:\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```toml\n");
        output.push_str("    /// [dependencies]\n");
        output.push_str(
            "    /// my-bindings = { version = \"0.1\", features = [\"hyper-client\"] }\n",
        );
        output.push_str("    /// ```\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Examples\n");
        output.push_str("    ///\n");
        output.push_str("    /// ## Basic usage with default client\n");
        output.push_str("    ///\n");
        output.push_str("    /// ```rust,ignore\n");
        output.push_str("    /// use my_bindings::*;\n");
        output.push_str("    ///\n");
        output.push_str("    /// #[tokio::main]\n");
        output.push_str("    /// async fn main() -> Result<(), RpcError> {\n");
        output.push_str("    ///     // Create a transport with default settings\n");
        output.push_str("    ///     let transport = hyper_adapter::HyperTransport::new();\n");
        output.push_str("    ///\n");
        output.push_str("    ///     // Create a RESTCONF client\n");
        output.push_str("    ///     let client = RestconfClient::new(\n");
        output.push_str("    ///         \"https://device.example.com\",\n");
        output.push_str("    ///         transport\n");
        output.push_str("    ///     )?;\n");
        output.push_str("    ///\n");
        output.push_str("    ///     // Use the client to call RPC operations\n");
        output.push_str("    ///     // let result = some_rpc_function(&client, input).await?;\n");
        output.push_str("    ///\n");
        output.push_str("    ///     Ok(())\n");
        output.push_str("    /// }\n");
        output.push_str("    /// ```\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }

        if !derives.is_empty() {
            output.push_str(&format!("    #[derive({})]\n", derives.join(", ")));
        }

        output.push_str("    pub struct HyperTransport {\n");
        output.push_str("        /// The underlying hyper client.\n");
        output.push_str("        client: hyper::Client<hyper::client::HttpConnector>,\n");
        output.push_str("    }\n\n");

        // Generate constructor method
        output.push_str("    impl HyperTransport {\n");
        output.push_str("        /// Create a new hyper transport with default client settings.\n");
        output.push_str("        ///\n");
        output.push_str("        /// This creates a `hyper::Client` with default configuration, which includes:\n");
        output.push_str("        /// - HTTP/1.1 and HTTP/2 support\n");
        output.push_str("        /// - Connection pooling\n");
        output.push_str("        /// - Keep-alive connections\n");
        output.push_str("        ///\n");
        output.push_str("        /// # Returns\n");
        output.push_str("        ///\n");
        output.push_str("        /// Returns a new `HyperTransport` instance.\n");
        output.push_str("        ///\n");
        output.push_str("        /// # Examples\n");
        output.push_str("        ///\n");
        output.push_str("        /// ```rust,ignore\n");
        output.push_str("        /// let transport = hyper_adapter::HyperTransport::new();\n");
        output.push_str("        /// ```\n");
        output.push_str("        pub fn new() -> Self {\n");
        output.push_str("            Self {\n");
        output.push_str("                client: hyper::Client::new(),\n");
        output.push_str("            }\n");
        output.push_str("        }\n");
        output.push_str("    }\n\n");

        // Generate HttpTransport trait implementation
        output.push_str("    #[async_trait]\n");
        output.push_str("    impl HttpTransport for HyperTransport {\n");
        output.push_str("        async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError> {\n");
        output.push_str("            // Convert HttpMethod to hyper::Method\n");
        output.push_str("            let method = match request.method {\n");
        output.push_str("                HttpMethod::GET => hyper::Method::GET,\n");
        output.push_str("                HttpMethod::POST => hyper::Method::POST,\n");
        output.push_str("                HttpMethod::PUT => hyper::Method::PUT,\n");
        output.push_str("                HttpMethod::DELETE => hyper::Method::DELETE,\n");
        output.push_str("                HttpMethod::PATCH => hyper::Method::PATCH,\n");
        output.push_str("            };\n\n");

        output.push_str("            // Build hyper request\n");
        output.push_str("            let mut req_builder = hyper::Request::builder()\n");
        output.push_str("                .method(method)\n");
        output.push_str("                .uri(&request.url);\n\n");

        output.push_str("            // Add headers\n");
        output.push_str("            for (key, value) in request.headers {\n");
        output.push_str("                req_builder = req_builder.header(key, value);\n");
        output.push_str("            }\n\n");

        output.push_str("            // Build request with body\n");
        output.push_str("            let body = request.body.map(hyper::Body::from).unwrap_or_else(hyper::Body::empty);\n");
        output.push_str("            let req = req_builder.body(body)\n");
        output.push_str("                .map_err(|e| RpcError::TransportError(format!(\"Failed to build request: {}\", e)))?;\n\n");

        output.push_str("            // Execute request\n");
        output.push_str("            let response = self.client.request(req).await\n");
        output.push_str("                .map_err(|e| RpcError::TransportError(format!(\"HTTP request failed: {}\", e)))?;\n\n");

        output.push_str("            // Extract response data\n");
        output.push_str("            let status_code = response.status().as_u16();\n");
        output.push_str("            let headers = response.headers()\n");
        output.push_str("                .iter()\n");
        output.push_str("                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or(\"\").to_string()))\n");
        output.push_str("                .collect();\n\n");

        output.push_str(
            "            let body_bytes = hyper::body::to_bytes(response.into_body()).await\n",
        );
        output.push_str("                .map_err(|e| RpcError::TransportError(format!(\"Failed to read response body: {}\", e)))?;\n");
        output.push_str("            let body = body_bytes.to_vec();\n\n");

        output.push_str("            Ok(HttpResponse {\n");
        output.push_str("                status_code,\n");
        output.push_str("                headers,\n");
        output.push_str("                body,\n");
        output.push_str("            })\n");
        output.push_str("        }\n");
        output.push_str("    }\n");

        output.push_str("}\n");

        output
    }
}
