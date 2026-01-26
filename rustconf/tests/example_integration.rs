//! Integration test for the interface-config example.
//!
//! Task 13.4: Write integration test for example
//! Requirements: 3.3, 6.1, 6.3, 6.4
//!
//! This test verifies that:
//! - The example builds successfully
//! - The generated code compiles
//! - The example runs without errors

use std::process::Command;

#[test]
fn test_example_builds_successfully() {
    // Test that the example builds successfully
    // This validates Requirements 3.3 (build integration) and 6.3 (build.rs integration)

    let output = Command::new("cargo")
        .args(["build", "--package", "interface-config"])
        .output()
        .expect("Failed to execute cargo build");

    if !output.status.success() {
        eprintln!("=== STDOUT ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("=== STDERR ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Example build failed");
    }

    assert!(output.status.success(), "Example should build successfully");
}

#[test]
fn test_example_runs_without_errors() {
    // Test that the example runs without errors
    // This validates Requirements 6.1 (example module) and 6.4 (usage demonstration)

    // First ensure it's built
    let build_output = Command::new("cargo")
        .args(["build", "--package", "interface-config"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        build_output.status.success(),
        "Example must build before running"
    );

    // Now run the example
    let run_output = Command::new("cargo")
        .args(["run", "--package", "interface-config"])
        .output()
        .expect("Failed to execute cargo run");

    if !run_output.status.success() {
        eprintln!("=== STDOUT ===");
        eprintln!("{}", String::from_utf8_lossy(&run_output.stdout));
        eprintln!("=== STDERR ===");
        eprintln!("{}", String::from_utf8_lossy(&run_output.stderr));
        panic!("Example run failed");
    }

    assert!(
        run_output.status.success(),
        "Example should run without errors"
    );

    // Verify the output contains expected messages
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("rustconf Example"),
        "Output should contain example header"
    );
    assert!(
        stdout.contains("Successfully generated"),
        "Output should indicate successful generation"
    );
}

#[test]
fn test_generated_code_compiles() {
    // Test that the generated code compiles without warnings
    // This validates Requirement 3.3 (generated code availability)

    let output = Command::new("cargo")
        .args(["check", "--package", "interface-config"])
        .output()
        .expect("Failed to execute cargo check");

    if !output.status.success() {
        eprintln!("=== STDOUT ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("=== STDERR ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("Generated code does not compile");
    }

    assert!(
        output.status.success(),
        "Generated code should compile without errors"
    );
}
