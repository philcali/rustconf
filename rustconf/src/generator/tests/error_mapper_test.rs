//! Tests for ErrorMapper trait and DefaultErrorMapper generation.

use crate::generator::{CodeGenerator, GeneratorConfig, NamespaceMode};
use crate::parser::YangModule;
use std::path::PathBuf;

fn create_test_module() -> YangModule {
    YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![crate::parser::Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    }
}

#[test]
fn test_error_mapper_trait_generated_when_restful_rpcs_enabled() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: true,
        restful_namespace_mode: NamespaceMode::Enabled,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // Verify ErrorMapper is imported from rustconf-runtime (not generated)
    assert!(
        content.contains("use rustconf_runtime::{"),
        "Should import from rustconf-runtime when RESTful RPCs are enabled"
    );
    assert!(
        content.contains("ErrorMapper,"),
        "ErrorMapper should be imported from rustconf-runtime"
    );
    assert!(
        content.contains("DefaultErrorMapper,"),
        "DefaultErrorMapper should be imported from rustconf-runtime"
    );

    // Verify ErrorMapper trait is NOT generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub trait ErrorMapper: Send + Sync"),
        "ErrorMapper trait should not be generated (imported from rustconf-runtime)"
    );
}

#[test]
fn test_error_mapper_trait_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: false,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // Verify ErrorMapper trait is NOT generated
    assert!(
        !content.contains("pub trait ErrorMapper"),
        "ErrorMapper trait should not be generated when enable_restful_rpcs is false"
    );
}

#[test]
fn test_default_error_mapper_generated_when_restful_rpcs_enabled() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: true,
        restful_namespace_mode: NamespaceMode::Enabled,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // Verify DefaultErrorMapper is imported from rustconf-runtime (not generated)
    assert!(
        content.contains("DefaultErrorMapper,"),
        "DefaultErrorMapper should be imported from rustconf-runtime"
    );

    // Verify DefaultErrorMapper struct is NOT generated (comes from rustconf-runtime)
    assert!(
        !content.contains("pub struct DefaultErrorMapper;"),
        "DefaultErrorMapper struct should not be generated (imported from rustconf-runtime)"
    );
    assert!(
        !content.contains("impl ErrorMapper for DefaultErrorMapper"),
        "DefaultErrorMapper implementation should not be generated (comes from rustconf-runtime)"
    );
}

#[test]
fn test_default_error_mapper_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: false,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // Verify DefaultErrorMapper is NOT generated
    assert!(
        !content.contains("pub struct DefaultErrorMapper"),
        "DefaultErrorMapper should not be generated when enable_restful_rpcs is false"
    );
}

#[test]
fn test_default_error_mapper_maps_status_codes_correctly() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: true,
        restful_namespace_mode: NamespaceMode::Enabled,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // DefaultErrorMapper is now in rustconf-runtime, so we just verify it's imported
    assert!(
        content.contains("DefaultErrorMapper,"),
        "DefaultErrorMapper should be imported from rustconf-runtime"
    );

    // The actual status code mapping logic is in rustconf-runtime, not in generated code
    assert!(
        !content.contains("400 => RpcError::InvalidInput(body_text)"),
        "Status code mapping should not be in generated code (it's in rustconf-runtime)"
    );
}

#[test]
fn test_restconf_client_has_error_mapper_field() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: true,
        restful_namespace_mode: NamespaceMode::Enabled,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // RestconfClient is now in rustconf-runtime, so we just verify it's imported
    assert!(
        content.contains("RestconfClient,"),
        "RestconfClient should be imported from rustconf-runtime"
    );

    // The RestconfClient struct definition is in rustconf-runtime, not in generated code
    assert!(
        !content.contains("error_mapper: Option<Box<dyn ErrorMapper>>"),
        "RestconfClient definition should not be in generated code (it's in rustconf-runtime)"
    );
}

#[test]
fn test_restconf_client_has_with_error_mapper_method() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: true,
        restful_namespace_mode: NamespaceMode::Enabled,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // RestconfClient is now in rustconf-runtime, so we just verify it's imported
    assert!(
        content.contains("RestconfClient,"),
        "RestconfClient should be imported from rustconf-runtime"
    );

    // The with_error_mapper method is in rustconf-runtime, not in generated code
    assert!(
        !content.contains(
            "pub fn with_error_mapper(mut self, error_mapper: impl ErrorMapper + 'static) -> Self"
        ),
        "with_error_mapper method should not be in generated code (it's in rustconf-runtime)"
    );
}

#[test]
fn test_default_error_mapper_respects_derive_config() {
    let config = GeneratorConfig {
        output_dir: PathBuf::from("test_output"),
        module_name: "test_module".to_string(),
        enable_restful_rpcs: true,
        restful_namespace_mode: NamespaceMode::Enabled,
        derive_debug: true,
        derive_clone: true,
        ..Default::default()
    };

    let module = create_test_module();

    let generator = CodeGenerator::new(config);
    let result = generator.generate(&module).unwrap();
    let content = &result.files[0].content;

    // DefaultErrorMapper is now in rustconf-runtime with its own derives
    // The generated code just imports it
    assert!(
        content.contains("DefaultErrorMapper,"),
        "DefaultErrorMapper should be imported from rustconf-runtime"
    );

    // The DefaultErrorMapper struct definition is in rustconf-runtime, not in generated code
    assert!(
        !content.contains("pub struct DefaultErrorMapper;"),
        "DefaultErrorMapper definition should not be in generated code (it's in rustconf-runtime)"
    );
}
