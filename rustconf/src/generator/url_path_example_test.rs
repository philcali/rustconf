//! Example test to demonstrate URL path construction output

use crate::parser::{Container, DataNode, Leaf, List, TypeSpec, YangModule};

use super::*;

#[test]
fn test_url_path_construction_example() {
    let config = GeneratorConfig::default();
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "network".to_string(),
        namespace: "urn:example:network".to_string(),
        prefix: "net".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![
            DataNode::Container(Container {
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
            }),
            DataNode::List(List {
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
            }),
        ],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    println!("\n=== Generated URL Path Construction Code ===\n");

    // Extract and print the percent_encode function
    if let Some(start) = content.find("fn percent_encode") {
        if let Some(end) = content[start..].find("\n    }\n") {
            println!("{}\n", &content[start..start + end + 6]);
        }
    }

    // Extract and print the system path helper
    if let Some(start) = content.find("fn system_path") {
        if let Some(end) = content[start..].find("\n        }\n") {
            println!("{}\n", &content[start..start + end + 10]);
        }
    }

    // Extract and print the interfaces path helpers
    if let Some(start) = content.find("fn interfaces_path") {
        if let Some(end) = content[start..].find("\n        }\n\n        ///") {
            println!("{}\n", &content[start..start + end + 10]);
        }
    }

    if let Some(start) = content.find("fn interfaces_item_path") {
        if let Some(end) = content[start..].find("\n        }\n\n        ///") {
            println!("{}\n", &content[start..start + end + 10]);
        }
    }

    // Verify the key components are present
    assert!(content.contains("fn percent_encode(s: &str) -> String"));
    assert!(content.contains("fn system_path() -> String"));
    assert!(content.contains("fn interfaces_path() -> String"));
    assert!(content.contains("fn interfaces_item_path(name: String) -> String"));
    assert!(content.contains("percent_encode(&name.to_string())"));
}
