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

    // Check RPC error type is generated (matching rustconf-runtime variants)
    assert!(content.contains("pub enum RpcError {"));
    assert!(content.contains("TransportError(String)"));
    assert!(content.contains("HttpError { status_code: u16, message: String }"));

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

    // Check RpcError enum (matching rustconf-runtime variants)
    assert!(content.contains("pub enum RpcError {"));
    assert!(content.contains("TransportError(String)"));
    assert!(content.contains("HttpError { status_code: u16, message: String }"));
    assert!(content.contains("SerializationError(String)"));
    assert!(content.contains("ValidationError(String)"));
    assert!(content.contains("NotImplemented"));

    // Check Display implementation
    assert!(content.contains("impl std::fmt::Display for RpcError {"));
    assert!(content.contains(r#"write!(f, "Transport error: {}", msg)"#));
    assert!(content.contains(r#"write!(f, "HTTP error {}: {}", status_code, message)"#));

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

    // HttpMethod is now imported from rustconf-runtime, not generated
    assert!(
        content.contains("use rustconf_runtime::{"),
        "Should import from rustconf-runtime when RESTful RPCs are enabled"
    );
    assert!(
        content.contains("HttpMethod,"),
        "HttpMethod should be imported from rustconf-runtime"
    );

    // HttpMethod enum should NOT be generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub enum HttpMethod {"),
        "HttpMethod enum should not be generated (imported from rustconf-runtime)"
    );
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

    // HttpRequest is now imported from rustconf-runtime, not generated
    assert!(
        content.contains("use rustconf_runtime::{"),
        "Should import from rustconf-runtime when RESTful RPCs are enabled"
    );
    assert!(
        content.contains("HttpRequest,"),
        "HttpRequest should be imported from rustconf-runtime"
    );

    // HttpRequest struct should NOT be generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub struct HttpRequest {"),
        "HttpRequest struct should not be generated (imported from rustconf-runtime)"
    );
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

    // HttpResponse is now imported from rustconf-runtime, not generated
    assert!(
        content.contains("use rustconf_runtime::{"),
        "Should import from rustconf-runtime when RESTful RPCs are enabled"
    );
    assert!(
        content.contains("HttpResponse,"),
        "HttpResponse should be imported from rustconf-runtime"
    );

    // HttpResponse struct should NOT be generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub struct HttpResponse {"),
        "HttpResponse struct should not be generated (imported from rustconf-runtime)"
    );
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

    // HttpTransport is now imported from rustconf-runtime, not generated
    assert!(
        content.contains("use rustconf_runtime::{"),
        "Should import from rustconf-runtime when RESTful RPCs are enabled"
    );
    assert!(
        content.contains("HttpTransport,"),
        "HttpTransport should be imported from rustconf-runtime"
    );

    // HttpTransport trait should NOT be generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub trait HttpTransport: Send + Sync {"),
        "HttpTransport trait should not be generated (imported from rustconf-runtime)"
    );
}

#[test]
fn test_http_transport_custom_example_includes_complete_implementation() {
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

    // HttpTransport trait and its documentation are now in rustconf-runtime
    // The generated code just imports it
    assert!(
        content.contains("HttpTransport,"),
        "HttpTransport should be imported from rustconf-runtime"
    );

    // The custom implementation examples are in rustconf-runtime's documentation, not in generated code
    assert!(
        !content.contains("/// struct MyCustomTransport {"),
        "Custom transport examples should not be in generated code (they're in rustconf-runtime docs)"
    );
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

    // RequestInterceptor is now imported from rustconf-runtime, not generated
    assert!(
        content.contains("use rustconf_runtime::{"),
        "Should import from rustconf-runtime when RESTful RPCs are enabled"
    );
    assert!(
        content.contains("RequestInterceptor,"),
        "RequestInterceptor should be imported from rustconf-runtime"
    );

    // RequestInterceptor trait should NOT be generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub trait RequestInterceptor: Send + Sync {"),
        "RequestInterceptor trait should not be generated (imported from rustconf-runtime)"
    );
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
