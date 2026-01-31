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

    // Verify ErrorMapper trait is generated
    assert!(
        content.contains("pub trait ErrorMapper: Send + Sync"),
        "ErrorMapper trait should be generated with Send + Sync bounds"
    );

    assert!(
        content.contains("fn map_error(&self, response: &HttpResponse) -> RpcError;"),
        "ErrorMapper trait should have map_error method"
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

    // Verify DefaultErrorMapper struct is generated
    assert!(
        content.contains("pub struct DefaultErrorMapper;"),
        "DefaultErrorMapper struct should be generated"
    );

    // Verify DefaultErrorMapper implements ErrorMapper trait
    assert!(
        content.contains("impl ErrorMapper for DefaultErrorMapper"),
        "DefaultErrorMapper should implement ErrorMapper trait"
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

    // Verify status code mappings in DefaultErrorMapper implementation
    assert!(
        content.contains("400 => RpcError::InvalidInput(body_text)"),
        "DefaultErrorMapper should map 400 to InvalidInput"
    );

    assert!(
        content.contains("401 | 403 => RpcError::Unauthorized(body_text)"),
        "DefaultErrorMapper should map 401 and 403 to Unauthorized"
    );

    assert!(
        content.contains("404 => RpcError::NotFound(body_text)"),
        "DefaultErrorMapper should map 404 to NotFound"
    );

    assert!(
        content.contains("500..=599 => RpcError::ServerError"),
        "DefaultErrorMapper should map 500-599 to ServerError"
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

    // Verify RestconfClient has error_mapper field
    assert!(
        content.contains("error_mapper: Option<Box<dyn ErrorMapper>>"),
        "RestconfClient should have error_mapper field"
    );

    // Verify error_mapper is initialized to None in constructor
    assert!(
        content.contains("error_mapper: None,"),
        "RestconfClient constructor should initialize error_mapper to None"
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

    // Verify with_error_mapper method exists
    assert!(
        content.contains(
            "pub fn with_error_mapper(mut self, error_mapper: impl ErrorMapper + 'static) -> Self"
        ),
        "RestconfClient should have with_error_mapper method"
    );

    // Verify method sets the error_mapper field
    assert!(
        content.contains("self.error_mapper = Some(Box::new(error_mapper));"),
        "with_error_mapper should set the error_mapper field"
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

    // Verify DefaultErrorMapper derives Debug, Clone, and Copy
    assert!(
        content.contains("#[derive(Debug, Clone, Copy)]")
            && content.contains("pub struct DefaultErrorMapper;"),
        "DefaultErrorMapper should derive Debug, Clone, and Copy when configured"
    );
}
