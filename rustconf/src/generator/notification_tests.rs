//! Unit tests for notification type generation (Task 10.2)

use crate::parser::{DataNode, Leaf, Notification, TypeSpec, YangModule};

use super::*;

#[test]
fn test_generate_notification_with_no_data() {
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
        notifications: vec![Notification {
            name: "system-restart".to_string(),
            description: Some("System is restarting".to_string()),
            data_nodes: vec![],
        }],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check notifications module
    assert!(content.contains("pub mod notifications {"));

    // Check notification struct
    assert!(content.contains("pub struct SystemRestart {"));

    // Check rustdoc comment
    assert!(content.contains("/// System is restarting"));

    // Check derive attributes
    assert!(content.contains("#[derive("));
    assert!(content.contains("Serialize"));
    assert!(content.contains("Deserialize"));
}

#[test]
fn test_generate_notification_with_data_nodes() {
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
        notifications: vec![Notification {
            name: "interface-state-change".to_string(),
            description: Some("Interface state has changed".to_string()),
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
                    name: "new-state".to_string(),
                    description: Some("New state of the interface".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "timestamp".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint64 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
            ],
        }],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check notification struct
    assert!(content.contains("pub struct InterfaceStateChange {"));

    // Check fields
    assert!(content.contains("pub interface_name: String,"));
    assert!(content.contains("pub new_state: String,"));
    assert!(content.contains("pub timestamp: u64,"));

    // Check field descriptions
    assert!(content.contains("/// Name of the interface"));
    assert!(content.contains("/// New state of the interface"));

    // Check serde attributes
    assert!(content.contains(r#"#[serde(rename = "interface-name")]"#));
    assert!(content.contains(r#"#[serde(rename = "new-state")]"#));
    assert!(content.contains(r#"#[serde(rename = "timestamp")]"#));
}

#[test]
fn test_generate_notification_with_optional_fields() {
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
        notifications: vec![Notification {
            name: "alarm".to_string(),
            description: Some("System alarm notification".to_string()),
            data_nodes: vec![
                DataNode::Leaf(Leaf {
                    name: "severity".to_string(),
                    description: Some("Alarm severity level".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "message".to_string(),
                    description: Some("Optional alarm message".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: false,
                    default: None,
                    config: false,
                }),
            ],
        }],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check mandatory field
    assert!(content.contains("pub severity: String,"));

    // Check optional field
    assert!(content.contains("pub message: Option<String>,"));

    // Check skip_serializing_if for optional field
    assert!(content.contains(r#"skip_serializing_if = "Option::is_none""#));
}

#[test]
fn test_generate_multiple_notifications() {
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
        notifications: vec![
            Notification {
                name: "link-up".to_string(),
                description: Some("Link is up".to_string()),
                data_nodes: vec![DataNode::Leaf(Leaf {
                    name: "interface".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                })],
            },
            Notification {
                name: "link-down".to_string(),
                description: Some("Link is down".to_string()),
                data_nodes: vec![DataNode::Leaf(Leaf {
                    name: "interface".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                })],
            },
        ],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check both notifications are generated
    assert!(content.contains("pub struct LinkUp {"));
    assert!(content.contains("/// Link is up"));

    assert!(content.contains("pub struct LinkDown {"));
    assert!(content.contains("/// Link is down"));

    // Check both have the interface field
    assert!(content.matches("pub interface: String,").count() >= 2);
}

#[test]
fn test_notification_not_generated_when_no_notifications() {
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

    // Check that notification module is not generated
    assert!(!content.contains("pub mod notifications"));
}

#[test]
fn test_notification_without_description() {
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
        notifications: vec![Notification {
            name: "event".to_string(),
            description: None,
            data_nodes: vec![],
        }],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check default rustdoc comment is generated
    assert!(content.contains("/// Notification payload for event."));
}

#[test]
fn test_notification_with_namespace_prefixes() {
    let config = GeneratorConfig { enable_namespace_prefixes: true, ..Default::default() };
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
        notifications: vec![Notification {
            name: "status-change".to_string(),
            description: Some("Status has changed".to_string()),
            data_nodes: vec![DataNode::Leaf(Leaf {
                name: "status".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })],
        }],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check namespace prefix in serde rename
    assert!(content.contains(r#"#[serde(rename = "t:status")]"#));
}
