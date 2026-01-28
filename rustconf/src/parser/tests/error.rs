#[cfg(test)]
mod tests {
    use crate::parser::ParseError;
    use std::io;

    #[test]
    fn test_syntax_error_display() {
        let error = ParseError::SyntaxError {
            line: 42,
            column: 15,
            message: "unexpected token".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Syntax error at 42:15: unexpected token");
    }

    #[test]
    fn test_semantic_error_display() {
        let error = ParseError::SemanticError {
            message: "undefined reference to 'foo'".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Semantic error: undefined reference to 'foo'");
    }

    #[test]
    fn test_unresolved_import_display() {
        let error = ParseError::UnresolvedImport {
            module: "ietf-interfaces".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Unresolved import: ietf-interfaces");
    }

    #[test]
    fn test_io_error_display() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = ParseError::IoError(io_error);
        let display = format!("{}", error);
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_syntax_error_location_tracking() {
        let error = ParseError::SyntaxError {
            line: 100,
            column: 50,
            message: "missing semicolon".to_string(),
        };

        if let ParseError::SyntaxError {
            line,
            column,
            message,
        } = error
        {
            assert_eq!(line, 100);
            assert_eq!(column, 50);
            assert_eq!(message, "missing semicolon");
        } else {
            panic!("Expected SyntaxError variant");
        }
    }

    // Additional tests for task 2.4

    #[test]
    fn test_syntax_error_at_start_of_file() {
        let error = ParseError::SyntaxError {
            line: 1,
            column: 1,
            message: "expected 'module' keyword".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Syntax error at 1:1: expected 'module' keyword");

        // Verify location information is preserved
        if let ParseError::SyntaxError { line, column, .. } = error {
            assert_eq!(line, 1);
            assert_eq!(column, 1);
        }
    }

    #[test]
    fn test_syntax_error_with_long_message() {
        let long_message = "expected one of: 'container', 'list', 'leaf', 'leaf-list', 'choice', 'grouping', 'typedef', but found 'invalid'";
        let error = ParseError::SyntaxError {
            line: 25,
            column: 8,
            message: long_message.to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("25:8"));
        assert!(display.contains(long_message));
    }

    #[test]
    fn test_syntax_error_with_special_characters() {
        let error = ParseError::SyntaxError {
            line: 10,
            column: 5,
            message: "unexpected character: '@'".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Syntax error at 10:5: unexpected character: '@'");
    }

    #[test]
    fn test_multiple_syntax_errors_have_distinct_locations() {
        let error1 = ParseError::SyntaxError {
            line: 10,
            column: 5,
            message: "missing semicolon".to_string(),
        };
        let error2 = ParseError::SyntaxError {
            line: 20,
            column: 15,
            message: "unexpected token".to_string(),
        };

        let display1 = format!("{}", error1);
        let display2 = format!("{}", error2);

        assert!(display1.contains("10:5"));
        assert!(display2.contains("20:15"));
        assert_ne!(display1, display2);
    }

    #[test]
    fn test_semantic_error_with_context() {
        let error = ParseError::SemanticError {
            message: "circular dependency detected: module 'a' imports 'b' which imports 'a'"
                .to_string(),
        };
        let display = format!("{}", error);
        assert!(display.starts_with("Semantic error:"));
        assert!(display.contains("circular dependency"));
    }

    #[test]
    fn test_unresolved_import_with_module_name() {
        let error = ParseError::UnresolvedImport {
            module: "ietf-yang-types".to_string(),
        };
        let display = format!("{}", error);
        assert_eq!(display, "Unresolved import: ietf-yang-types");
    }

    #[test]
    fn test_io_error_variants() {
        let test_cases = vec![
            (io::ErrorKind::NotFound, "file not found"),
            (io::ErrorKind::PermissionDenied, "permission denied"),
            (io::ErrorKind::InvalidData, "invalid data"),
        ];

        for (kind, msg) in test_cases {
            let io_error = io::Error::new(kind, msg);
            let error = ParseError::IoError(io_error);
            let display = format!("{}", error);
            assert!(display.contains("I/O error"));
            assert!(display.contains(msg));
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = ParseError::SyntaxError {
            line: 42,
            column: 15,
            message: "test error".to_string(),
        };
        let debug = format!("{:?}", error);
        assert!(debug.contains("SyntaxError"));
        assert!(debug.contains("42"));
        assert!(debug.contains("15"));
        assert!(debug.contains("test error"));
    }

    #[test]
    fn test_syntax_error_location_at_large_line_numbers() {
        let error = ParseError::SyntaxError {
            line: 9999,
            column: 999,
            message: "error at end of large file".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("9999:999"));
    }

    #[test]
    fn test_error_message_formatting_consistency() {
        // All syntax errors should follow the same format: "Syntax error at line:column: message"
        let errors = vec![
            ParseError::SyntaxError {
                line: 1,
                column: 1,
                message: "msg1".to_string(),
            },
            ParseError::SyntaxError {
                line: 100,
                column: 50,
                message: "msg2".to_string(),
            },
            ParseError::SyntaxError {
                line: 5,
                column: 10,
                message: "msg3".to_string(),
            },
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(display.starts_with("Syntax error at "));
            assert!(display.contains(":"));
        }
    }

    #[test]
    fn test_semantic_error_without_location() {
        // Semantic errors don't include line:column location information
        let error = ParseError::SemanticError {
            message: "type mismatch".to_string(),
        };
        let display = format!("{}", error);
        // Should not contain line:column format (e.g., "42:15")
        assert!(!display.contains("at "));
        assert!(display.starts_with("Semantic error:"));
        assert!(display.contains("type mismatch"));
    }
}
