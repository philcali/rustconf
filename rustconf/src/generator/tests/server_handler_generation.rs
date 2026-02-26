//! Tests for server handler trait generation.

use crate::generator::server_handlers::ServerHandlerGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{Container, DataNode, Leaf, List, Rpc, TypeSpec, YangModule};

#[test]
fn test_generate_handler_trait_with_rpcs() {
    let config = GeneratorConfig::default();
    let generator = ServerHandlerGenerator::new(&config);

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

    let result = generator.generate_handler_trait(&module);
    assert!(result.is_ok());

    let trait_code = result.unwrap();

    // Check trait definition
    assert!(trait_code.contains("pub trait DeviceManagementHandler"));
    assert!(trait_code.contains("#[async_trait]"));
    assert!(trait_code.contains("Send + Sync"));

    // Check RPC with input and output
    assert!(trait_code.contains("async fn restart_device"));
    assert!(trait_code.contains("input: RestartDeviceInput"));
    assert!(trait_code.contains("Result<RestartDeviceOutput, ServerError>"));

    // Check RPC with only output
    assert!(trait_code.contains("async fn get_system_info"));
    assert!(trait_code.contains("Result<GetSystemInfoOutput, ServerError>"));
}

#[test]
fn test_generate_handler_trait_with_container() {
    let config = GeneratorConfig::default();
    let generator = ServerHandlerGenerator::new(&config);

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
            children: vec![],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_handler_trait(&module);
    assert!(result.is_ok());

    let trait_code = result.unwrap();

    // Should have GET, PUT, PATCH, DELETE for config=true
    assert!(trait_code.contains("async fn get_interfaces"));
    assert!(trait_code.contains("Result<Interfaces, ServerError>"));
    assert!(trait_code.contains("async fn put_interfaces"));
    assert!(trait_code.contains("data: Interfaces"));
    assert!(trait_code.contains("async fn patch_interfaces"));
    assert!(trait_code.contains("async fn delete_interfaces"));
}

#[test]
fn test_generate_handler_trait_with_container_config_false() {
    let config = GeneratorConfig::default();
    let generator = ServerHandlerGenerator::new(&config);

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

    let result = generator.generate_handler_trait(&module);
    assert!(result.is_ok());

    let trait_code = result.unwrap();

    // Should only have GET for config=false
    assert!(trait_code.contains("async fn get_system_state"));
    assert!(!trait_code.contains("async fn put_system_state"));
    assert!(!trait_code.contains("async fn patch_system_state"));
    assert!(!trait_code.contains("async fn delete_system_state"));
}

#[test]
fn test_generate_handler_trait_with_list() {
    let config = GeneratorConfig::default();
    let generator = ServerHandlerGenerator::new(&config);

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

    let result = generator.generate_handler_trait(&module);
    assert!(result.is_ok());

    let trait_code = result.unwrap();

    // Should have collection GET, item GET, POST, PUT, PATCH, DELETE
    assert!(trait_code.contains("async fn get_interfaces"));
    assert!(trait_code.contains("Result<Vec<Interface>, ServerError>"));
    assert!(trait_code.contains("async fn get_interfaces_by_key"));
    assert!(trait_code.contains("name: String"));
    assert!(trait_code.contains("Result<Interface, ServerError>"));
    assert!(trait_code.contains("async fn create_interfaces"));
    assert!(trait_code.contains("async fn put_interfaces"));
    assert!(trait_code.contains("async fn patch_interfaces"));
    assert!(trait_code.contains("async fn delete_interfaces"));
}

#[test]
fn test_generate_handler_trait_with_list_config_false() {
    let config = GeneratorConfig::default();
    let generator = ServerHandlerGenerator::new(&config);

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

    let result = generator.generate_handler_trait(&module);
    assert!(result.is_ok());

    let trait_code = result.unwrap();

    // Should only have GET methods for config=false
    assert!(trait_code.contains("async fn get_system_processes"));
    assert!(trait_code.contains("async fn get_system_processes_by_key"));
    assert!(!trait_code.contains("async fn create_system_processes"));
    assert!(!trait_code.contains("async fn put_system_processes"));
    assert!(!trait_code.contains("async fn patch_system_processes"));
    assert!(!trait_code.contains("async fn delete_system_processes"));
}

#[test]
fn test_generate_full_handler_trait() {
    let config = GeneratorConfig::default();
    let generator = ServerHandlerGenerator::new(&config);

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
            children: vec![],
        })],
        rpcs: vec![Rpc {
            name: "restart-device".to_string(),
            description: Some("Restart the device".to_string()),
            input: Some(vec![]),
            output: Some(vec![]),
        }],
        notifications: vec![],
    };

    let result = generator.generate_handler_trait(&module);
    assert!(result.is_ok());

    let trait_code = result.unwrap();
    assert!(trait_code.contains("pub trait DeviceManagementHandler"));
    assert!(trait_code.contains("async fn restart_device"));
    assert!(trait_code.contains("async fn get_interfaces"));
    assert!(trait_code.contains("#[async_trait]"));
}
