//! Tests for server stub handler implementation generation.

use crate::generator::server_stubs::StubHandlerGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{Container, DataNode, Leaf, List, Rpc, TypeSpec, YangModule};

#[test]
fn test_generate_stub_impl_with_rpcs() {
    let config = GeneratorConfig::default();
    let generator = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "device-management".to_string(),
        namespace: "http://example.com/device-management".to_string(),
        prefix: "dm".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![
            Rpc {
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
            },
            Rpc {
                name: "get-system-info".to_string(),
                description: Some("Get system information".to_string()),
                input: None,
                output: Some(vec![DataNode::Leaf(Leaf {
                    name: "hostname".to_string(),
                    description: Some("Device hostname".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: false,
                    default: None,
                    config: false,
                })]),
            },
        ],
        notifications: vec![],
    };

    let result = generator.generate_stub_impl(&module);
    assert!(result.is_ok());

    let stub_code = result.unwrap();

    // Check struct definition
    assert!(stub_code.contains("pub struct StubDeviceManagementHandler"));
    assert!(stub_code.contains("call_log: Arc<Mutex<Vec<String>>>"));

    // Check constructor and accessor
    assert!(stub_code.contains("pub fn new() -> Self"));
    assert!(stub_code.contains("pub fn get_call_log(&self) -> Vec<String>"));

    // Check trait implementation
    assert!(stub_code.contains("#[async_trait]"));
    assert!(stub_code.contains("impl DeviceManagementHandler for StubDeviceManagementHandler"));

    // Check RPC stub methods
    assert!(stub_code.contains("async fn restart_device"));
    assert!(stub_code.contains("input: RestartDeviceInput"));
    assert!(stub_code.contains("Result<RestartDeviceOutput, ServerError>"));
    assert!(stub_code.contains("call_log.lock().unwrap().push"));

    assert!(stub_code.contains("async fn get_system_info"));
    assert!(stub_code.contains("Result<GetSystemInfoOutput, ServerError>"));
}

#[test]
fn test_generate_stub_impl_with_container() {
    let config = GeneratorConfig::default();
    let generator = StubHandlerGenerator::new(&config);

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
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_stub_impl(&module);
    assert!(result.is_ok());

    let stub_code = result.unwrap();

    // Should have GET, PUT, PATCH, DELETE for config=true
    assert!(stub_code.contains("async fn get_interfaces"));
    assert!(stub_code.contains("Result<Interfaces, ServerError>"));
    assert!(stub_code.contains("async fn put_interfaces"));
    assert!(stub_code.contains("data: Interfaces"));
    assert!(stub_code.contains("async fn patch_interfaces"));
    assert!(stub_code.contains("async fn delete_interfaces"));

    // Check call logging
    assert!(stub_code.contains("get_interfaces()"));
    assert!(stub_code.contains("put_interfaces({:?})"));
    assert!(stub_code.contains("patch_interfaces({:?})"));
    assert!(stub_code.contains("delete_interfaces()"));
}

#[test]
fn test_generate_stub_impl_with_list() {
    let config = GeneratorConfig::default();
    let generator = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "device-management".to_string(),
        namespace: "http://example.com/device-management".to_string(),
        prefix: "dm".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::List(List {
            name: "interfaces".to_string(),
            description: Some("Network interfaces list".to_string()),
            config: true,
            keys: vec!["name".to_string()],
            children: vec![DataNode::Leaf(Leaf {
                name: "name".to_string(),
                description: Some("Interface name".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_stub_impl(&module);
    assert!(result.is_ok());

    let stub_code = result.unwrap();

    // Should have collection GET, item GET, POST, PUT, PATCH, DELETE
    assert!(stub_code.contains("async fn get_interfaces"));
    assert!(stub_code.contains("Result<Vec<Interface>, ServerError>"));
    assert!(stub_code.contains("async fn get_interfaces_by_key"));
    assert!(stub_code.contains("name: String"));
    assert!(stub_code.contains("Result<Interface, ServerError>"));
    assert!(stub_code.contains("async fn create_interfaces"));
    assert!(stub_code.contains("async fn put_interfaces"));
    assert!(stub_code.contains("async fn patch_interfaces"));
    assert!(stub_code.contains("async fn delete_interfaces"));

    // Check call logging with key parameters
    assert!(stub_code.contains("get_interfaces_by_key({:?})"));
    assert!(stub_code.contains("put_interfaces({:?}, {:?})"));
}

#[test]
fn test_default_value_generation() {
    let config = GeneratorConfig::default();
    let generator = StubHandlerGenerator::new(&config);

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
            name: "test-types".to_string(),
            description: None,
            input: None,
            output: Some(vec![
                DataNode::Leaf(Leaf {
                    name: "int8-field".to_string(),
                    description: None,
                    type_spec: TypeSpec::Int8 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "uint32-field".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint32 { range: None },
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
                DataNode::Leaf(Leaf {
                    name: "optional-field".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: false,
                    default: None,
                    config: false,
                }),
            ]),
        }],
        notifications: vec![],
    };

    let result = generator.generate_stub_impl(&module);
    assert!(result.is_ok());

    let stub_code = result.unwrap();

    // Check default values for different types
    assert!(stub_code.contains("int8_field: 0i8"));
    assert!(stub_code.contains("uint32_field: 0u32"));
    assert!(stub_code.contains("string_field: String::new()"));
    assert!(stub_code.contains("bool_field: false"));
    assert!(stub_code.contains("optional_field: None"));
}

#[test]
fn test_stub_impl_with_config_false_container() {
    let config = GeneratorConfig::default();
    let generator = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "device-management".to_string(),
        namespace: "http://example.com/device-management".to_string(),
        prefix: "dm".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "system-state".to_string(),
            description: Some("System state information".to_string()),
            config: false,
            mandatory: false,
            children: vec![],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_stub_impl(&module);
    assert!(result.is_ok());

    let stub_code = result.unwrap();

    // Should only have GET for config=false
    assert!(stub_code.contains("async fn get_system_state"));
    assert!(!stub_code.contains("async fn put_system_state"));
    assert!(!stub_code.contains("async fn patch_system_state"));
    assert!(!stub_code.contains("async fn delete_system_state"));
}

#[test]
fn test_stub_impl_with_config_false_list() {
    let config = GeneratorConfig::default();
    let generator = StubHandlerGenerator::new(&config);

    let module = YangModule {
        name: "device-management".to_string(),
        namespace: "http://example.com/device-management".to_string(),
        prefix: "dm".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::List(List {
            name: "system-processes".to_string(),
            description: Some("Running system processes".to_string()),
            config: false,
            keys: vec!["pid".to_string()],
            children: vec![DataNode::Leaf(Leaf {
                name: "pid".to_string(),
                description: Some("Process ID".to_string()),
                type_spec: TypeSpec::Uint32 { range: None },
                mandatory: true,
                default: None,
                config: false,
            })],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_stub_impl(&module);
    assert!(result.is_ok());

    let stub_code = result.unwrap();

    // Should only have GET methods for config=false
    assert!(stub_code.contains("async fn get_system_processes"));
    assert!(stub_code.contains("async fn get_system_processes_by_key"));
    assert!(!stub_code.contains("async fn create_system_processes"));
    assert!(!stub_code.contains("async fn put_system_processes"));
    assert!(!stub_code.contains("async fn patch_system_processes"));
    assert!(!stub_code.contains("async fn delete_system_processes"));
}
