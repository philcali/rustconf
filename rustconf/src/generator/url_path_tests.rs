//! Unit tests for URL path construction (Task 10.4)

use crate::parser::{Container, DataNode, Leaf, List, TypeSpec, YangModule};

use super::*;

#[test]
fn test_generate_container_path_helper() {
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

    // Check that path helper function is generated
    assert!(content.contains("fn system_config_path() -> String"));
    assert!(content.contains("Build the RESTCONF URL path for the system-config container"));

    // Check that path is constructed correctly (without namespace prefix by default)
    assert!(content.contains("\"/restconf/data/system-config\".to_string()"));

    // Check that path helper is called in operations
    assert!(content.contains("let _path = system_config_path();"));
}

#[test]
fn test_generate_container_path_with_namespace_prefix() {
    let config = GeneratorConfig {
        enable_namespace_prefixes: true,
        ..Default::default()
    };
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
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

    // Check that path includes namespace prefix
    assert!(content.contains("\"/restconf/data/test:config\".to_string()"));
}

#[test]
fn test_generate_list_path_helpers() {
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
            description: None,
            config: true,
            keys: vec!["name".to_string()],
            children: vec![DataNode::Leaf(Leaf {
                name: "name".to_string(),
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

    // Check collection path helper
    assert!(content.contains("fn interfaces_path() -> String"));
    assert!(content.contains("Build the RESTCONF URL path for the interfaces collection"));
    assert!(content.contains("\"/restconf/data/interfaces\".to_string()"));

    // Check item path helper
    assert!(content.contains("fn interfaces_item_path(name: String) -> String"));
    assert!(content.contains("Build the RESTCONF URL path for a specific interfaces item"));
    assert!(content.contains("Keys are percent-encoded for URL safety"));

    // Check that path construction includes key encoding
    assert!(content.contains("let mut path = \"/restconf/data/interfaces\".to_string();"));
    assert!(
        content.contains("path.push_str(&format!(\"={}=\", percent_encode(&name.to_string())));")
    );

    // Check that path helpers are called in operations
    assert!(content.contains("let _path = interfaces_path();"));
    assert!(content.contains("let _path = interfaces_item_path(name);"));
}

#[test]
fn test_generate_list_path_with_multiple_keys() {
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
            description: None,
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
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Check item path helper with multiple keys
    assert!(
        content.contains("fn routes_item_path(destination: String, prefix_length: u8) -> String")
    );

    // Check that both keys are encoded
    assert!(content
        .contains("path.push_str(&format!(\"={}=\", percent_encode(&destination.to_string())));"));
    assert!(content.contains(
        "path.push_str(&format!(\"={}=\", percent_encode(&prefix_length.to_string())));"
    ));

    // Check that path helpers are called with both keys
    assert!(content.contains("let _path = routes_item_path(destination, prefix_length);"));
}

#[test]
fn test_generate_list_path_with_namespace_prefix() {
    let config = GeneratorConfig {
        enable_namespace_prefixes: true,
        ..Default::default()
    };
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::List(List {
            name: "users".to_string(),
            description: None,
            config: true,
            keys: vec!["id".to_string()],
            children: vec![DataNode::Leaf(Leaf {
                name: "id".to_string(),
                description: None,
                type_spec: TypeSpec::Uint32 { range: None },
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

    // Check that paths include namespace prefix
    assert!(content.contains("\"/restconf/data/test:users\".to_string()"));
    assert!(content.contains("let mut path = \"/restconf/data/test:users\".to_string();"));
}

#[test]
fn test_percent_encode_helper_generated() {
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
            name: "items".to_string(),
            description: None,
            config: true,
            keys: vec!["key".to_string()],
            children: vec![DataNode::Leaf(Leaf {
                name: "key".to_string(),
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

    // Check that percent_encode helper function is generated
    assert!(content.contains("fn percent_encode(s: &str) -> String"));
    assert!(content.contains("Percent-encode a string for use in URLs"));
    assert!(content.contains("This function encodes special characters according to RFC 3986"));

    // Check the implementation
    assert!(content.contains("'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~'"));
    assert!(content.contains("format!(\"%{:02X}\", c as u8)"));
}

#[test]
fn test_path_helpers_for_state_containers() {
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
            description: None,
            config: false,
            mandatory: false,
            children: vec![],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Path helpers should be generated even for state (read-only) containers
    assert!(content.contains("fn system_state_path() -> String"));
    assert!(content.contains("let _path = system_state_path();"));
}

#[test]
fn test_path_helpers_for_multiple_data_nodes() {
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
            DataNode::List(List {
                name: "items".to_string(),
                description: None,
                config: true,
                keys: vec!["id".to_string()],
                children: vec![DataNode::Leaf(Leaf {
                    name: "id".to_string(),
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

    // Check that path helpers are generated for all data nodes
    assert!(content.contains("fn config_path() -> String"));
    assert!(content.contains("fn items_path() -> String"));
    assert!(content.contains("fn items_item_path(id: String) -> String"));
}

#[test]
fn test_path_helpers_marked_as_allow_dead_code() {
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
            name: "test".to_string(),
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

    // Path helpers should have #[allow(dead_code)] to avoid warnings
    // since they're placeholders for future implementation
    assert!(content.contains("#[allow(dead_code)]"));
}
