//! Integration tests for error handling and reporting.
//!
//! Task 12.3: Implement error handling and reporting
//! Requirements: 3.5, 7.1, 7.2, 7.3

use rustconf::{BuildError, RustconfBuilder};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_error_reporting_for_syntax_error() {
    let temp_dir = TempDir::new().unwrap();
    let yang_file = temp_dir.path().join("syntax_error.yang");

    // Create a YANG file with a syntax error (missing closing brace)
    fs::write(
        &yang_file,
        r#"
module syntax-error {
    namespace "http://example.com/syntax-error";
    prefix se;
    
    container data {
        leaf value {
            type string;
        }
    // Missing closing brace
}
"#,
    )
    .unwrap();

    let output_dir = temp_dir.path().join("output");
    let builder = RustconfBuilder::new()
        .yang_file(&yang_file)
        .output_dir(&output_dir);

    let result = builder.generate();

    // Should fail with a parse error
    assert!(result.is_err());
    match result {
        Err(BuildError::ParseError(_)) => {
            // Expected - syntax errors are parse errors
        }
        Err(e) => panic!("Expected ParseError but got: {:?}", e),
        Ok(_) => panic!("Expected error but generation succeeded"),
    }
}

#[test]
fn test_error_reporting_for_configuration_error() {
    let temp_dir = TempDir::new().unwrap();

    // Try to generate without specifying any YANG files
    let builder = RustconfBuilder::new().output_dir(temp_dir.path());

    let result = builder.generate();

    // Should fail with a configuration error
    assert!(result.is_err());
    match result {
        Err(BuildError::ConfigurationError { message }) => {
            assert!(message.contains("No YANG files specified"));
        }
        Err(e) => panic!("Expected ConfigurationError but got: {:?}", e),
        Ok(_) => panic!("Expected error but generation succeeded"),
    }
}

#[test]
fn test_error_reporting_for_io_error() {
    let temp_dir = TempDir::new().unwrap();

    // Try to use a non-existent file
    let builder = RustconfBuilder::new()
        .yang_file("/nonexistent/path/to/file.yang")
        .output_dir(temp_dir.path());

    let result = builder.generate();

    // Should fail with a configuration error (file doesn't exist)
    assert!(result.is_err());
    match result {
        Err(BuildError::ConfigurationError { message }) => {
            assert!(message.contains("YANG file does not exist"));
        }
        Err(e) => panic!("Expected ConfigurationError but got: {:?}", e),
        Ok(_) => panic!("Expected error but generation succeeded"),
    }
}

#[test]
fn test_successful_generation_with_valid_yang() {
    let temp_dir = TempDir::new().unwrap();
    let yang_file = temp_dir.path().join("valid.yang");

    // Create a valid YANG file
    fs::write(
        &yang_file,
        r#"
module valid {
    namespace "http://example.com/valid";
    prefix v;
    
    container settings {
        leaf enabled {
            type boolean;
        }
        leaf name {
            type string;
        }
    }
}
"#,
    )
    .unwrap();

    let output_dir = temp_dir.path().join("output");
    let builder = RustconfBuilder::new()
        .yang_file(&yang_file)
        .output_dir(&output_dir)
        .enable_validation(true);

    let result = builder.generate();

    // Should succeed
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    // Verify output file was created
    let generated_file = output_dir.join("yang_bindings.rs");
    assert!(generated_file.exists(), "Generated file not found");

    // Verify content
    let content = fs::read_to_string(&generated_file).unwrap();
    assert!(content.contains("pub struct Settings"));
}

#[test]
fn test_error_display_format() {
    // Test that BuildError variants have proper Display implementations
    let config_error = BuildError::ConfigurationError {
        message: "test configuration error".to_string(),
    };

    let display = format!("{}", config_error);
    assert!(display.contains("Configuration error"));
    assert!(display.contains("test configuration error"));
}

#[test]
fn test_error_with_file_context() {
    use std::path::PathBuf;

    let error = BuildError::ConfigurationError {
        message: "test error".to_string(),
    };

    let path = PathBuf::from("/path/to/test.yang");
    let error_with_context = error.with_file_context(path);

    let display = format!("{}", error_with_context);
    assert!(display.contains("/path/to/test.yang"));
    assert!(display.contains("test error"));
}
