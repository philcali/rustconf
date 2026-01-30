//! Unit tests for RPC generation (Task 10.1)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};

#[test]
fn test_generate_rpc_with_no_input_or_output() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "reset-system".to_string(),
            description: Some("Reset the system to default state".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check RPC error type is generated
    assert!(content.contains("pub enum RpcError {"));
    assert!(content.contains("NetworkError(String)"));
    assert!(content.contains("ServerError { code: u16, message: String }"));

    // Check operations module
    assert!(content.contains("pub mod operations {"));

    // Check function signature
    assert!(content.contains("pub async fn reset_system() -> Result<(), RpcError>"));

    // Check rustdoc comment
    assert!(content.contains("/// Reset the system to default state"));

    // Check error documentation
    assert!(content.contains("/// # Errors"));
    assert!(content
        .contains("/// Returns `RpcError::NotImplemented` as RESTful RPC generation is disabled."));
}

#[test]
fn test_generate_rpc_with_input_only() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "set-config".to_string(),
            description: Some("Set configuration parameters".to_string()),
            input: Some(vec![
                DataNode::Leaf(Leaf {
                    name: "config-name".to_string(),
                    description: Some("Name of the configuration".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "value".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
            ]),
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check input type is generated
    assert!(content.contains("pub struct SetConfigInput {"));
    assert!(content.contains("/// Input parameters for set-config RPC"));

    // Check input fields
    assert!(content.contains("pub config_name: String,"));
    assert!(content.contains("pub value: String,"));
    assert!(content.contains("/// Name of the configuration"));

    // Check function signature with input parameter
    assert!(
        content.contains("pub async fn set_config(input: SetConfigInput) -> Result<(), RpcError>")
    );
}

#[test]
fn test_generate_rpc_with_output_only() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "get-statistics".to_string(),
            description: Some("Retrieve system statistics".to_string()),
            input: None,
            output: Some(vec![
                DataNode::Leaf(Leaf {
                    name: "uptime".to_string(),
                    description: Some("System uptime in seconds".to_string()),
                    type_spec: TypeSpec::Uint64 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "cpu-usage".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint8 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
            ]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check output type is generated
    assert!(content.contains("pub struct GetStatisticsOutput {"));
    assert!(content.contains("/// Output result for get-statistics RPC"));

    // Check output fields
    assert!(content.contains("pub uptime: u64,"));
    assert!(content.contains("pub cpu_usage: u8,"));
    assert!(content.contains("/// System uptime in seconds"));

    // Check function signature with output type
    assert!(
        content.contains("pub async fn get_statistics() -> Result<GetStatisticsOutput, RpcError>")
    );
}

#[test]
fn test_generate_rpc_with_input_and_output() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "calculate-sum".to_string(),
            description: Some("Calculate the sum of two numbers".to_string()),
            input: Some(vec![
                DataNode::Leaf(Leaf {
                    name: "a".to_string(),
                    description: Some("First number".to_string()),
                    type_spec: TypeSpec::Int32 { range: None },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "b".to_string(),
                    description: Some("Second number".to_string()),
                    type_spec: TypeSpec::Int32 { range: None },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
            ]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: Some("Sum of a and b".to_string()),
                type_spec: TypeSpec::Int32 { range: None },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check input type
    assert!(content.contains("pub struct CalculateSumInput {"));
    assert!(content.contains("pub a: i32,"));
    assert!(content.contains("pub b: i32,"));
    assert!(content.contains("/// First number"));
    assert!(content.contains("/// Second number"));

    // Check output type
    assert!(content.contains("pub struct CalculateSumOutput {"));
    assert!(content.contains("pub result: i32,"));
    assert!(content.contains("/// Sum of a and b"));

    // Check function signature
    assert!(content.contains("pub async fn calculate_sum(input: CalculateSumInput) -> Result<CalculateSumOutput, RpcError>"));

    // Check rustdoc
    assert!(content.contains("/// Calculate the sum of two numbers"));
}

#[test]
fn test_generate_multiple_rpcs() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![
            Rpc {
                name: "start-service".to_string(),
                description: Some("Start a service".to_string()),
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "service-name".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                })]),
                output: None,
            },
            Rpc {
                name: "stop-service".to_string(),
                description: Some("Stop a service".to_string()),
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "service-name".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                })]),
                output: None,
            },
        ],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check both RPCs are generated
    assert!(content.contains("pub struct StartServiceInput {"));
    assert!(content
        .contains("pub async fn start_service(input: StartServiceInput) -> Result<(), RpcError>"));
    assert!(content.contains("/// Start a service"));

    assert!(content.contains("pub struct StopServiceInput {"));
    assert!(content
        .contains("pub async fn stop_service(input: StopServiceInput) -> Result<(), RpcError>"));
    assert!(content.contains("/// Stop a service"));
}

#[test]
fn test_rpc_error_type_generation() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check RpcError enum
    assert!(content.contains("pub enum RpcError {"));
    assert!(content.contains("NetworkError(String)"));
    assert!(content.contains("ServerError { code: u16, message: String }"));
    assert!(content.contains("SerializationError(String)"));
    assert!(content.contains("InvalidInput(String)"));
    assert!(content.contains("NotImplemented"));

    // Check Display implementation
    assert!(content.contains("impl std::fmt::Display for RpcError {"));
    assert!(content.contains(r#"write!(f, "Network error: {}", msg)"#));
    assert!(content.contains(r#"write!(f, "Server error {}: {}", code, message)"#));

    // Check Error trait implementation
    assert!(content.contains("impl std::error::Error for RpcError {}"));
}

#[test]
fn test_rpc_not_generated_when_no_rpcs() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that RPC-related code is not generated
    assert!(!content.contains("pub enum RpcError"));
    assert!(!content.contains("pub mod operations"));
}

#[test]
fn test_rpc_with_empty_input_list() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "ping".to_string(),
            description: Some("Ping the system".to_string()),
            input: Some(vec![]),
            output: Some(vec![]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Empty input/output should be treated as no input/output
    assert!(!content.contains("pub struct PingInput"));
    assert!(!content.contains("pub struct PingOutput"));

    // Function should have no parameters and return unit
    assert!(content.contains("pub async fn ping() -> Result<(), RpcError>"));
}

#[test]
fn test_http_method_generated_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpMethod enum is generated
    assert!(content.contains("pub enum HttpMethod {"));
    assert!(content.contains("/// HTTP GET method."));
    assert!(content.contains("GET,"));
    assert!(content.contains("/// HTTP POST method."));
    assert!(content.contains("POST,"));
    assert!(content.contains("/// HTTP PUT method."));
    assert!(content.contains("PUT,"));
    assert!(content.contains("/// HTTP DELETE method."));
    assert!(content.contains("DELETE,"));
    assert!(content.contains("/// HTTP PATCH method."));
    assert!(content.contains("PATCH,"));

    // Check that HttpMethod has proper derives
    assert!(content.contains("HTTP methods for RESTful operations."));
}

#[test]
fn test_http_method_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpMethod enum is NOT generated
    assert!(!content.contains("pub enum HttpMethod {"));
    assert!(!content.contains("HTTP methods for RESTful operations."));
}

#[test]
fn test_http_request_generated_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpRequest struct is generated
    assert!(content.contains("pub struct HttpRequest {"));
    assert!(content.contains("/// HTTP request for RESTful operations."));

    // Check all fields are public
    assert!(content.contains("pub method: HttpMethod,"));
    assert!(content.contains("pub url: String,"));
    assert!(content.contains("pub headers: Vec<(String, String)>,"));
    assert!(content.contains("pub body: Option<Vec<u8>>,"));

    // Check field documentation
    assert!(content.contains("/// The HTTP method for this request."));
    assert!(content.contains("/// The target URL for this request."));
    assert!(content.contains("/// HTTP headers as key-value pairs."));
    assert!(content.contains("/// Optional request body as raw bytes."));

    // Check struct documentation mentions custom transport access
    assert!(content.contains("All fields are public to allow"));
    assert!(content.contains("custom transport implementations to access request details."));
}

#[test]
fn test_http_request_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpRequest struct is NOT generated
    assert!(!content.contains("pub struct HttpRequest {"));
    assert!(!content.contains("HTTP request for RESTful operations."));
}

#[test]
fn test_http_response_generated_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpResponse struct is generated
    assert!(content.contains("pub struct HttpResponse {"));
    assert!(content.contains("/// HTTP response from RESTful operations."));

    // Check all fields are public
    assert!(content.contains("pub status_code: u16,"));
    assert!(content.contains("pub headers: Vec<(String, String)>,"));
    assert!(content.contains("pub body: Vec<u8>,"));

    // Check field documentation
    assert!(content.contains("/// The HTTP status code (e.g., 200, 404, 500)."));
    assert!(content.contains("/// HTTP headers as key-value pairs."));
    assert!(content.contains("/// Response body as raw bytes."));

    // Check struct documentation mentions custom transport access
    assert!(content.contains("All fields are public to allow custom transport implementations"));
}

#[test]
fn test_http_response_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpResponse struct is NOT generated
    assert!(!content.contains("pub struct HttpResponse {"));
    assert!(!content.contains("HTTP response from RESTful operations."));
}

#[test]
fn test_http_transport_generated_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpTransport trait is generated
    assert!(content.contains("pub trait HttpTransport: Send + Sync {"));
    assert!(content.contains("/// HTTP transport abstraction for executing RESTful operations."));

    // Check async_trait macro is used
    assert!(content.contains("#[async_trait::async_trait]"));

    // Check execute method signature
    assert!(content.contains(
        "async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>;"
    ));

    // Check comprehensive documentation
    assert!(content.contains("/// This trait provides a pluggable interface for HTTP execution"));
    assert!(content.contains("/// # Thread Safety"));
    assert!(content.contains("/// Implementations must be `Send + Sync`"));
    assert!(content.contains("/// # Examples"));
    assert!(content.contains("/// ## Using a built-in transport adapter"));
    assert!(content.contains("/// ## Implementing a custom transport"));
    assert!(content.contains("/// # Error Handling"));

    // Check example code in documentation
    assert!(content.contains("let transport = reqwest_adapter::ReqwestTransport::new();"));
    assert!(content.contains("let client = RestconfClient::new("));
    assert!(content.contains("struct MyCustomTransport {"));
    assert!(content.contains("impl HttpTransport for MyCustomTransport {"));

    // Check method documentation
    assert!(content.contains("/// Execute an HTTP request and return the response."));
    assert!(content.contains("/// # Arguments"));
    assert!(content.contains("/// * `request` - The HTTP request to execute"));
    assert!(content.contains("/// # Returns"));
    assert!(content.contains("/// # Errors"));
    assert!(content.contains("/// Returns `RpcError::TransportError` for:"));
    assert!(content.contains("/// - Network connectivity issues"));
    assert!(content.contains("/// - DNS resolution failures"));
    assert!(content.contains("/// - Connection timeouts"));
    assert!(content.contains("/// - TLS/SSL errors"));
}

#[test]
fn test_http_transport_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check HttpTransport trait is NOT generated
    assert!(!content.contains("pub trait HttpTransport"));
    assert!(!content.contains("HTTP transport abstraction for executing RESTful operations."));
    assert!(!content.contains("#[async_trait::async_trait]"));
}

#[test]
fn test_request_interceptor_generated_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check RequestInterceptor trait is generated
    assert!(content.contains("pub trait RequestInterceptor: Send + Sync {"));
    assert!(content.contains("/// Request interceptor for modifying HTTP requests and responses."));

    // Check async_trait macro is used
    assert!(content.contains("#[async_trait::async_trait]"));

    // Check before_request method signature
    assert!(content.contains(
        "async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError>;"
    ));

    // Check after_response method signature
    assert!(content.contains(
        "async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError>;"
    ));

    // Check comprehensive documentation
    assert!(content
        .contains("/// This trait provides hooks for intercepting and modifying HTTP requests"));
    assert!(content.contains("/// # Thread Safety"));
    assert!(content.contains("/// Implementations must be `Send + Sync`"));
    assert!(content.contains("/// # Execution Order"));
    assert!(content.contains("/// - `before_request` hooks are called in registration order"));
    assert!(
        content.contains("/// - `after_response` hooks are called in reverse registration order")
    );
    assert!(content.contains("/// # Examples"));
    assert!(content.contains("/// ## Basic authentication interceptor"));
    assert!(content.contains("/// ## Logging interceptor"));
    assert!(content.contains("/// # Error Handling"));

    // Check example code in documentation
    assert!(content.contains("struct AuthInterceptor {"));
    assert!(content.contains("token: String,"));
    assert!(content.contains("impl RequestInterceptor for AuthInterceptor {"));
    assert!(content.contains("request.headers.push(("));
    assert!(content.contains("\"Authorization\".to_string(),"));
    assert!(content.contains("format!(\"Bearer {}\", self.token)"));
    assert!(content.contains("struct LoggingInterceptor;"));
    assert!(content.contains(".with_interceptor(AuthInterceptor {"));

    // Check method documentation
    assert!(content.contains("/// Called before sending an HTTP request."));
    assert!(content.contains("/// This method receives a mutable reference to the `HttpRequest`"));
    assert!(content.contains("/// Called after receiving an HTTP response."));
    assert!(
        content.contains("/// This method receives an immutable reference to the `HttpResponse`")
    );
    assert!(content.contains("/// # Arguments"));
    assert!(content.contains("/// * `request` - A mutable reference to the HTTP request"));
    assert!(content.contains("/// * `response` - An immutable reference to the HTTP response"));
    assert!(content.contains("/// # Returns"));
    assert!(content.contains("/// # Errors"));
    assert!(content.contains("/// - Authentication token is missing or expired"));
    assert!(content.contains("/// - Response validation fails"));
}

#[test]
fn test_request_interceptor_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check RequestInterceptor trait is NOT generated
    assert!(!content.contains("pub trait RequestInterceptor"));
    assert!(!content.contains("Request interceptor for modifying HTTP requests and responses."));
    assert!(!content.contains("async fn before_request"));
    assert!(!content.contains("async fn after_response"));
}

#[test]
fn test_stub_rpc_function_when_restful_rpcs_disabled() {
    // Test that when enable_restful_rpcs is false, stub functions are generated
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-operation".to_string(),
            description: Some("Test RPC operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "param".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that stub function is generated
    assert!(
        content.contains("pub async fn test_operation(input: TestOperationInput) -> Result<TestOperationOutput, RpcError>"),
        "Stub function signature should be generated"
    );

    // Check that stub returns NotImplemented
    assert!(
        content.contains("Err(RpcError::NotImplemented)"),
        "Stub function should return NotImplemented error"
    );

    // Check that RestconfClient is NOT used in the signature
    assert!(
        !content.contains("client: &RestconfClient"),
        "Stub function should not have client parameter"
    );
}

#[test]
fn test_restful_rpc_function_when_restful_rpcs_enabled() {
    // Test that when enable_restful_rpcs is true, RESTful functions are generated
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-operation".to_string(),
            description: Some("Test RPC operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "param".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that RESTful function is generated with client parameter
    assert!(
        content.contains("pub async fn test_operation<T: HttpTransport>(client: &RestconfClient<T>, input: TestOperationInput) -> Result<TestOperationOutput, RpcError>"),
        "RESTful function signature should include client parameter and generic type"
    );

    // Check that stub NotImplemented is NOT used
    assert!(
        !content.contains("Err(RpcError::NotImplemented)"),
        "RESTful function should not return NotImplemented error"
    );

    // Check that RestconfClient is used in the signature
    assert!(
        content.contains("client: &RestconfClient"),
        "RESTful function should have client parameter"
    );
}

#[test]
fn test_restful_rpc_function_without_input() {
    // Test RESTful function generation for RPC with no input
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "ping".to_string(),
            description: Some("Ping operation".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that RESTful function is generated with only client parameter
    assert!(
        content.contains("pub async fn ping<T: HttpTransport>(client: &RestconfClient<T>) -> Result<(), RpcError>"),
        "RESTful function without input should only have client parameter"
    );
}

#[test]
fn test_stub_rpc_function_without_input() {
    // Test stub function generation for RPC with no input
    let config = GeneratorConfig::default(); // enable_restful_rpcs is false by default
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "ping".to_string(),
            description: Some("Ping operation".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that stub function is generated without any parameters
    assert!(
        content.contains("pub async fn ping() -> Result<(), RpcError>"),
        "Stub function without input should have no parameters"
    );

    // Check that stub returns NotImplemented
    assert!(
        content.contains("Err(RpcError::NotImplemented)"),
        "Stub function should return NotImplemented error"
    );
}

#[test]
fn test_rpc_documentation_with_restful_enabled() {
    // Test that comprehensive documentation is generated for RESTful RPCs
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "interface-mgmt".to_string(),
        namespace: "urn:interface-mgmt".to_string(),
        prefix: "if".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "reset-interface".to_string(),
            description: Some("Reset a network interface to its default state".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "interface-name".to_string(),
                description: Some("Name of the interface to reset".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "status".to_string(),
                description: Some("Status of the reset operation".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that YANG description is included
    assert!(
        content.contains("/// Reset a network interface to its default state"),
        "Should include RPC description from YANG"
    );

    // Check that parameters are documented
    assert!(
        content.contains("/// # Arguments"),
        "Should have Arguments section"
    );
    assert!(
        content
            .contains("/// * `client` - The RestconfClient to use for executing the RPC request"),
        "Should document client parameter"
    );
    assert!(
        content.contains("/// * `input` - The input parameters for the reset-interface operation"),
        "Should document input parameter"
    );

    // Check that return type is documented
    assert!(
        content.contains("/// # Returns"),
        "Should have Returns section"
    );
    assert!(
        content.contains(
            "/// Returns `Ok(ResetInterfaceOutput)` on success, containing the operation result."
        ),
        "Should document return type"
    );

    // Check that errors are documented
    assert!(
        content.contains("/// # Errors"),
        "Should have Errors section"
    );
    assert!(
        content.contains("/// Returns an error if:"),
        "Should list error conditions"
    );
    assert!(
        content.contains("/// - Input serialization fails (`RpcError::SerializationError`)"),
        "Should document serialization errors"
    );
    assert!(
        content.contains("/// - The HTTP request fails (`RpcError::TransportError`)"),
        "Should document transport errors"
    );
    assert!(
        content.contains("/// - The server returns an error status:"),
        "Should document HTTP status errors"
    );
    assert!(
        content.contains("///   - 400: `RpcError::InvalidInput`"),
        "Should document 400 error"
    );
    assert!(
        content.contains("///   - 401/403: `RpcError::Unauthorized`"),
        "Should document auth errors"
    );
    assert!(
        content.contains("///   - 404: `RpcError::NotFound`"),
        "Should document 404 error"
    );
    assert!(
        content.contains("///   - 500-599: `RpcError::ServerError`"),
        "Should document server errors"
    );
    assert!(
        content.contains("/// - Response deserialization fails (`RpcError::DeserializationError`)"),
        "Should document deserialization errors"
    );

    // Check that usage example is included
    assert!(
        content.contains("/// # Example"),
        "Should have Example section"
    );
    assert!(
        content.contains("/// ```rust,ignore"),
        "Should have example code block"
    );
    assert!(
        content.contains("/// use interface_mgmt::*;"),
        "Should show module import"
    );
    assert!(
        content.contains("/// #[tokio::main]"),
        "Should show async main"
    );
    assert!(
        content.contains("/// let transport = reqwest_adapter::ReqwestTransport::new();"),
        "Should show transport creation"
    );
    assert!(
        content.contains("/// let client = RestconfClient::new("),
        "Should show client creation"
    );
    assert!(
        content.contains("device.example.com"),
        "Should show example URL"
    );
    assert!(
        content.contains("Input {"),
        "Should show input type construction"
    );
    assert!(
        content.contains("reset_interface(&client, input).await?;"),
        "Should show function call"
    );
    assert!(
        content.contains("Operation completed successfully"),
        "Should show result processing"
    );
}
