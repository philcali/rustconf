//! Tests for modular code generation.

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{Container, DataNode, Leaf, TypeSpec, YangModule, YangVersion};

#[test]
fn test_modular_generation_creates_multiple_files() {
    let config = GeneratorConfig {
        modular_output: true,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
            description: Some("Configuration container".to_string()),
            config: true,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "hostname".to_string(),
                description: Some("Device hostname".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: true,
            })],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Should generate 4 files: mod.rs, types.rs, operations.rs, validation.rs
    assert_eq!(generated.file_count(), 4);

    // Check that mod.rs exists
    let mod_file = generated.files.iter().find(|f| f.path.ends_with("mod.rs"));
    assert!(mod_file.is_some());
    let mod_content = &mod_file.unwrap().content;
    assert!(mod_content.contains("pub mod types;"));
    assert!(mod_content.contains("pub mod operations;"));
    assert!(mod_content.contains("pub mod validation;"));
    assert!(mod_content.contains("pub use rustconf_runtime::{"));

    // Check that types.rs exists
    let types_file = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("types.rs"));
    assert!(types_file.is_some());
    let types_content = &types_file.unwrap().content;
    assert!(types_content.contains("use serde::{Deserialize, Serialize};"));
    assert!(types_content.contains("pub struct Config"));

    // Check that operations.rs exists
    let ops_file = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("operations.rs"));
    assert!(ops_file.is_some());
    let ops_content = &ops_file.unwrap().content;
    assert!(
        ops_content.contains("use rustconf_runtime::{RestconfClient, HttpTransport, HttpRequest, HttpMethod, RpcError};")
    );

    // Check that validation.rs exists
    let val_file = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("validation.rs"));
    assert!(val_file.is_some());
    let val_content = &val_file.unwrap().content;
    assert!(val_content.contains("pub enum ValidationError"));
}

#[test]
fn test_modular_generation_without_validation() {
    let config = GeneratorConfig {
        modular_output: true,
        enable_validation: false,
        enable_restful_rpcs: true,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
            description: Some("Configuration container".to_string()),
            config: true,
            mandatory: false,
            children: vec![],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Should generate 3 files: mod.rs, types.rs, operations.rs (no validation.rs)
    assert_eq!(generated.file_count(), 3);

    // Check that validation.rs does not exist
    let val_file = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("validation.rs"));
    assert!(val_file.is_none());
}

#[test]
fn test_modular_generation_without_operations() {
    let config = GeneratorConfig {
        modular_output: true,
        enable_validation: true,
        enable_restful_rpcs: false,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
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

    // Should generate 3 files: mod.rs, types.rs, validation.rs (no operations.rs)
    assert_eq!(generated.file_count(), 3);

    // Check that operations.rs does not exist
    let ops_file = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("operations.rs"));
    assert!(ops_file.is_none());
}

#[test]
fn test_single_file_generation_still_works() {
    let config = GeneratorConfig {
        modular_output: false,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
            description: Some("Configuration container".to_string()),
            config: true,
            mandatory: false,
            children: vec![],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Should generate 1 file
    assert_eq!(generated.file_count(), 1);
}
