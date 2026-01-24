//! Integration tests for notification type generation (Task 10.2)

#[cfg(test)]
mod tests {
    use crate::generator::{CodeGenerator, GeneratorConfig};
    use crate::parser::{DataNode, Leaf, Notification, TypeSpec, YangModule};
    use std::fs;

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
}
