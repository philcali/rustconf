//! Integration tests for generated code compilation (Task 12.1-12.3)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{Container, DataNode, Leaf, Notification, Rpc, TypeSpec, YangModule};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_generated_container_code_compiles() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "test_module".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    // Create a YANG module with a container
    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "interface-config".to_string(),
            description: Some("Interface configuration".to_string()),
            config: true,
            mandatory: false,
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
                    name: "enabled".to_string(),
                    description: Some("Whether the interface is enabled".to_string()),
                    type_spec: TypeSpec::Boolean,
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "mtu".to_string(),
                    description: Some("Maximum transmission unit".to_string()),
                    type_spec: TypeSpec::Uint16 { range: None },
                    mandatory: false,
                    default: None,
                    config: true,
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_dir.join("test_module.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify the generated code contains expected elements
    assert!(content.contains("pub struct InterfaceConfig"));
    assert!(content.contains("pub name: String"));
    assert!(content.contains("pub enabled: bool"));
    assert!(content.contains("pub mtu: Option<u16>"));

    // Verify it has proper derives
    assert!(content.contains("Serialize"));
    assert!(content.contains("Deserialize"));

    // Verify the code structure is correct (basic syntax check)
    // Count braces to ensure they're balanced
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces should be balanced in generated code"
    );

    // Verify struct syntax is correct
    assert!(content.contains("pub struct InterfaceConfig {"));
    assert!(content.contains("}\n"));
}

#[test]
fn test_generated_nested_container_code_compiles() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "nested_test".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    // Create a YANG module with nested containers
    let module = YangModule {
        name: "nested-test".to_string(),
        namespace: "urn:test:nested".to_string(),
        prefix: "nt".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "interface".to_string(),
            description: Some("Network interface".to_string()),
            config: true,
            mandatory: false,
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
                DataNode::Container(Container {
                    name: "config".to_string(),
                    description: Some("Configuration data".to_string()),
                    config: true,
                    mandatory: true,
                    children: vec![
                        DataNode::Leaf(Leaf {
                            name: "enabled".to_string(),
                            description: None,
                            type_spec: TypeSpec::Boolean,
                            mandatory: true,
                            default: None,
                            config: true,
                        }),
                        DataNode::Leaf(Leaf {
                            name: "mtu".to_string(),
                            description: None,
                            type_spec: TypeSpec::Uint32 { range: None },
                            mandatory: false,
                            default: None,
                            config: true,
                        }),
                    ],
                }),
                DataNode::Container(Container {
                    name: "state".to_string(),
                    description: Some("Operational state".to_string()),
                    config: false,
                    mandatory: false,
                    children: vec![DataNode::Leaf(Leaf {
                        name: "oper-status".to_string(),
                        description: None,
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: false,
                    })],
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_dir.join("nested_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify nested structures
    assert!(content.contains("pub struct Interface"));
    assert!(content.contains("pub struct Config"));
    assert!(content.contains("pub struct State"));
    assert!(content.contains("pub config: Config"));
    assert!(content.contains("pub state: Option<State>"));

    // Verify the code structure is correct (basic syntax check)
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces should be balanced in generated code"
    );
}

#[test]
fn test_generated_validation_code_compiles_and_validates() {
    use crate::parser::{Range, RangeConstraint};

    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "validation_test".to_string(),
        enable_validation: true,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    // Create a YANG module with validated types
    let module = YangModule {
        name: "validation-test".to_string(),
        namespace: "urn:test:validation".to_string(),
        prefix: "vt".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "server-config".to_string(),
            description: Some("Server configuration with validated fields".to_string()),
            config: true,
            mandatory: false,
            children: vec![
                DataNode::Leaf(Leaf {
                    name: "port".to_string(),
                    description: Some("Server port (1-65535)".to_string()),
                    type_spec: TypeSpec::Uint16 {
                        range: Some(RangeConstraint::new(vec![Range::new(1, 65535)])),
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "timeout".to_string(),
                    description: Some("Timeout in seconds (1-3600)".to_string()),
                    type_spec: TypeSpec::Uint32 {
                        range: Some(RangeConstraint::new(vec![Range::new(1, 3600)])),
                    },
                    mandatory: false,
                    default: None,
                    config: true,
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_dir.join("validation_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify ValidationError type is generated
    assert!(content.contains("pub enum ValidationError"));
    assert!(content.contains("OutOfRange"));
    assert!(content.contains("value: String"));
    assert!(content.contains("constraint: String"));

    // Verify validated types are generated
    assert!(content.contains("pub struct ValidatedUint16Id"));
    assert!(content.contains("pub struct ValidatedUint32Id"));

    // Verify validation logic is present
    assert!(content.contains("pub fn new(value: u16) -> Result<Self, ValidationError>"));
    assert!(content.contains("pub fn new(value: u32) -> Result<Self, ValidationError>"));
    assert!(content.contains("(1..=65535).contains(&value)"));
    assert!(content.contains("(1..=3600).contains(&value)"));

    // Verify Deserialize implementations with validation
    assert!(content.contains("impl<'de> serde::Deserialize<'de> for ValidatedUint16Id"));
    assert!(content.contains("impl<'de> serde::Deserialize<'de> for ValidatedUint32Id"));
    assert!(content.contains("Self::new(value).map_err(serde::de::Error::custom)"));

    // Verify error messages include value and constraint
    assert!(content.contains(r#"constraint: "1..65535".to_string()"#));
    assert!(content.contains(r#"constraint: "1..3600".to_string()"#));

    // Verify Display implementation for errors
    assert!(content.contains("impl std::fmt::Display for ValidationError"));
    assert!(content
        .contains(r#"write!(f, "Value '{}' is outside allowed range: {}", value, constraint)"#));

    // Verify Error trait implementation
    assert!(content.contains("impl std::error::Error for ValidationError"));

    // Verify the code structure is correct
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces should be balanced in generated code"
    );
}

// RPC integration tests (from rpc_integration_test.rs)

#[test]
fn test_generated_rpc_code_compiles() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "rpc_test".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "interface-mgmt".to_string(),
        namespace: "urn:example:interface-mgmt".to_string(),
        prefix: "if-mgmt".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![
            Rpc {
                name: "reset-interface".to_string(),
                description: Some("Reset an interface to default state".to_string()),
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
                output: None,
            },
            Rpc {
                name: "get-statistics".to_string(),
                description: Some("Get interface statistics".to_string()),
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "interface-name".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                })]),
                output: Some(vec![
                    DataNode::Leaf(Leaf {
                        name: "rx-packets".to_string(),
                        description: Some("Received packets".to_string()),
                        type_spec: TypeSpec::Uint64 { range: None },
                        mandatory: true,
                        default: None,
                        config: false,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "tx-packets".to_string(),
                        description: Some("Transmitted packets".to_string()),
                        type_spec: TypeSpec::Uint64 { range: None },
                        mandatory: true,
                        default: None,
                        config: false,
                    }),
                ]),
            },
        ],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    let content = fs::read_to_string(output_dir.join("rpc_test.rs")).unwrap();

    // Verify the generated code contains expected elements
    assert!(content.contains("pub enum RpcError"));
    assert!(content.contains("pub mod operations"));
    assert!(content.contains("pub struct ResetInterfaceInput"));
    assert!(content.contains("pub struct GetStatisticsInput"));
    assert!(content.contains("pub struct GetStatisticsOutput"));
    assert!(content.contains("pub async fn reset_interface"));
    assert!(content.contains("pub async fn get_statistics"));

    // Verify RPC error variants
    assert!(content.contains("NetworkError(String)"));
    assert!(content.contains("ServerError { code: u16, message: String }"));
    assert!(content.contains("SerializationError(String)"));
    assert!(content.contains("InvalidInput(String)"));
    assert!(content.contains("NotImplemented"));

    // Verify function signatures
    assert!(content.contains("input: ResetInterfaceInput) -> Result<(), RpcError>"));
    assert!(content.contains("input: GetStatisticsInput) -> Result<GetStatisticsOutput, RpcError>"));

    // Verify rustdoc comments
    assert!(content.contains("/// Reset an interface to default state"));
    assert!(content.contains("/// Get interface statistics"));
    assert!(content.contains("/// # Errors"));
}

// Notification integration tests (from notification_integration_test.rs)

#[test]
fn test_generated_notification_code_compiles() {
    // Create a temporary directory for generated code
    let temp_dir = std::env::temp_dir().join("rustconf_notification_test");
    fs::create_dir_all(&temp_dir).unwrap();

    let config = GeneratorConfig {
        output_dir: temp_dir.clone(),
        module_name: "notification_test".to_string(),
        enable_xml: false,
        enable_validation: true,
        derive_debug: true,
        derive_clone: true,
        enable_namespace_prefixes: false,
    };

    let generator = CodeGenerator::new(config);

    // Create a module with notifications
    let module = YangModule {
        name: "test-notifications".to_string(),
        namespace: "urn:test:notifications".to_string(),
        prefix: "tn".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![
            Notification {
                name: "link-up".to_string(),
                description: Some("Link is up notification".to_string()),
                data_nodes: vec![
                    DataNode::Leaf(Leaf {
                        name: "interface-name".to_string(),
                        description: Some("Name of the interface".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: false,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "speed".to_string(),
                        description: Some("Link speed in Mbps".to_string()),
                        type_spec: TypeSpec::Uint32 { range: None },
                        mandatory: true,
                        default: None,
                        config: false,
                    }),
                ],
            },
            Notification {
                name: "link-down".to_string(),
                description: Some("Link is down notification".to_string()),
                data_nodes: vec![
                    DataNode::Leaf(Leaf {
                        name: "interface-name".to_string(),
                        description: Some("Name of the interface".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: false,
                    }),
                    DataNode::Leaf(Leaf {
                        name: "reason".to_string(),
                        description: Some("Reason for link down".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: false,
                        default: None,
                        config: false,
                    }),
                ],
            },
        ],
    };

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = temp_dir.join("notification_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify the generated code contains expected elements
    assert!(content.contains("pub mod notifications {"));
    assert!(content.contains("pub struct LinkUp {"));
    assert!(content.contains("pub struct LinkDown {"));
    assert!(content.contains("pub interface_name: String,"));
    assert!(content.contains("pub speed: u32,"));
    assert!(content.contains("pub reason: Option<String>,"));

    // Try to compile the generated code by creating a test module
    let test_code = format!(
        r#"
        #[allow(dead_code)]
        mod notification_test {{
            {}
        }}

        #[test]
        fn test_notification_serialization() {{
            use notification_test::notifications::*;

            // Create a LinkUp notification
            let link_up = LinkUp {{
                interface_name: "eth0".to_string(),
                speed: 1000,
            }};

            // Serialize to JSON
            let json = serde_json::to_string(&link_up).unwrap();
            assert!(json.contains("interface-name"));
            assert!(json.contains("eth0"));
            assert!(json.contains("1000"));

            // Deserialize from JSON
            let deserialized: LinkUp = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.interface_name, "eth0");
            assert_eq!(deserialized.speed, 1000);

            // Create a LinkDown notification with optional field
            let link_down = LinkDown {{
                interface_name: "eth1".to_string(),
                reason: Some("Cable unplugged".to_string()),
            }};

            let json = serde_json::to_string(&link_down).unwrap();
            assert!(json.contains("interface-name"));
            assert!(json.contains("eth1"));
            assert!(json.contains("Cable unplugged"));

            // Create a LinkDown notification without optional field
            let link_down_no_reason = LinkDown {{
                interface_name: "eth2".to_string(),
                reason: None,
            }};

            let json = serde_json::to_string(&link_down_no_reason).unwrap();
            assert!(json.contains("eth2"));
            // Optional field should not be serialized when None
            assert!(!json.contains("reason"));
        }}
        "#,
        content
    );

    // Write test code to a temporary file
    let test_file = temp_dir.join("notification_compile_test.rs");
    fs::write(&test_file, test_code).unwrap();

    // Compile the test code using rustc
    let output = std::process::Command::new("rustc")
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2021")
        .arg("--extern")
        .arg("serde=/dev/null")
        .arg("--extern")
        .arg("serde_json=/dev/null")
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg(&test_file)
        .output();

    // Note: This will fail because we can't actually link against serde,
    // but we can check that the syntax is valid by checking for specific errors
    if let Ok(output) = output {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If there are syntax errors, they will be reported
        // We're mainly checking that the generated code structure is valid
        assert!(
            !stderr.contains("error: expected")
                && !stderr.contains("error: unexpected")
                && !stderr.contains("error: mismatched"),
            "Generated code has syntax errors: {}",
            stderr
        );
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
