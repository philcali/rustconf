//! Unit tests for RESTCONF CRUD operation generation (Task 10.3)

use crate::parser::{Container, DataNode, Leaf, List, TypeSpec, YangModule};

use super::*;

#[test]
fn test_generate_crud_for_config_container() {
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
        data_nodes: vec![DataNode::Container(Container {
            name: "system-config".to_string(),
            description: Some("System configuration container".to_string()),
            config: true,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "hostname".to_string(),
                description: None,
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

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check CRUD module exists
    assert!(content.contains("pub mod crud {"));

    // Check GET operation
    assert!(content.contains("pub async fn get_system_config() -> Result<SystemConfig, RpcError>"));
    assert!(content.contains("/// Retrieve the system-config container"));

    // Check PUT operation (config = true)
    assert!(content
        .contains("pub async fn put_system_config(data: SystemConfig) -> Result<(), RpcError>"));
    assert!(content.contains("/// Replace the system-config container"));

    // Check PATCH operation (config = true)
    assert!(content
        .contains("pub async fn patch_system_config(data: SystemConfig) -> Result<(), RpcError>"));
    assert!(content.contains("/// Partially update the system-config container"));

    // Check DELETE operation (config = true)
    assert!(content.contains("pub async fn delete_system_config() -> Result<(), RpcError>"));
    assert!(content.contains("/// Delete the system-config container"));

    // Check error documentation
    assert!(content.contains("/// # Errors"));
    assert!(content.contains("/// Returns an error if the operation fails"));
}

#[test]
fn test_generate_crud_for_state_container() {
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
        data_nodes: vec![DataNode::Container(Container {
            name: "system-state".to_string(),
            description: Some("System state container".to_string()),
            config: false,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "uptime".to_string(),
                description: None,
                type_spec: TypeSpec::Uint64 { range: None },
                mandatory: true,
                default: None,
                config: false,
            })],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check GET operation exists (always available)
    assert!(content.contains("pub async fn get_system_state() -> Result<SystemState, RpcError>"));

    // Check that write operations are NOT generated (config = false)
    assert!(!content.contains("pub async fn put_system_state"));
    assert!(!content.contains("pub async fn patch_system_state"));
    assert!(!content.contains("pub async fn delete_system_state"));
}

#[test]
fn test_generate_crud_for_list_with_single_key() {
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
        data_nodes: vec![DataNode::List(List {
            name: "interfaces".to_string(),
            description: Some("Network interfaces".to_string()),
            config: true,
            keys: vec!["name".to_string()],
            children: vec![
                DataNode::Leaf(Leaf {
                    name: "name".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "enabled".to_string(),
                    description: None,
                    type_spec: TypeSpec::Boolean,
                    mandatory: true,
                    default: None,
                    config: true,
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check GET operation for entire list
    assert!(content.contains("pub async fn get_interfaces() -> Result<Interfaces, RpcError>"));
    assert!(content.contains("/// Retrieve all interfaces items"));

    // Check GET operation for single item by key
    assert!(content.contains(
        "pub async fn get_interfaces_by_key(name: String) -> Result<Interface, RpcError>"
    ));
    assert!(content.contains("/// Retrieve a single interfaces item by key"));

    // Check POST operation (create)
    assert!(
        content.contains("pub async fn create_interfaces(data: Interface) -> Result<(), RpcError>")
    );
    assert!(content.contains("/// Create a new interfaces item"));

    // Check PUT operation (replace by key)
    assert!(content.contains(
        "pub async fn put_interfaces(name: String, data: Interface) -> Result<(), RpcError>"
    ));
    assert!(content.contains("/// Replace a interfaces item by key"));

    // Check PATCH operation (partial update by key)
    assert!(content.contains(
        "pub async fn patch_interfaces(name: String, data: Interface) -> Result<(), RpcError>"
    ));
    assert!(content.contains("/// Partially update a interfaces item by key"));

    // Check DELETE operation (remove by key)
    assert!(
        content.contains("pub async fn delete_interfaces(name: String) -> Result<(), RpcError>")
    );
    assert!(content.contains("/// Delete a interfaces item by key"));
}

#[test]
fn test_generate_crud_for_list_with_multiple_keys() {
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
        data_nodes: vec![DataNode::List(List {
            name: "routes".to_string(),
            description: Some("Routing table entries".to_string()),
            config: true,
            keys: vec!["destination".to_string(), "prefix-length".to_string()],
            children: vec![
                DataNode::Leaf(Leaf {
                    name: "destination".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "prefix-length".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint8 { range: None },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "next-hop".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that operations include both key parameters
    assert!(content.contains("pub async fn get_routes_by_key(destination: String, prefix_length: u8) -> Result<Route, RpcError>"));
    assert!(content.contains("pub async fn put_routes(destination: String, prefix_length: u8, data: Route) -> Result<(), RpcError>"));
    assert!(content.contains("pub async fn patch_routes(destination: String, prefix_length: u8, data: Route) -> Result<(), RpcError>"));
    assert!(content.contains("pub async fn delete_routes(destination: String, prefix_length: u8) -> Result<(), RpcError>"));
}

#[test]
fn test_generate_crud_for_state_list() {
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
        data_nodes: vec![DataNode::List(List {
            name: "connections".to_string(),
            description: Some("Active connections".to_string()),
            config: false,
            keys: vec!["id".to_string()],
            children: vec![
                DataNode::Leaf(Leaf {
                    name: "id".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint32 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "state".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check GET operations exist
    assert!(content.contains("pub async fn get_connections() -> Result<Connections, RpcError>"));
    assert!(content
        .contains("pub async fn get_connections_by_key(id: u32) -> Result<Connection, RpcError>"));

    // Check that write operations are NOT generated (config = false)
    assert!(!content.contains("pub async fn create_connections"));
    assert!(!content.contains("pub async fn put_connections"));
    assert!(!content.contains("pub async fn patch_connections"));
    assert!(!content.contains("pub async fn delete_connections"));
}

#[test]
fn test_generate_crud_for_multiple_data_nodes() {
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
        data_nodes: vec![
            DataNode::Container(Container {
                name: "config".to_string(),
                description: None,
                config: true,
                mandatory: false,
                children: vec![],
            }),
            DataNode::Container(Container {
                name: "state".to_string(),
                description: None,
                config: false,
                mandatory: false,
                children: vec![],
            }),
            DataNode::List(List {
                name: "users".to_string(),
                description: None,
                config: true,
                keys: vec!["username".to_string()],
                children: vec![DataNode::Leaf(Leaf {
                    name: "username".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                })],
            }),
        ],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check operations for config container
    assert!(content.contains("pub async fn get_config() -> Result<Config, RpcError>"));
    assert!(content.contains("pub async fn put_config(data: Config) -> Result<(), RpcError>"));

    // Check operations for state container (read-only)
    assert!(content.contains("pub async fn get_state() -> Result<State, RpcError>"));
    assert!(!content.contains("pub async fn put_state"));

    // Check operations for users list
    assert!(content.contains("pub async fn get_users() -> Result<Users, RpcError>"));
    assert!(content.contains("pub async fn create_users(data: User) -> Result<(), RpcError>"));
}

#[test]
fn test_crud_not_generated_when_no_data_nodes() {
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

    // Check that CRUD module is not generated
    assert!(!content.contains("pub mod crud"));
}

#[test]
fn test_crud_operations_use_rpc_error() {
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
        data_nodes: vec![DataNode::Container(Container {
            name: "test-container".to_string(),
            description: None,
            config: true,
            mandatory: false,
            children: vec![],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check that all CRUD operations return Result with RpcError
    assert!(content.contains("Result<TestContainer, RpcError>"));
    assert!(content.contains("Result<(), RpcError>"));
}
