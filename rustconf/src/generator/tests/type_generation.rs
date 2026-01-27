//! Tests for type generation (structs, enums, typedefs).

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{
    Case, Choice, Container, DataNode, Leaf, List, TypeDef, TypeSpec, YangModule, YangVersion,
};
use std::path::PathBuf;

#[test]
fn test_generate_simple_container() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let container = Container {
        name: "system".to_string(),
        description: Some("System configuration".to_string()),
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
    };

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(container)],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();
    let content = &generated.files[0].content;

    // Check that struct is generated
    assert!(content.contains("pub struct System"));
    assert!(content.contains("pub hostname: String"));
}

#[test]
fn test_generate_list() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let list = List {
        name: "users".to_string(),
        description: Some("List of users".to_string()),
        config: true,
        keys: vec!["username".to_string()],
        children: vec![
            DataNode::Leaf(Leaf {
                name: "username".to_string(),
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
                name: "email".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: true,
            }),
        ],
    };

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::List(list)],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();
    let content = &generated.files[0].content;

    // Check that struct is generated (singular form)
    assert!(content.contains("pub struct User"));
    assert!(content.contains("pub username: String"));
    assert!(content.contains("pub email: Option<String>"));
}

#[test]
fn test_generate_choice() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let choice = Choice {
        name: "protocol".to_string(),
        description: Some("Protocol choice".to_string()),
        mandatory: false,
        cases: vec![
            Case {
                name: "tcp".to_string(),
                description: None,
                data_nodes: vec![DataNode::Leaf(Leaf {
                    name: "tcp-port".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint16 { range: None },
                    mandatory: true,
                    default: None,
                    config: true,
                })],
            },
            Case {
                name: "udp".to_string(),
                description: None,
                data_nodes: vec![DataNode::Leaf(Leaf {
                    name: "udp-port".to_string(),
                    description: None,
                    type_spec: TypeSpec::Uint16 { range: None },
                    mandatory: true,
                    default: None,
                    config: true,
                })],
            },
        ],
    };

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Choice(choice)],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();
    let content = &generated.files[0].content;

    // Check that enum is generated
    assert!(content.contains("pub enum Protocol"));
    assert!(content.contains("Tcp(u16)"));
    assert!(content.contains("Udp(u16)"));
}

#[test]
fn test_generate_typedef() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test".to_string(),
        enable_validation: false,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let typedef = TypeDef {
        name: "percentage".to_string(),
        description: Some("Percentage value".to_string()),
        type_spec: TypeSpec::Uint8 { range: None },
        default: None,
        units: None,
    };

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![typedef],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();
    let content = &generated.files[0].content;

    // Check that type alias is generated
    assert!(content.contains("pub type Percentage = u8"));
}
