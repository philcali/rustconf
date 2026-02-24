//! Property-based tests for validation logic preservation.
//!
//! These tests verify that validation logic is identical whether generated
//! to OUT_DIR (single-file mode) or src/ (modular mode).

#[cfg(test)]
mod tests {
    use crate::generator::{CodeGenerator, GeneratorConfig, NamespaceMode};
    use crate::parser::{
        DataNode, Leaf, LengthConstraint, LengthRange, Range, RangeConstraint, TypeSpec,
        YangModule, YangVersion,
    };
    use proptest::prelude::*;
    use tempfile::TempDir;

    // Property-based test generators

    /// Generate a valid range constraint for numeric types
    fn numeric_range_constraint() -> impl Strategy<Value = RangeConstraint> {
        prop::collection::vec((any::<i64>(), any::<i64>()), 1..3).prop_map(|ranges| {
            let ranges = ranges
                .into_iter()
                .map(|(min, max)| {
                    let (min, max) = if min > max { (max, min) } else { (min, max) };
                    Range { min, max }
                })
                .collect();
            RangeConstraint { ranges }
        })
    }

    /// Generate a valid length constraint for string/binary types
    fn length_constraint() -> impl Strategy<Value = LengthConstraint> {
        prop::collection::vec((0u64..1000u64, 1u64..1000u64), 1..3).prop_map(|lengths| {
            let lengths = lengths
                .into_iter()
                .map(|(min, max)| {
                    let (min, max) = if min > max { (max, min) } else { (min, max) };
                    LengthRange { min, max }
                })
                .collect();
            LengthConstraint { lengths }
        })
    }

    /// Generate a constrained TypeSpec
    fn constrained_type_spec() -> impl Strategy<Value = TypeSpec> {
        prop_oneof![
            numeric_range_constraint().prop_map(|range| TypeSpec::Int32 { range: Some(range) }),
            numeric_range_constraint().prop_map(|range| TypeSpec::Uint32 { range: Some(range) }),
            length_constraint().prop_map(|length| TypeSpec::String {
                length: Some(length),
                pattern: None
            }),
            length_constraint().prop_map(|length| TypeSpec::Binary {
                length: Some(length)
            }),
        ]
    }

    // Helper function to extract validation code from generated content
    fn extract_validation_code(content: &str) -> Option<String> {
        // Find ValidationError enum, trait implementations, and all validated type implementations
        let mut validation_code = String::new();
        let mut in_validation = false;
        let mut brace_count = 0;
        let mut paren_count = 0;

        for line in content.lines() {
            // Start capturing when we find validation-related items
            if line.contains("pub enum ValidationError")
                || line.contains("pub struct Validated")
                || (line.contains("impl")
                    && (line.contains("ValidationError") || line.contains("Validated")))
            {
                in_validation = true;
                brace_count = 0;
                paren_count = 0;
            }

            if in_validation {
                validation_code.push_str(line);
                validation_code.push('\n');

                // Track braces and parens to know when we're done with the type/impl
                for ch in line.chars() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 && paren_count == 0 {
                                in_validation = false;
                            }
                        }
                        '(' => paren_count += 1,
                        ')' => paren_count -= 1,
                        _ => {}
                    }
                }
            }
        }

        if validation_code.is_empty() {
            None
        } else {
            Some(validation_code)
        }
    }

    // Helper function to normalize generated code for comparison
    fn normalize_code(code: &str) -> String {
        code.lines()
            .filter(|line| {
                // Remove comments and empty lines
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("///")
            })
            .map(|line| {
                let trimmed = line.trim();
                // Remove use statements and derive attributes since they may differ in location/format
                if trimmed.starts_with("use ") || trimmed.starts_with("#[derive(") {
                    ""
                } else {
                    trimmed
                }
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    // Property-based tests

    proptest! {
        /// Property 7: Validation Logic Preservation
        /// For any YANG type with constraints, the generated validation logic SHALL be
        /// identical whether generated to OUT_DIR or src/, and SHALL validate the same
        /// set of valid and invalid inputs.
        /// Validates: Requirements 10.1, 10.2
        #[test]
        fn prop_validation_logic_identical_across_generation_modes(
            type_spec in constrained_type_spec(),
            module_name in "[a-z][a-z0-9_]{2,10}",
        ) {
            let leaf = Leaf {
                name: "test_field".to_string(),
                type_spec: type_spec.clone(),
                description: None,
                mandatory: false,
                config: true,
                default: None,
            };

            let module = YangModule {
                name: module_name.clone(),
                namespace: format!("http://example.com/{}", module_name),
                prefix: module_name[..2].to_string(),
                yang_version: Some(YangVersion::V1_1),
                imports: Vec::new(),
                typedefs: Vec::new(),
                groupings: Vec::new(),
                data_nodes: vec![DataNode::Leaf(leaf)],
                rpcs: Vec::new(),
                notifications: Vec::new(),
            };

            // Generate in single-file mode (OUT_DIR style)
            let temp_dir_single = TempDir::new().unwrap();
            let config_single = GeneratorConfig {
                output_dir: temp_dir_single.path().to_path_buf(),
                module_name: module.name.clone(),
                enable_validation: true,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
            };

            let generator_single = CodeGenerator::new(config_single);
            let result_single = generator_single.generate(&module);
            prop_assert!(result_single.is_ok(), "Single-file generation failed: {:?}", result_single.err());
            let generated_single = result_single.unwrap();

            // Generate in modular mode (src/ style)
            let temp_dir_modular = TempDir::new().unwrap();
            let config_modular = GeneratorConfig {
                output_dir: temp_dir_modular.path().to_path_buf(),
                module_name: module.name.clone(),
                enable_validation: true,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: true,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
            };

            let generator_modular = CodeGenerator::new(config_modular);
            let result_modular = generator_modular.generate(&module);
            prop_assert!(result_modular.is_ok(), "Modular generation failed: {:?}", result_modular.err());
            let generated_modular = result_modular.unwrap();

            // Extract validation code from single-file output
            let single_file_content = &generated_single.files[0].content;
            let validation_single = extract_validation_code(single_file_content);
            prop_assert!(validation_single.is_some(), "No validation code found in single-file output");

            // Extract validation code from modular output (validation.rs)
            let validation_file = generated_modular.files.iter()
                .find(|f| f.path.ends_with("validation.rs"));
            prop_assert!(validation_file.is_some(), "No validation.rs file found in modular output");
            let validation_modular = validation_file.unwrap().content.clone();

            // Normalize and compare validation code
            let normalized_single = normalize_code(&validation_single.unwrap());
            let normalized_modular = normalize_code(&validation_modular);

            // The validation logic should be identical
            prop_assert_eq!(
                normalized_single,
                normalized_modular,
                "Validation logic differs between single-file and modular generation"
            );
        }

        /// Property: ValidationError type is consistent
        /// For any generation mode, the ValidationError type should be identical
        #[test]
        fn prop_validation_error_type_consistent(
            module_name in "[a-z][a-z0-9_]{2,10}",
        ) {
            let module = YangModule {
                name: module_name.clone(),
                namespace: format!("http://example.com/{}", module_name),
                prefix: module_name[..2].to_string(),
                yang_version: Some(YangVersion::V1_1),
                imports: Vec::new(),
                typedefs: Vec::new(),
                groupings: Vec::new(),
                data_nodes: Vec::new(),
                rpcs: Vec::new(),
                notifications: Vec::new(),
            };

            // Generate in both modes
            let temp_dir_single = TempDir::new().unwrap();
            let config_single = GeneratorConfig {
                output_dir: temp_dir_single.path().to_path_buf(),
                module_name: module.name.clone(),
                enable_validation: true,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: false,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
            };

            let generator_single = CodeGenerator::new(config_single);
            let result_single = generator_single.generate(&module);
            prop_assert!(result_single.is_ok());
            let generated_single = result_single.unwrap();

            let temp_dir_modular = TempDir::new().unwrap();
            let config_modular = GeneratorConfig {
                output_dir: temp_dir_modular.path().to_path_buf(),
                module_name: module.name.clone(),
                enable_validation: true,
                enable_restful_rpcs: false,
                enable_xml: false,
                modular_output: true,
                derive_debug: true,
                derive_clone: true,
                enable_namespace_prefixes: false,
                restful_namespace_mode: NamespaceMode::Enabled,
            };

            let generator_modular = CodeGenerator::new(config_modular);
            let result_modular = generator_modular.generate(&module);
            prop_assert!(result_modular.is_ok());
            let generated_modular = result_modular.unwrap();

            // Both should contain ValidationError
            let single_content = &generated_single.files[0].content;
            prop_assert!(single_content.contains("pub enum ValidationError"));

            let validation_file = generated_modular.files.iter()
                .find(|f| f.path.ends_with("validation.rs"));
            prop_assert!(validation_file.is_some());
            prop_assert!(validation_file.unwrap().content.contains("pub enum ValidationError"));
        }
    }

    // Unit tests

    #[test]
    fn test_validation_file_contains_error_type() {
        let module = YangModule {
            name: "test".to_string(),
            namespace: "http://example.com/test".to_string(),
            prefix: "t".to_string(),
            yang_version: Some(YangVersion::V1_1),
            imports: Vec::new(),
            typedefs: Vec::new(),
            groupings: Vec::new(),
            data_nodes: Vec::new(),
            rpcs: Vec::new(),
            notifications: Vec::new(),
        };

        let temp_dir = TempDir::new().unwrap();
        let config = GeneratorConfig {
            output_dir: temp_dir.path().to_path_buf(),
            module_name: "test".to_string(),
            enable_validation: true,
            enable_restful_rpcs: false,
            enable_xml: false,
            modular_output: true,
            derive_debug: true,
            derive_clone: true,
            enable_namespace_prefixes: false,
            restful_namespace_mode: NamespaceMode::Enabled,
        };

        let generator = CodeGenerator::new(config);
        let result = generator.generate(&module);
        assert!(result.is_ok());

        let generated = result.unwrap();
        let validation_file = generated
            .files
            .iter()
            .find(|f| f.path.ends_with("validation.rs"));

        assert!(validation_file.is_some());
        let content = &validation_file.unwrap().content;

        // Should contain ValidationError enum
        assert!(content.contains("pub enum ValidationError"));
        assert!(content.contains("OutOfRange"));
        assert!(content.contains("InvalidLength"));
        assert!(content.contains("InvalidPattern"));
    }

    #[test]
    fn test_validation_logic_for_range_constraint() {
        let leaf = Leaf {
            name: "port".to_string(),
            type_spec: TypeSpec::Uint16 {
                range: Some(RangeConstraint {
                    ranges: vec![Range {
                        min: 1024,
                        max: 65535,
                    }],
                }),
            },
            description: None,
            mandatory: false,
            config: true,
            default: None,
        };

        let module = YangModule {
            name: "test".to_string(),
            namespace: "http://example.com/test".to_string(),
            prefix: "t".to_string(),
            yang_version: Some(YangVersion::V1_1),
            imports: Vec::new(),
            typedefs: Vec::new(),
            groupings: Vec::new(),
            data_nodes: vec![DataNode::Leaf(leaf)],
            rpcs: Vec::new(),
            notifications: Vec::new(),
        };

        let temp_dir = TempDir::new().unwrap();
        let config = GeneratorConfig {
            output_dir: temp_dir.path().to_path_buf(),
            module_name: "test".to_string(),
            enable_validation: true,
            enable_restful_rpcs: false,
            enable_xml: false,
            modular_output: true,
            derive_debug: true,
            derive_clone: true,
            enable_namespace_prefixes: false,
            restful_namespace_mode: NamespaceMode::Enabled,
        };

        let generator = CodeGenerator::new(config);
        let result = generator.generate(&module);
        assert!(result.is_ok());

        let generated = result.unwrap();
        let validation_file = generated
            .files
            .iter()
            .find(|f| f.path.ends_with("validation.rs"));

        assert!(validation_file.is_some());
        let content = &validation_file.unwrap().content;

        // Should contain validated type with range check
        assert!(content.contains("1024..=65535"));
        assert!(content.contains("OutOfRange"));
    }
}
