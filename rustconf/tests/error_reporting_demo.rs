//! Demonstration of error reporting functionality.
//!
//! This test demonstrates how errors are reported through cargo's build script protocol.
//! Task 12.3: Implement error handling and reporting

use rustconf::{BuildError, RustconfBuilder};
use std::fs;
use tempfile::TempDir;

/// This test demonstrates error reporting for various error types.
/// The error messages are printed to stdout using cargo:warning format.
#[test]
fn demo_error_reporting() {
    println!("\n=== Error Reporting Demo ===\n");

    // Demo 1: Configuration error
    println!("Demo 1: Configuration Error (no YANG files)");
    let temp_dir = TempDir::new().unwrap();
    let builder = RustconfBuilder::new().output_dir(temp_dir.path());
    if let Err(e) = builder.generate() {
        println!("Error type: {:?}", e);
        println!("Error display: {}", e);
        e.report_to_cargo();
    }
    println!();

    // Demo 2: Parse error with file context
    println!("Demo 2: Parse Error with File Context");
    let yang_file = temp_dir.path().join("invalid.yang");
    fs::write(
        &yang_file,
        r#"
module invalid {
    namespace "http://example.com/invalid";
    prefix inv;
    
    container data {
        leaf value {
            type string;
        }
    // Missing closing brace
}
"#,
    )
    .unwrap();

    let builder = RustconfBuilder::new()
        .yang_file(&yang_file)
        .output_dir(temp_dir.path().join("output"));
    if let Err(e) = builder.generate() {
        println!("Error type: {:?}", e);
        println!("Error display: {}", e);
    }
    println!();

    // Demo 3: Successful generation
    println!("Demo 3: Successful Generation");
    let valid_yang = temp_dir.path().join("valid.yang");
    fs::write(
        &valid_yang,
        r#"
module valid {
    namespace "http://example.com/valid";
    prefix v;
    
    container settings {
        leaf enabled {
            type boolean;
        }
    }
}
"#,
    )
    .unwrap();

    let builder = RustconfBuilder::new()
        .yang_file(&valid_yang)
        .output_dir(temp_dir.path().join("output2"));
    match builder.generate() {
        Ok(_) => println!("✓ Generation succeeded!"),
        Err(e) => {
            println!("✗ Generation failed: {}", e);
            e.report_to_cargo();
        }
    }
    println!();

    println!("=== End of Demo ===\n");
}

/// Test that demonstrates error context preservation.
#[test]
fn demo_error_context() {
    use std::path::PathBuf;

    println!("\n=== Error Context Demo ===\n");

    let error = BuildError::ConfigurationError {
        message: "Invalid module name".to_string(),
    };

    println!("Original error: {}", error);

    let path = PathBuf::from("/path/to/my-module.yang");
    let error_with_context = error.with_file_context(path);

    println!("Error with context: {}", error_with_context);
    println!("\nReporting to cargo:");
    error_with_context.report_to_cargo();

    println!("\n=== End of Demo ===\n");
}
