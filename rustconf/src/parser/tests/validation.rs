//! Unit tests for semantic validation
//! Task 5.5: Write unit tests for semantic validation
//! Requirements: 7.2, 7.5

#[cfg(test)]
mod tests {
    use crate::parser::{ParseError, YangParser};

    // ========== Undefined Reference Tests ==========

    #[test]
    fn test_detect_undefined_typedef_reference() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type undefined-type;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Undefined typedef reference"));
                assert!(message.contains("undefined-type"));
            }
            _ => panic!("Expected SemanticError for undefined typedef"),
        }
    }

    #[test]
    fn test_detect_undefined_grouping_reference() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container system {
                    uses undefined-grouping;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Undefined grouping reference"));
                assert!(message.contains("undefined-grouping"));
            }
            _ => panic!("Expected SemanticError for undefined grouping"),
        }
    }

    #[test]
    fn test_valid_typedef_reference() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef percent {
                    type uint8;
                }
                
                leaf value {
                    type percent;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Valid typedef reference should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_valid_grouping_reference() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping endpoint {
                    leaf address {
                        type string;
                    }
                }
                
                container server {
                    uses endpoint;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Valid grouping reference should pass validation: {:?}",
            validation_result.err()
        );
    }

    // ========== Circular Dependency Tests ==========

    #[test]
    fn test_detect_circular_import_dependency() {
        // Create module A that imports B
        let module_a = r#"
            module module-a {
                namespace "urn:a";
                prefix a;
                
                import module-b {
                    prefix b;
                }
            }
        "#;

        // Create module B that imports A (circular)
        let module_b = r#"
            module module-b {
                namespace "urn:b";
                prefix b;
                
                import module-a {
                    prefix a;
                }
            }
        "#;

        let mut parser = YangParser::new();

        // Parse module B first
        let result_b = parser.parse_string(module_b, "module-b.yang");
        assert!(result_b.is_ok());

        // Parse module A (which imports B)
        let result_a = parser.parse_string(module_a, "module-a.yang");
        assert!(result_a.is_ok());
        let mod_a = result_a.unwrap();

        // Validation should detect circular import
        let validation_result = parser.validate_module(&mod_a);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Circular import dependency"));
            }
            _ => panic!("Expected SemanticError for circular import"),
        }
    }

    #[test]
    fn test_detect_circular_grouping_dependency() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping group-a {
                    uses group-b;
                }
                
                grouping group-b {
                    uses group-a;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Circular grouping dependency"));
            }
            _ => panic!("Expected SemanticError for circular grouping"),
        }
    }

    #[test]
    fn test_detect_indirect_circular_grouping() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping group-a {
                    uses group-b;
                }
                
                grouping group-b {
                    uses group-c;
                }
                
                grouping group-c {
                    uses group-a;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Circular grouping dependency"));
            }
            _ => panic!("Expected SemanticError for indirect circular grouping"),
        }
    }

    #[test]
    fn test_no_circular_dependency_in_valid_groupings() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping common-fields {
                    leaf value {
                        type string;
                    }
                }
                
                grouping extended-fields {
                    uses common-fields;
                    leaf extra {
                        type uint32;
                    }
                }
                
                container data {
                    uses extended-fields;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Valid grouping hierarchy should pass validation: {:?}",
            validation_result.err()
        );
    }

    // ========== Type Constraint Validation Tests ==========

    #[test]
    fn test_detect_invalid_range_constraint_min_greater_than_max() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type uint32 {
                        range "100..50";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Invalid range constraint"));
                assert!(message.contains("min"));
                assert!(message.contains("max"));
            }
            _ => panic!("Expected SemanticError for invalid range constraint"),
        }
    }

    #[test]
    fn test_detect_invalid_length_constraint_min_greater_than_max() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf name {
                    type string {
                        length "100..50";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Invalid length constraint"));
                assert!(message.contains("min"));
                assert!(message.contains("max"));
            }
            _ => panic!("Expected SemanticError for invalid length constraint"),
        }
    }

    #[test]
    fn test_valid_range_constraint() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type int32 {
                        range "1..100 | 200..300";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Valid range constraint should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_valid_length_constraint() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf name {
                    type string {
                        length "1..255";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Valid length constraint should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_valid_single_value_range() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type uint8 {
                        range "42..42";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Single value range (min == max) should be valid: {:?}",
            validation_result.err()
        );
    }

    // ========== Leafref Path Validation Tests ==========

    #[test]
    fn test_detect_empty_leafref_path() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf reference {
                    type leafref {
                        path "";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        // The parser might fail to parse empty path, which is also acceptable
        if result.is_err() {
            // If parsing fails, that's also a valid way to catch the error
            return;
        }

        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                assert!(message.contains("Leafref path cannot be empty"));
            }
            _ => panic!("Expected SemanticError for empty leafref path"),
        }
    }

    #[test]
    fn test_valid_leafref_path() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf target-value {
                    type string;
                }
                
                leaf ref-value {
                    type leafref {
                        path "../target-value";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Valid leafref path should pass validation: {:?}",
            validation_result.err()
        );
    }

    // ========== Complex Validation Scenarios ==========

    #[test]
    fn test_validation_with_nested_typedef_references() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef base-uint {
                    type uint32;
                }
                
                typedef derived-uint {
                    type base-uint;
                }
                
                leaf value {
                    type derived-uint;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Nested typedef references should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_validation_with_union_types() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef custom-type {
                    type uint32;
                }
                
                leaf value {
                    type union {
                        type string;
                        type custom-type;
                        type int32 {
                            range "1..100";
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Union types with valid references should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_validation_with_nested_containers_and_groupings() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping common-fields {
                    leaf id {
                        type uint32;
                    }
                    leaf name {
                        type string;
                    }
                }
                
                container outer {
                    container inner {
                        uses common-fields;
                        leaf extra {
                            type boolean;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Nested containers with groupings should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_error_messages_include_constraint_details() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type int32 {
                        range "500..100";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(validation_result.is_err());
        match validation_result.unwrap_err() {
            ParseError::SemanticError { message } => {
                // Error message should include the actual min and max values
                assert!(message.contains("500"));
                assert!(message.contains("100"));
            }
            _ => panic!("Expected SemanticError with constraint details"),
        }
    }

    #[test]
    fn test_validation_passes_for_module_without_issues() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef port-number {
                    type uint16 {
                        range "1..65535";
                    }
                }
                
                grouping server-config {
                    leaf hostname {
                        type string {
                            length "1..255";
                        }
                    }
                    leaf port {
                        type port-number;
                    }
                }
                
                container servers {
                    list server {
                        key "name";
                        leaf name {
                            type string;
                        }
                        uses server-config;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());
        let module = result.unwrap();

        let validation_result = parser.validate_module(&module);
        assert!(
            validation_result.is_ok(),
            "Well-formed module should pass all validation: {:?}",
            validation_result.err()
        );
    }
}
