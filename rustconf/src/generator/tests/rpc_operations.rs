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
    assert!(content.contains("/// Returns an error if the RPC operation fails."));
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
