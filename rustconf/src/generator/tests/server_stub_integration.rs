//! Integration tests for server stub handler generation.

use crate::generator::server_handlers::ServerHandlerGenerator;
use crate::generator::server_stubs::StubHandlerGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{Container, DataNode, Leaf, Rpc, TypeSpec, YangModule};

#[test]
fn test_stub_implements_handler_trait() {
    let config = GeneratorConfig::default();
    let handler_gen = ServerHandlerGenerator::new(&config);
    let stub_gen = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "device-management".to_string(),
        namespace: "http://example.com/device-management".to_string(),
        prefix: "dm".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "interfaces".to_string(),
            description: Some("Network interfaces".to_string()),
            config: true,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "enabled".to_string(),
                description: Some("Whether interfaces are enabled".to_string()),
                type_spec: TypeSpec::Boolean,
                mandatory: false,
                default: None,
                config: true,
            })],
        })],
        rpcs: vec![Rpc {
            name: "restart-device".to_string(),
            description: Some("Restart the device".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "delay-seconds".to_string(),
                description: Some("Delay before restart in seconds".to_string()),
                type_spec: TypeSpec::Uint32 { range: None },
                mandatory: false,
                default: Some("0".to_string()),
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "success".to_string(),
                description: Some("Whether the restart was initiated successfully".to_string()),
                type_spec: TypeSpec::Boolean,
                mandatory: false,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    // Generate handler trait
    let handler_result = handler_gen.generate_handler_trait(&module);
    assert!(handler_result.is_ok());
    let handler_code = handler_result.unwrap();

    // Generate stub implementation
    let stub_result = stub_gen.generate_stub_impl(&module);
    assert!(stub_result.is_ok());
    let stub_code = stub_result.unwrap();

    // Verify trait and implementation match
    assert!(handler_code.contains("pub trait DeviceManagementHandler"));
    assert!(stub_code.contains("impl DeviceManagementHandler for StubDeviceManagementHandler"));

    // Verify all handler methods have stub implementations
    assert!(handler_code.contains("async fn restart_device"));
    assert!(stub_code.contains("async fn restart_device"));

    assert!(handler_code.contains("async fn get_interfaces"));
    assert!(stub_code.contains("async fn get_interfaces"));

    assert!(handler_code.contains("async fn put_interfaces"));
    assert!(stub_code.contains("async fn put_interfaces"));

    assert!(handler_code.contains("async fn patch_interfaces"));
    assert!(stub_code.contains("async fn patch_interfaces"));

    assert!(handler_code.contains("async fn delete_interfaces"));
    assert!(stub_code.contains("async fn delete_interfaces"));
}

#[test]
fn test_stub_call_logging_format() {
    let config = GeneratorConfig::default();
    let stub_gen = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![
            Rpc {
                name: "no-input-no-output".to_string(),
                description: None,
                input: None,
                output: None,
            },
            Rpc {
                name: "with-input".to_string(),
                description: None,
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
                output: None,
            },
        ],
        notifications: vec![],
    };

    let result = stub_gen.generate_stub_impl(&module);
    assert!(result.is_ok());
    let stub_code = result.unwrap();

    // Verify logging format for RPC without input
    assert!(stub_code.contains("\"no_input_no_output()\""));

    // Verify logging format for RPC with input
    assert!(stub_code.contains("format!(\"with_input({:?})\", input)"));
}

#[test]
fn test_stub_default_values_are_valid() {
    let config = GeneratorConfig::default();
    let stub_gen = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-defaults".to_string(),
            description: None,
            input: None,
            output: Some(vec![
                DataNode::Leaf(Leaf {
                    name: "int-field".to_string(),
                    description: None,
                    type_spec: TypeSpec::Int32 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "string-field".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "bool-field".to_string(),
                    description: None,
                    type_spec: TypeSpec::Boolean,
                    mandatory: true,
                    default: None,
                    config: false,
                }),
            ]),
        }],
        notifications: vec![],
    };

    let result = stub_gen.generate_stub_impl(&module);
    assert!(result.is_ok());
    let stub_code = result.unwrap();

    // Verify default values are syntactically correct
    assert!(stub_code.contains("int_field: 0i32"));
    assert!(stub_code.contains("string_field: String::new()"));
    assert!(stub_code.contains("bool_field: false"));

    // Verify the code structure is valid
    assert!(stub_code.contains("Ok(TestDefaultsOutput {"));
}
