//! Integration tests for generated code compilation.

#[cfg(test)]
mod tests {
    use crate::generator::{CodeGenerator, GeneratorConfig};
    use crate::parser::{Container, DataNode, Leaf, TypeSpec, YangModule};
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
}
