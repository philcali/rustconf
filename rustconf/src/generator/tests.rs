//! Consolidated unit tests for code generator module.
//!
//! This module organizes tests into logical submodules:
//! - `type_generation`: Tests for type generation (structs, enums, typedefs)
//! - `crud_operations`: Tests for CRUD operation generation
//! - `rpc_operations`: Tests for RPC operation generation
//! - `notifications`: Tests for notification generation
//! - `url_path`: Tests for RPC URL generation
//! - `url_path_example`: Integration tests for RPC URL generation
//! - `integration`: Integration tests for full module generation

use std::path::PathBuf;
use tempfile::TempDir;

use crate::parser::{YangModule, YangVersion};

use super::*;

// Basic generator tests
#[test]
fn test_generate_creates_generated_code() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();
    assert_eq!(generated.file_count(), 1);
    assert!(generated.total_size() > 0);
}

#[test]
fn test_generated_file_has_correct_path() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("output"),
        module_name: "my_module".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();
    assert_eq!(
        generated.files[0].path,
        PathBuf::from("output/my_module.rs")
    );
}

#[test]
fn test_write_files_creates_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_path.clone(),
        module_name: "test".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let result = generator.write_files(&generated);

    assert!(result.is_ok());
    assert!(output_path.exists());
}

// Submodules for organized tests
mod crud_operations;
mod integration;
mod notifications;
mod rpc_operations;
mod type_generation;
mod url_path;
mod url_path_example;
