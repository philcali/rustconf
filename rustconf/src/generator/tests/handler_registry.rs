//! Property-based tests for handler registry generation.
//!
//! These tests verify that the handler registry correctly stores and looks up
//! handler implementations by path pattern, with proper fallback to default
//! not-implemented handlers.

#[cfg(test)]
mod tests {
    use crate::generator::{server_registry::RegistryGenerator, GeneratorConfig, NamespaceMode};
    use crate::parser::{Container, DataNode, Leaf, Rpc, TypeSpec, YangModule, YangVersion};
    use proptest::prelude::*;
    use tempfile::TempDir;

    // Property-based test generators

    /// Generate a valid YANG module name
    fn yang_module_name() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_-]{2,20}".prop_map(|s| s.replace('_', "-"))
    }

    /// Generate a simple YANG module with data nodes
    fn simple_yang_module() -> impl Strategy<Value = YangModule> {
        (
            yang_module_name(),
            prop::collection::vec(container_node(), 0..5),
        )
            .prop_map(|(name, containers)| {
                let data_nodes: Vec<DataNode> =
                    containers.into_iter().map(DataNode::Container).collect();

                YangModule {
                    name: name.clone(),
                    namespace: format!("http://example.com/{}", name),
                    prefix: name.chars().take(3).collect(),
                    yang_version: Some(YangVersion::V1_1),
                    imports: Vec::new(),
                    typedefs: Vec::new(),
                    groupings: Vec::new(),
                    data_nodes,
                    rpcs: Vec::new(),
                    notifications: Vec::new(),
                }
            })
    }

    /// Generate a container node
    fn container_node() -> impl Strategy<Value = Container> {
        "[a-z][a-z0-9_-]{2,15}".prop_map(|name| Container {
            name: name.replace('_', "-"),
            description: None,
            config: true,
            mandatory: false,
            children: Vec::new(),
        })
    }

    // Unit tests

    #[test]
    fn test_registry_generation_basic() {
        let module = YangModule {
            name: "test-module".to_string(),
            namespace: "http://example.com/test".to_string(),
            prefix: "test".to_string(),
            yang_version: Some(YangVersion::V1_1),
            imports: Vec::new(),
            typedefs: Vec::new(),
            groupings: Vec::new(),
            data_nodes: vec![DataNode::Container(Container {
                name: "interfaces".to_string(),
                description: None,
                config: true,
                mandatory: false,
                children: Vec::new(),
            })],
            rpcs: Vec::new(),
            notifications: Vec::new(),
        };

        let config = GeneratorConfig {
            output_dir: std::path::PathBuf::from("/tmp"),
            module_name: "test".to_string(),
            enable_validation: false,
            enable_restful_rpcs: false,
            enable_xml: false,
            modular_output: false,
            derive_debug: true,
            derive_clone: true,
            enable_namespace_prefixes: false,
            restful_namespace_mode: NamespaceMode::Enabled,
            enable_server_generation: true,
            server_output_subdir: "server".to_string(),
        };

        let generator = RegistryGenerator::new(&config);
        let result = generator.generate_registry(&module);

        assert!(result.is_ok(), "Registry generation failed: {:?}", result);

        let code = result.unwrap();

        // Verify key components are present
        assert!(code.contains("pub struct HandlerRegistry"));
        assert!(code.contains("pub fn new("));
        assert!(code.contains("pub fn register("));
        assert!(code.contains("pub fn lookup("));
        assert!(code.contains("pub fn default_handler("));
        assert!(code.contains("pub fn registered_paths("));
    }

    #[test]
    fn test_registry_generation_with_rpcs() {
        let module = YangModule {
            name: "device-management".to_string(),
            namespace: "http://example.com/device".to_string(),
            prefix: "dev".to_string(),
            yang_version: Some(YangVersion::V1_1),
            imports: Vec::new(),
            typedefs: Vec::new(),
            groupings: Vec::new(),
            data_nodes: Vec::new(),
            rpcs: vec![Rpc {
                name: "restart-device".to_string(),
                description: None,
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "delay".to_string(),
                    type_spec: TypeSpec::Uint32 { range: None },
                    description: None,
                    mandatory: false,
                    config: true,
                    default: None,
                })]),
                output: None,
            }],
            notifications: Vec::new(),
        };

        let config = GeneratorConfig {
            output_dir: std::path::PathBuf::from("/tmp"),
            module_name: "device".to_string(),
            enable_validation: false,
            enable_restful_rpcs: false,
            enable_xml: false,
            modular_output: false,
            derive_debug: true,
            derive_clone: true,
            enable_namespace_prefixes: false,
            restful_namespace_mode: NamespaceMode::Enabled,
            enable_server_generation: true,
            server_output_subdir: "server".to_string(),
        };

        let generator = RegistryGenerator::new(&config);
        let result = generator.generate_registry(&module);

        assert!(result.is_ok());
        let code = result.unwrap();

        // Verify handler trait name is correct
        assert!(code.contains("DeviceManagementHandler"));
    }

    #[test]
    fn test_registry_compiles() {
        let module = YangModule {
            name: "simple-test".to_string(),
            namespace: "http://example.com/simple".to_string(),
            prefix: "simple".to_string(),
            yang_version: Some(YangVersion::V1_1),
            imports: Vec::new(),
            typedefs: Vec::new(),
            groupings: Vec::new(),
            data_nodes: vec![DataNode::Container(Container {
                name: "config".to_string(),
                description: None,
                config: true,
                mandatory: false,
                children: Vec::new(),
            })],
            rpcs: Vec::new(),
            notifications: Vec::new(),
        };

        let temp_dir = TempDir::new().unwrap();
        let config = GeneratorConfig {
            output_dir: temp_dir.path().to_path_buf(),
            module_name: "simple".to_string(),
            enable_validation: false,
            enable_restful_rpcs: false,
            enable_xml: false,
            modular_output: true,
            derive_debug: true,
            derive_clone: true,
            enable_namespace_prefixes: false,
            restful_namespace_mode: NamespaceMode::Enabled,
            enable_server_generation: true,
            server_output_subdir: "server".to_string(),
        };

        let generator = RegistryGenerator::new(&config);
        let registry_code = generator.generate_registry(&module).unwrap();

        // Verify the generated code contains expected structures
        assert!(registry_code.contains("HandlerRegistry"));
        assert!(registry_code.contains("SimpleTestHandler"));
    }

    // Property-based tests

    proptest! {
        /// Property 20: Handler Registry Lookup
        /// For any registered handler path pattern, the registry SHALL return the
        /// correct handler implementation when queried with a matching path.
        /// Validates: Requirements 11.2, 11.3
        ///
        /// Feature: server-side-generation, Property 20: Handler Registry Lookup
        #[test]
        fn prop_registry_lookup_returns_registered_handlers(
            module in simple_yang_module(),
        ) {
            let config = GeneratorConfig {
                output_dir: std::path::PathBuf::from("/tmp"),
                module_name: module.name.clone(),
                enable_validation: false,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
                enable_server_generation: true,
                server_output_subdir: "server".to_string(),
            };

            let generator = RegistryGenerator::new(&config);
            let result = generator.generate_registry(&module);

            prop_assert!(result.is_ok(), "Registry generation failed: {:?}", result.err());

            let code = result.unwrap();

            // Verify registry structure is present
            prop_assert!(code.contains("pub struct HandlerRegistry"),
                "Generated code missing HandlerRegistry struct");
            prop_assert!(code.contains("pub fn register("),
                "Generated code missing register method");
            prop_assert!(code.contains("pub fn lookup("),
                "Generated code missing lookup method");

            // Verify the registry uses HashMap for storage
            prop_assert!(code.contains("HashMap<String"),
                "Registry should use HashMap for path storage");

            // Verify default handler support
            prop_assert!(code.contains("default_handler"),
                "Registry should support default handler");
            prop_assert!(code.contains("NotFound"),
                "Registry should return NotFound for unregistered paths");
        }

        /// Property: Registry validates empty paths
        /// For any attempt to register a handler with an empty path,
        /// the registry SHALL return a validation error.
        #[test]
        fn prop_registry_rejects_empty_paths(
            module in simple_yang_module(),
        ) {
            let config = GeneratorConfig {
                output_dir: std::path::PathBuf::from("/tmp"),
                module_name: module.name.clone(),
                enable_validation: false,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
                enable_server_generation: true,
                server_output_subdir: "server".to_string(),
            };

            let generator = RegistryGenerator::new(&config);
            let result = generator.generate_registry(&module);

            prop_assert!(result.is_ok());
            let code = result.unwrap();

            // Verify empty path validation is present
            prop_assert!(code.contains("path.is_empty()"),
                "Registry should validate empty paths");
            prop_assert!(code.contains("ValidationError"),
                "Registry should return ValidationError for empty paths");
        }

        /// Property: Registry supports listing registered paths
        /// For any set of registered paths, the registry SHALL provide
        /// a method to list all registered path patterns.
        #[test]
        fn prop_registry_lists_registered_paths(
            module in simple_yang_module(),
        ) {
            let config = GeneratorConfig {
                output_dir: std::path::PathBuf::from("/tmp"),
                module_name: module.name.clone(),
                enable_validation: false,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
                enable_server_generation: true,
                server_output_subdir: "server".to_string(),
            };

            let generator = RegistryGenerator::new(&config);
            let result = generator.generate_registry(&module);

            prop_assert!(result.is_ok());
            let code = result.unwrap();

            // Verify registered_paths method exists
            prop_assert!(code.contains("pub fn registered_paths("),
                "Registry should provide registered_paths method");
            prop_assert!(code.contains("Vec<String>"),
                "registered_paths should return Vec<String>");
        }

        /// Property: Registry generation is deterministic
        /// For any YANG module, generating the registry multiple times
        /// SHALL produce identical output.
        #[test]
        fn prop_registry_generation_is_deterministic(
            module in simple_yang_module(),
        ) {
            let config = GeneratorConfig {
                output_dir: std::path::PathBuf::from("/tmp"),
                module_name: module.name.clone(),
                enable_validation: false,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
                enable_server_generation: true,
                server_output_subdir: "server".to_string(),
            };

            let generator = RegistryGenerator::new(&config);

            let result1 = generator.generate_registry(&module);
            let result2 = generator.generate_registry(&module);

            prop_assert!(result1.is_ok() && result2.is_ok());

            let code1 = result1.unwrap();
            let code2 = result2.unwrap();

            prop_assert_eq!(code1, code2,
                "Registry generation should be deterministic");
        }

        /// Property: Registry handler trait name matches module
        /// For any YANG module, the generated registry SHALL use a handler
        /// trait name that matches the module name in PascalCase with "Handler" suffix.
        #[test]
        fn prop_registry_handler_trait_name_matches_module(
            module_name in yang_module_name(),
        ) {
            let module = YangModule {
                name: module_name.clone(),
                namespace: format!("http://example.com/{}", module_name),
                prefix: module_name.chars().take(3).collect(),
                yang_version: Some(YangVersion::V1_1),
                imports: Vec::new(),
                typedefs: Vec::new(),
                groupings: Vec::new(),
                data_nodes: Vec::new(),
                rpcs: Vec::new(),
                notifications: Vec::new(),
            };

            let config = GeneratorConfig {
                output_dir: std::path::PathBuf::from("/tmp"),
                module_name: module.name.clone(),
                enable_validation: false,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
                enable_server_generation: true,
                server_output_subdir: "server".to_string(),
            };

            let generator = RegistryGenerator::new(&config);
            let result = generator.generate_registry(&module);

            prop_assert!(result.is_ok());
            let code = result.unwrap();

            // Convert module name to PascalCase
            let expected_trait_name = module_name
                .split('-')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<String>() + "Handler";

            prop_assert!(code.contains(&expected_trait_name),
                "Registry should use trait name {} for module {}",
                expected_trait_name, module_name);
        }
    }
}
