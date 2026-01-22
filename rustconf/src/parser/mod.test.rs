//! Unit tests for YANG module header parsing.
//! Task 4.2: Write unit tests for module header parsing
//! Requirements: 1.1, 1.2, 1.3, 1.4

#[cfg(test)]
mod tests {
    use crate::parser::{ParseError, YangParser, YangVersion};

    #[test]
    fn test_parse_simple_module_with_namespace_and_prefix() {
        let input = r#"
            module example {
                namespace "http://example.com/example";
                prefix ex;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(
            result.is_ok(),
            "Failed to parse simple module: {:?}",
            result.err()
        );
        let module = result.unwrap();

        assert_eq!(module.name, "example");
        assert_eq!(module.namespace, "http://example.com/example");
        assert_eq!(module.prefix, "ex");
        assert_eq!(module.yang_version, None);
        assert!(module.imports.is_empty());
    }

    #[test]
    fn test_parse_module_with_yang_version() {
        let input = r#"
            module test-module {
                yang-version "1.1";
                namespace "urn:test:module";
                prefix test;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(
            result.is_ok(),
            "Failed to parse module with yang-version: {:?}",
            result.err()
        );
        let module = result.unwrap();

        assert_eq!(module.name, "test-module");
        assert_eq!(module.yang_version, Some(YangVersion::V1_1));
        assert_eq!(module.namespace, "urn:test:module");
        assert_eq!(module.prefix, "test");
    }

    #[test]
    fn test_parse_module_with_yang_version_1_0() {
        let input = r#"
            module old-module {
                yang-version "1.0";
                namespace "urn:old:module";
                prefix old;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.yang_version, Some(YangVersion::V1_0));
    }

    #[test]
    fn test_parse_module_with_import_statements() {
        let input = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import ietf-yang-types {
                    prefix yang;
                }
                
                import ietf-inet-types {
                    prefix inet;
                }
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(
            result.is_ok(),
            "Failed to parse module with imports: {:?}",
            result.err()
        );
        let module = result.unwrap();

        assert_eq!(module.name, "main");
        assert_eq!(module.imports.len(), 2);

        assert_eq!(module.imports[0].module, "ietf-yang-types");
        assert_eq!(module.imports[0].prefix, "yang");

        assert_eq!(module.imports[1].module, "ietf-inet-types");
        assert_eq!(module.imports[1].prefix, "inet");
    }

    #[test]
    fn test_parse_module_with_multiple_statements() {
        let input = r#"
            module complex {
                yang-version "1.1";
                namespace "urn:complex:module";
                prefix cx;
                
                import ietf-yang-types {
                    prefix yang;
                }
                
                organization "Test Organization";
                contact "test@example.com";
                description "A complex test module";
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(
            result.is_ok(),
            "Failed to parse complex module: {:?}",
            result.err()
        );
        let module = result.unwrap();

        assert_eq!(module.name, "complex");
        assert_eq!(module.yang_version, Some(YangVersion::V1_1));
        assert_eq!(module.namespace, "urn:complex:module");
        assert_eq!(module.prefix, "cx");
        assert_eq!(module.imports.len(), 1);
    }

    // Error cases

    #[test]
    fn test_error_missing_namespace() {
        let input = r#"
            module bad {
                prefix bad;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("namespace"));
            }
            _ => panic!("Expected SyntaxError for missing namespace"),
        }
    }

    #[test]
    fn test_error_missing_prefix() {
        let input = r#"
            module bad {
                namespace "urn:bad";
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("prefix"));
            }
            _ => panic!("Expected SyntaxError for missing prefix"),
        }
    }

    #[test]
    fn test_error_invalid_syntax_missing_brace() {
        let input = r#"
            module bad {
                namespace "urn:bad";
                prefix bad;
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_module_name() {
        let input = r#"
            module {
                namespace "urn:bad";
                prefix bad;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("module name"));
            }
            _ => panic!("Expected SyntaxError for invalid module name"),
        }
    }

    #[test]
    fn test_error_missing_semicolon_after_namespace() {
        let input = r#"
            module bad {
                namespace "urn:bad"
                prefix bad;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_yang_version() {
        let input = r#"
            module bad {
                yang-version "2.0";
                namespace "urn:bad";
                prefix bad;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("yang-version"));
            }
            _ => panic!("Expected SyntaxError for invalid yang-version"),
        }
    }

    #[test]
    fn test_error_import_missing_prefix() {
        let input = r#"
            module bad {
                namespace "urn:bad";
                prefix bad;
                
                import ietf-yang-types {
                }
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("prefix"));
            }
            _ => panic!("Expected SyntaxError for import missing prefix"),
        }
    }

    #[test]
    fn test_parse_module_with_hyphenated_name() {
        let input = r#"
            module my-test-module {
                namespace "urn:my:test";
                prefix mt;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, "my-test-module");
    }

    #[test]
    fn test_parse_module_with_underscored_prefix() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test_prefix;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.prefix, "test_prefix");
    }

    #[test]
    fn test_parse_module_with_comments() {
        let input = r#"
            // This is a test module
            module test {
                /* Multi-line
                   comment */
                namespace "urn:test"; // inline comment
                prefix test;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, "test");
    }

    #[test]
    fn test_parse_empty_module() {
        let input = r#"
            module minimal {
                namespace "urn:minimal";
                prefix min;
            }
        "#;

        let parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(
            result.is_ok(),
            "Failed to parse minimal module: {:?}",
            result.err()
        );
        let module = result.unwrap();
        assert_eq!(module.name, "minimal");
        assert!(module.data_nodes.is_empty());
        assert!(module.rpcs.is_empty());
        assert!(module.notifications.is_empty());
    }
}
