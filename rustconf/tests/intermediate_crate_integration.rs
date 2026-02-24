//! Integration test for the intermediate crate pattern
//!
//! This test validates that:
//! 1. The test-intermediate-crate builds successfully with rustconf as build-dep
//! 2. The test-end-user project builds successfully depending only on test-intermediate-crate
//! 3. The test-end-user project has no build.rs
//! 4. Generated code is properly structured in src/generated/

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_intermediate_crate_builds() {
    let intermediate_crate_path = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("test-intermediate-crate");

    println!(
        "Building test-intermediate-crate at: {:?}",
        intermediate_crate_path
    );

    // Build the intermediate crate
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&intermediate_crate_path)
        .output()
        .expect("Failed to execute cargo build");

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("test-intermediate-crate failed to build");
    }

    println!("✓ test-intermediate-crate built successfully");

    // Verify that generated code exists
    let generated_dir = intermediate_crate_path.join("src").join("generated");
    assert!(
        generated_dir.exists(),
        "Generated directory should exist at {:?}",
        generated_dir
    );

    // Check for expected generated files
    let mod_file = generated_dir.join("mod.rs");
    let types_file = generated_dir.join("types.rs");
    let operations_file = generated_dir.join("operations.rs");

    assert!(
        mod_file.exists(),
        "mod.rs should be generated at {:?}",
        mod_file
    );
    assert!(
        types_file.exists(),
        "types.rs should be generated at {:?}",
        types_file
    );
    assert!(
        operations_file.exists(),
        "operations.rs should be generated at {:?}",
        operations_file
    );

    println!("✓ Generated files exist in src/generated/");
}

#[test]
fn test_end_user_project_builds_without_rustconf() {
    let end_user_path = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("test-end-user");

    println!("Building test-end-user at: {:?}", end_user_path);

    // Verify that test-end-user has no build.rs
    let build_rs = end_user_path.join("build.rs");
    assert!(
        !build_rs.exists(),
        "test-end-user should not have build.rs, but found one at {:?}",
        build_rs
    );

    println!("✓ test-end-user has no build.rs");

    // Verify that test-end-user does not depend on rustconf
    let cargo_toml_path = end_user_path.join("Cargo.toml");
    let cargo_toml_content =
        std::fs::read_to_string(&cargo_toml_path).expect("Failed to read test-end-user Cargo.toml");

    assert!(
        !cargo_toml_content.contains("rustconf"),
        "test-end-user should not depend on rustconf"
    );

    println!("✓ test-end-user does not depend on rustconf");

    // Build the end-user project
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&end_user_path)
        .output()
        .expect("Failed to execute cargo build");

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("test-end-user failed to build");
    }

    println!("✓ test-end-user built successfully without rustconf");
}

#[test]
fn test_end_user_can_run() {
    let end_user_path = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("test-end-user");

    println!("Running test-end-user at: {:?}", end_user_path);

    // Run the end-user application
    let output = Command::new("cargo")
        .arg("run")
        .current_dir(&end_user_path)
        .output()
        .expect("Failed to execute cargo run");

    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("test-end-user failed to run");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify expected output
    assert!(
        stdout.contains("Test End-User Application"),
        "Output should contain application title"
    );
    assert!(
        stdout.contains("Created device:"),
        "Output should show device creation"
    );
    assert!(
        stdout.contains("Created RestconfClient successfully"),
        "Output should show client creation"
    );
    assert!(
        stdout.contains("Success!"),
        "Output should indicate success"
    );

    println!("✓ test-end-user ran successfully");
    println!("Output:\n{}", stdout);
}

#[test]
fn test_generated_code_structure() {
    let intermediate_crate_path = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("test-intermediate-crate");

    let generated_dir = intermediate_crate_path.join("src").join("generated");

    // Read mod.rs and verify it has proper structure
    let mod_rs = generated_dir.join("mod.rs");
    let mod_content = std::fs::read_to_string(&mod_rs).expect("Failed to read mod.rs");

    // Should declare submodules
    assert!(
        mod_content.contains("pub mod types;"),
        "mod.rs should declare types module"
    );
    assert!(
        mod_content.contains("pub mod operations;"),
        "mod.rs should declare operations module"
    );

    // Should re-export rustconf-runtime types
    assert!(
        mod_content.contains("pub use rustconf_runtime"),
        "mod.rs should re-export rustconf_runtime types"
    );
    assert!(
        mod_content.contains("RestconfClient"),
        "mod.rs should re-export RestconfClient"
    );
    assert!(
        mod_content.contains("HttpTransport"),
        "mod.rs should re-export HttpTransport"
    );
    assert!(
        mod_content.contains("RpcError"),
        "mod.rs should re-export RpcError"
    );

    println!("✓ mod.rs has correct structure");

    // Read types.rs and verify it imports from rustconf-runtime
    let types_rs = generated_dir.join("types.rs");
    let types_content = std::fs::read_to_string(&types_rs).expect("Failed to read types.rs");

    // Should have serde imports
    assert!(
        types_content.contains("use serde"),
        "types.rs should import serde"
    );

    // Should contain Device type from YANG
    assert!(
        types_content.contains("Device"),
        "types.rs should contain Device type"
    );

    println!("✓ types.rs has correct structure");

    // Read operations.rs and verify it uses rustconf-runtime types
    let operations_rs = generated_dir.join("operations.rs");
    let operations_content =
        std::fs::read_to_string(&operations_rs).expect("Failed to read operations.rs");

    // Should import from rustconf-runtime
    assert!(
        operations_content.contains("use rustconf_runtime"),
        "operations.rs should import from rustconf_runtime"
    );
    assert!(
        operations_content.contains("RestconfClient"),
        "operations.rs should use RestconfClient"
    );
    assert!(
        operations_content.contains("HttpTransport"),
        "operations.rs should use HttpTransport"
    );

    println!("✓ operations.rs has correct structure");
}
