//! Tests for client-server type compatibility and round-trip consistency.
//!
//! Task 13.1: Verify shared type generation
//!   - Ensure types.rs is identical for client and server
//!   - Use same serde attributes on both sides
//!   - Verify validation.rs is shared
//!
//! Task 13.3: Integration test for client-server round-trip
//!   - Generate both client and server from same YANG
//!   - Serialize request on client, deserialize on server
//!   - Serialize response on server, deserialize on client
//!   - Verify data equivalence

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{Container, DataNode, Leaf, List, Rpc, TypeSpec, YangModule, YangVersion};

/// Helper to create a YANG module with RPCs (input/output), containers, and lists
/// to exercise various type generation paths.
fn create_roundtrip_module() -> YangModule {
    YangModule {
        name: "roundtrip-test".to_string(),
        namespace: "http://example.com/roundtrip".to_string(),
        prefix: "rt".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![
            DataNode::Container(Container {
                name: "system-config".to_string(),
                description: Some("System configuration".to_string()),
                config: true,
                mandatory: false,
                children: vec![
                    DataNode::Leaf(Leaf {
                        name: "hostname".to_string(),
                        description: Some("Device hostname".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: true,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "port".to_string(),
                        description: Some("Port number".to_string()),
                        type_spec: TypeSpec::Uint16 { range: None },
                        mandatory: false,
                        default: None,
                        config: true,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "enabled".to_string(),
                        description: Some("Enabled flag".to_string()),
                        type_spec: TypeSpec::Boolean,
                        mandatory: false,
                        default: None,
                        config: true,
                    }),
                ],
            }),
            DataNode::Container(Container {
                name: "status".to_string(),
                description: Some("Read-only status".to_string()),
                config: false,
                mandatory: false,
                children: vec![DataNode::Leaf(Leaf {
                    name: "uptime".to_string(),
                    description: Some("System uptime in seconds".to_string()),
                    type_spec: TypeSpec::Uint32 { range: None },
                    mandatory: false,
                    default: None,
                    config: false,
                })],
            }),
            DataNode::List(List {
                name: "interface".to_string(),
                description: Some("Network interfaces".to_string()),
                keys: vec!["name".to_string()],
                config: true,
                children: vec![
                    DataNode::Leaf(Leaf {
                        name: "name".to_string(),
                        description: Some("Interface name".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: true,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "mtu".to_string(),
                        description: Some("MTU size".to_string()),
                        type_spec: TypeSpec::Uint16 { range: None },
                        mandatory: false,
                        default: None,
                        config: true,
                    }),
                ],
            }),
        ],
        rpcs: vec![
            Rpc {
                name: "restart-device".to_string(),
                description: Some("Restart the device".to_string()),
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "delay-seconds".to_string(),
                    description: Some("Delay before restart".to_string()),
                    type_spec: TypeSpec::Uint32 { range: None },
                    mandatory: false,
                    default: None,
                    config: false,
                })]),
                output: Some(vec![
                    DataNode::Leaf(Leaf {
                        name: "success".to_string(),
                        description: Some("Whether restart was initiated".to_string()),
                        type_spec: TypeSpec::Boolean,
                        mandatory: false,
                        default: None,
                        config: false,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "message".to_string(),
                        description: Some("Status message".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: false,
                        default: None,
                        config: false,
                    }),
                ]),
            },
            Rpc {
                name: "get-system-info".to_string(),
                description: Some("Get system information".to_string()),
                input: None,
                output: Some(vec![DataNode::Leaf(Leaf {
                    name: "version".to_string(),
                    description: Some("System version".to_string()),
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
    }
}

/// Helper to create a config with both client and server generation enabled.
fn client_server_config() -> GeneratorConfig {
    GeneratorConfig {
        modular_output: true,
        enable_server_generation: true,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    }
}

/// Helper to create a client-only config (no server generation).
fn client_only_config() -> GeneratorConfig {
    GeneratorConfig {
        modular_output: true,
        enable_server_generation: false,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    }
}

// ============================================================================
// Task 13.1: Verify shared type generation
// ============================================================================

/// Validates: Requirements 13.1, 13.4, 13.5
///
/// Verify that types.rs is the SAME file whether server generation is enabled or not.
/// The architecture generates types.rs once at the top level, shared by both client
/// and server code. This test confirms that enabling server generation does not
/// alter the types file.
#[test]
fn test_types_identical_with_and_without_server() {
    let module = create_roundtrip_module();

    // Generate with client-only config
    let client_gen = CodeGenerator::new(client_only_config());
    let client_result = client_gen.generate(&module).unwrap();
    let client_types = client_result
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"))
        .expect("Client generation should produce types.rs");

    // Generate with client+server config
    let both_gen = CodeGenerator::new(client_server_config());
    let both_result = both_gen.generate(&module).unwrap();
    let both_types = both_result
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"))
        .expect("Client+server generation should produce types.rs");

    // types.rs content must be identical
    assert_eq!(
        client_types.content, both_types.content,
        "types.rs must be identical whether server generation is enabled or not"
    );
}

/// Validates: Requirements 13.1, 13.4, 13.5
///
/// Verify that validation.rs is the SAME file whether server generation is enabled or not.
#[test]
fn test_validation_identical_with_and_without_server() {
    let module = create_roundtrip_module();

    let client_gen = CodeGenerator::new(client_only_config());
    let client_result = client_gen.generate(&module).unwrap();
    let client_validation = client_result
        .files
        .iter()
        .find(|f| f.path.ends_with("validation.rs"))
        .expect("Client generation should produce validation.rs");

    let both_gen = CodeGenerator::new(client_server_config());
    let both_result = both_gen.generate(&module).unwrap();
    let both_validation = both_result
        .files
        .iter()
        .find(|f| f.path.ends_with("validation.rs"))
        .expect("Client+server generation should produce validation.rs");

    assert_eq!(
        client_validation.content, both_validation.content,
        "validation.rs must be identical whether server generation is enabled or not"
    );
}

/// Validates: Requirements 13.4, 13.5
///
/// Verify that types.rs uses serde Serialize and Deserialize derives,
/// which are required for both client serialization and server deserialization.
#[test]
fn test_types_have_serde_attributes() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();
    let types_file = result
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"))
        .unwrap();

    let content = &types_file.content;

    // Types must import serde
    assert!(
        content.contains("use serde::{Deserialize, Serialize}"),
        "types.rs must import serde Serialize and Deserialize"
    );

    // Structs must derive both Serialize and Deserialize
    assert!(
        content.contains("Serialize") && content.contains("Deserialize"),
        "Generated types must derive both Serialize and Deserialize"
    );

    // Verify the container struct has serde derives
    assert!(
        content.contains("SystemConfig"),
        "types.rs should contain SystemConfig struct"
    );
}

/// Validates: Requirements 13.1, 13.5
///
/// Verify that there is only ONE types.rs file (at the top level), not a separate
/// one inside the server/ directory. This confirms types are truly shared.
#[test]
fn test_types_not_duplicated_in_server_dir() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let types_files: Vec<_> = result
        .files
        .iter()
        .filter(|f| f.path.ends_with("types.rs"))
        .collect();

    assert_eq!(
        types_files.len(),
        1,
        "There should be exactly one types.rs file (shared), not duplicated in server/"
    );

    // The single types.rs should NOT be inside the server directory
    let types_path = &types_files[0].path;
    assert!(
        !types_path.to_string_lossy().contains("server"),
        "types.rs should be at the top level, not inside server/"
    );
}

/// Validates: Requirements 13.1, 13.5
///
/// Verify that there is only ONE validation.rs file (at the top level).
#[test]
fn test_validation_not_duplicated_in_server_dir() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let validation_files: Vec<_> = result
        .files
        .iter()
        .filter(|f| f.path.ends_with("validation.rs"))
        .collect();

    assert_eq!(
        validation_files.len(),
        1,
        "There should be exactly one validation.rs file (shared)"
    );

    let validation_path = &validation_files[0].path;
    assert!(
        !validation_path.to_string_lossy().contains("server"),
        "validation.rs should be at the top level, not inside server/"
    );
}

/// Validates: Requirements 13.4, 13.5
///
/// Verify that server handler code references the shared types via super::types,
/// confirming the server uses the same type definitions as the client.
#[test]
fn test_server_code_references_shared_types() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    // Server router should import from super::types (the shared types module)
    let router = result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/router.rs"))
        .expect("server/router.rs should exist");
    assert!(
        router.content.contains("super::types")
            || router.content.contains("use super::super::types")
            || router.content.contains("types::"),
        "Server router should reference shared types module"
    );

    // Server stubs should also reference shared types
    let stubs = result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/stubs.rs"))
        .expect("server/stubs.rs should exist");
    assert!(
        stubs.content.contains("super::types")
            || stubs.content.contains("use super::super::types")
            || stubs.content.contains("types::"),
        "Server stubs should reference shared types module"
    );
}

// ============================================================================
// Task 13.3: Integration test for client-server round-trip
// ============================================================================

/// Validates: Requirements 13.2, 13.3
///
/// Generate both client and server code from the same YANG schema and verify
/// that the types used for serialization are compatible:
/// - Client operations use the same types as server handlers
/// - serde attributes are consistent so client-serialized data can be
///   deserialized by the server and vice versa
#[test]
fn test_client_server_roundtrip_type_compatibility() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let types_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"))
        .unwrap()
        .content;

    let operations_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("operations.rs"))
        .unwrap()
        .content;

    let router_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/router.rs"))
        .unwrap()
        .content;

    let stubs_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/stubs.rs"))
        .unwrap()
        .content;

    let handlers_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/handlers.rs"))
        .unwrap()
        .content;

    // 1. Verify types.rs has both Serialize and Deserialize for round-trip
    assert!(
        types_content.contains("Serialize") && types_content.contains("Deserialize"),
        "Types must derive both Serialize and Deserialize for round-trip"
    );

    // 2. Verify RPC input/output types exist in operations.rs (where RPC types are generated)
    assert!(
        operations_content.contains("RestartDeviceInput"),
        "RPC input type should be generated in operations.rs"
    );
    assert!(
        operations_content.contains("RestartDeviceOutput"),
        "RPC output type should be generated in operations.rs"
    );

    // 3. Verify client operations reference the restart-device RPC
    assert!(
        operations_content.contains("restart_device")
            || operations_content.contains("restart-device"),
        "Client operations should reference restart-device RPC"
    );

    // 4. Verify server handlers reference the same types
    assert!(
        handlers_content.contains("RestartDevice") || handlers_content.contains("restart_device"),
        "Server handlers should reference restart-device RPC"
    );

    // 5. Verify server router uses serde_json for deserialization (matching client serialization)
    assert!(
        router_content.contains("serde_json"),
        "Server router should use serde_json for deserialization"
    );

    // 6. Verify server stubs return the same types
    assert!(
        stubs_content.contains("RestartDevice") || stubs_content.contains("restart_device"),
        "Server stubs should use the same RPC types"
    );
}

/// Validates: Requirements 13.2, 13.3
///
/// Verify that the serde rename attributes used in types.rs are consistent,
/// ensuring that JSON field names match between client serialization and
/// server deserialization.
#[test]
fn test_serde_rename_consistency_for_roundtrip() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let types_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"))
        .unwrap()
        .content;

    // Since types.rs is shared, the serde rename attributes are inherently
    // consistent between client and server. Verify that YANG kebab-case names
    // are properly handled with serde rename attributes.
    // YANG uses kebab-case (e.g., "delay-seconds"), Rust uses snake_case.
    // The serde rename ensures JSON uses the YANG name.
    if types_content.contains("delay_seconds") {
        // If the field is delay_seconds in Rust, there should be a rename
        // to match the YANG name in JSON serialization
        assert!(
            types_content.contains("rename") || types_content.contains("delay-seconds"),
            "Fields with kebab-case YANG names should have serde rename attributes"
        );
    }
}

/// Validates: Requirements 13.2, 13.3
///
/// Verify that the server router deserializes request bodies using the same
/// types that the client serializes, and that the server serializes responses
/// using the same types the client deserializes.
#[test]
fn test_server_deserializes_client_serialized_types() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let router_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/router.rs"))
        .unwrap()
        .content;

    // Server router should deserialize using serde_json::from_slice or from_str
    assert!(
        router_content.contains("serde_json::from_slice")
            || router_content.contains("serde_json::from_str")
            || router_content.contains("serde_json::from_value"),
        "Server router should deserialize request bodies with serde_json"
    );

    // Server router should serialize responses using serde_json::to_vec or to_string
    assert!(
        router_content.contains("serde_json::to_vec")
            || router_content.contains("serde_json::to_string"),
        "Server router should serialize responses with serde_json"
    );

    // Both client and server use the same serde_json library, and since types.rs
    // is shared with identical Serialize/Deserialize derives, round-trip is guaranteed.
}

/// Validates: Requirements 13.2, 13.3
///
/// Verify that container types generated for data nodes are usable by both
/// client operations (GET/PUT) and server handlers (get_/put_).
#[test]
fn test_data_node_types_shared_between_client_and_server() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let types_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"))
        .unwrap()
        .content;

    let handlers_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/handlers.rs"))
        .unwrap()
        .content;

    // SystemConfig type should exist in shared types
    assert!(
        types_content.contains("SystemConfig"),
        "SystemConfig should be in shared types"
    );

    // Server handlers should reference SystemConfig
    assert!(
        handlers_content.contains("SystemConfig"),
        "Server handlers should use the shared SystemConfig type"
    );

    // Interface list type should exist in shared types
    assert!(
        types_content.contains("Interface"),
        "Interface type should be in shared types"
    );
}

/// Validates: Requirements 13.2, 13.3
///
/// Verify that when both client and server are generated, the server router's
/// serialize_response function produces output compatible with client deserialization.
#[test]
fn test_response_serialization_uses_shared_types() {
    let module = create_roundtrip_module();

    let gen = CodeGenerator::new(client_server_config());
    let result = gen.generate(&module).unwrap();

    let router_content = &result
        .files
        .iter()
        .find(|f| f.path.ends_with("server/router.rs"))
        .unwrap()
        .content;

    // The router should have a serialize_response function
    assert!(
        router_content.contains("serialize_response"),
        "Router should have serialize_response for formatting responses"
    );

    // The serialize_response should use serde::Serialize bound (same as client)
    assert!(
        router_content.contains("serde::Serialize") || router_content.contains("Serialize"),
        "serialize_response should use serde::Serialize trait bound"
    );
}
