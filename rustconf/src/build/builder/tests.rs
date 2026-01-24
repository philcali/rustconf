//! Tests for RustconfBuilder.

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_builder_new_defaults() {
    let builder = RustconfBuilder::new();
    assert!(builder.yang_files.is_empty());
    assert!(builder.search_paths.is_empty());
}

#[test]
fn test_builder_yang_file() {
    let builder = RustconfBuilder::new().yang_file("test.yang");
    assert_eq!(builder.yang_files.len(), 1);
    assert_eq!(builder.yang_files[0], PathBuf::from("test.yang"));
}

#[test]
fn test_builder_search_path() {
    let builder = RustconfBuilder::new().search_path("specs/");
    assert_eq!(builder.search_paths.len(), 1);
    assert_eq!(builder.search_paths[0], PathBuf::from("specs/"));
}

#[test]
fn test_builder_output_dir() {
    let builder = RustconfBuilder::new().output_dir("/tmp/output");
    assert_eq!(builder.output_dir, PathBuf::from("/tmp/output"));
    assert_eq!(builder.config.output_dir, PathBuf::from("/tmp/output"));
}

#[test]
fn test_builder_enable_xml() {
    let builder = RustconfBuilder::new().enable_xml(true);
    assert!(builder.config.enable_xml);
}

#[test]
fn test_builder_enable_validation() {
    let builder = RustconfBuilder::new().enable_validation(true);
    assert!(builder.config.enable_validation);
}

#[test]
fn test_builder_module_name() {
    let builder = RustconfBuilder::new().module_name("my_module");
    assert_eq!(builder.config.module_name, "my_module");
}

#[test]
fn test_generate_no_yang_files() {
    let temp_dir = TempDir::new().unwrap();
    let builder = RustconfBuilder::new().output_dir(temp_dir.path());

    let result = builder.generate();
    assert!(result.is_err());
    match result {
        Err(BuildError::ConfigurationError { message }) => {
            assert!(message.contains("No YANG files specified"));
        }
        _ => panic!("Expected ConfigurationError"),
    }
}

#[test]
fn test_generate_missing_yang_file() {
    let temp_dir = TempDir::new().unwrap();
    let builder = RustconfBuilder::new()
        .yang_file("nonexistent.yang")
        .output_dir(temp_dir.path());

    let result = builder.generate();
    assert!(result.is_err());
}

#[test]
fn test_generate_simple_module() {
    let temp_dir = TempDir::new().unwrap();
    let yang_file = temp_dir.path().join("simple.yang");

    // Create a simple YANG module
    fs::write(
        &yang_file,
        r#"
module simple {
    namespace "http://example.com/simple";
    prefix simple;

    container settings {
        leaf name {
            type string;
        }
        leaf enabled {
            type boolean;
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
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    // Check that output file was created (default module name is "yang_bindings")
    let generated_file = output_dir.join("yang_bindings.rs");
    assert!(
        generated_file.exists(),
        "Generated file not found at {:?}",
        generated_file
    );

    // Check that the generated file contains expected content
    let content = fs::read_to_string(&generated_file).unwrap();
    assert!(content.contains("pub struct Settings"));
    assert!(content.contains("pub name:"));
    assert!(content.contains("pub enabled:"));
}

#[test]
fn test_generate_with_imports() {
    let temp_dir = TempDir::new().unwrap();
    let specs_dir = temp_dir.path().join("specs");
    fs::create_dir(&specs_dir).unwrap();

    // Create base module
    let base_yang = specs_dir.join("base-types.yang");
    fs::write(
        &base_yang,
        r#"
module base-types {
    namespace "http://example.com/base";
    prefix bt;

    typedef status-type {
        type string;
    }
}
"#,
    )
    .unwrap();

    // Create main module that imports base
    let main_yang = specs_dir.join("main.yang");
    fs::write(
        &main_yang,
        r#"
module main {
    namespace "http://example.com/main";
    prefix main;

    import base-types {
        prefix bt;
    }

    container settings {
        leaf current-status {
            type string;
        }
    }
}
"#,
    )
    .unwrap();

    let output_dir = temp_dir.path().join("output");
    let builder = RustconfBuilder::new()
        .yang_file(&main_yang)
        .search_path(&specs_dir)
        .output_dir(&output_dir);

    let result = builder.generate();
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    // Check that output file was created (default module name is "yang_bindings")
    let generated_file = output_dir.join("yang_bindings.rs");
    assert!(generated_file.exists(), "Generated file not found");
}

#[test]
fn test_generate_creates_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let yang_file = temp_dir.path().join("test.yang");

    fs::write(
        &yang_file,
        r#"
module test {
    namespace "http://example.com/test";
    prefix test;

    container data {
        leaf value {
            type int32;
        }
    }
}
"#,
    )
    .unwrap();

    // Use a nested output directory that doesn't exist yet
    let output_dir = temp_dir.path().join("nested").join("output");
    let builder = RustconfBuilder::new()
        .yang_file(&yang_file)
        .output_dir(&output_dir);

    let result = builder.generate();
    assert!(result.is_ok(), "Generation failed: {:?}", result.err());

    // Check that the nested directory was created
    assert!(output_dir.exists(), "Output directory was not created");
    assert!(
        output_dir.join("yang_bindings.rs").exists(),
        "Generated file not found"
    );
}
