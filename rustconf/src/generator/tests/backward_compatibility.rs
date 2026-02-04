//! Backward compatibility tests for RESTful RPC generation (Task 12)
//!
//! These tests verify that when enable_restful_rpcs is false, the generator
//! produces the same output as before the RESTful RPC feature was added.

use crate::generator::{CodeGenerator, GeneratorConfig, NamespaceMode};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};

/// Test that stub functions are generated when enable_restful_rpcs is false
#[test]
fn test_stub_generation_with_restful_rpcs_disabled() {
    // Create config with RESTful RPCs explicitly disabled
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
        ..Default::default()
    };
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![
            Rpc {
                name: "simple-rpc".to_string(),
                description: Some("A simple RPC with no parameters".to_string()),
                input: None,
                output: None,
            },
            Rpc {
                name: "rpc-with-input".to_string(),
                description: Some("RPC with input only".to_string()),
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "param".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                })]),
                output: None,
            },
            Rpc {
                name: "rpc-with-output".to_string(),
                description: Some("RPC with output only".to_string()),
                input: None,
                output: Some(vec![DataNode::Leaf(Leaf {
                    name: "result".to_string(),
                    description: None,
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                })]),
            },
            Rpc {
                name: "full-rpc".to_string(),
                description: Some("RPC with both input and output".to_string()),
                input: Some(vec![DataNode::Leaf(Leaf {
                    name: "input-param".to_string(),
                    description: None,
                    type_spec: TypeSpec::Int32 { range: None },
                    mandatory: true,
                    default: None,
                    config: true,
                })]),
                output: Some(vec![DataNode::Leaf(Leaf {
                    name: "output-result".to_string(),
                    description: None,
                    type_spec: TypeSpec::Int32 { range: None },
                    mandatory: true,
                    default: None,
                    config: false,
                })]),
            },
        ],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify stub functions are generated
    assert!(
        content.contains("pub async fn simple_rpc() -> Result<(), RpcError>"),
        "Simple RPC stub function should be generated"
    );
    assert!(
        content.contains(
            "pub async fn rpc_with_input(input: RpcWithInputInput) -> Result<(), RpcError>"
        ),
        "RPC with input stub function should be generated"
    );
    assert!(
        content.contains("pub async fn rpc_with_output() -> Result<RpcWithOutputOutput, RpcError>"),
        "RPC with output stub function should be generated"
    );
    assert!(
        content.contains(
            "pub async fn full_rpc(input: FullRpcInput) -> Result<FullRpcOutput, RpcError>"
        ),
        "Full RPC stub function should be generated"
    );

    // Verify all stubs return NotImplemented
    let stub_count = content.matches("Err(RpcError::NotImplemented)").count();
    assert_eq!(
        stub_count, 4,
        "All 4 stub functions should return NotImplemented"
    );

    // Verify NO client parameter in any function signature
    assert!(
        !content.contains("client: &RestconfClient"),
        "Stub functions should not have client parameter"
    );
    assert!(
        !content.contains("<T: HttpTransport>"),
        "Stub functions should not have generic type parameter"
    );
}

/// Test that function signatures remain unchanged when RESTful RPCs are disabled
#[test]
fn test_function_signatures_unchanged_with_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-operation".to_string(),
            description: Some("Test operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "value".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify exact function signature (no client parameter, no generic)
    assert!(
        content.contains("pub async fn test_operation(input: TestOperationInput) -> Result<TestOperationOutput, RpcError>"),
        "Function signature should match pre-RESTful format exactly"
    );

    // Verify input/output types are still generated
    assert!(
        content.contains("pub struct TestOperationInput {"),
        "Input type should be generated"
    );
    assert!(
        content.contains("pub struct TestOperationOutput {"),
        "Output type should be generated"
    );

    // Verify RpcError enum is still generated
    assert!(
        content.contains("pub enum RpcError {"),
        "RpcError enum should be generated"
    );
    assert!(
        content.contains("NotImplemented"),
        "NotImplemented variant should exist"
    );
}

/// Test that documentation indicates NotImplemented when RESTful RPCs are disabled
#[test]
fn test_documentation_indicates_not_implemented() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: Some("Test RPC".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify error documentation mentions NotImplemented
    assert!(
        content.contains("/// # Errors"),
        "Should have Errors section"
    );
    assert!(
        content.contains(
            "/// Returns `RpcError::NotImplemented` as RESTful RPC generation is disabled."
        ),
        "Error documentation should indicate NotImplemented"
    );

    // Verify NO RESTful usage examples
    assert!(
        !content.contains("RestconfClient::new"),
        "Should not show RestconfClient usage"
    );
    assert!(
        !content.contains("reqwest_adapter::ReqwestTransport"),
        "Should not show transport adapter usage"
    );
}

/// Test that HTTP abstractions are NOT generated when RESTful RPCs are disabled
#[test]
fn test_no_http_abstractions_when_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify HTTP abstractions are NOT generated
    assert!(
        !content.contains("pub enum HttpMethod {"),
        "HttpMethod should not be generated"
    );
    assert!(
        !content.contains("pub struct HttpRequest {"),
        "HttpRequest should not be generated"
    );
    assert!(
        !content.contains("pub struct HttpResponse {"),
        "HttpResponse should not be generated"
    );
    assert!(
        !content.contains("pub trait HttpTransport"),
        "HttpTransport trait should not be generated"
    );
    assert!(
        !content.contains("pub trait RequestInterceptor"),
        "RequestInterceptor trait should not be generated"
    );
    assert!(
        !content.contains("pub struct RestconfClient"),
        "RestconfClient should not be generated"
    );
}

/// Test that transport adapters are NOT generated when RESTful RPCs are disabled
#[test]
fn test_no_transport_adapters_when_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify transport adapters are NOT generated
    assert!(
        !content.contains("pub mod reqwest_adapter"),
        "reqwest_adapter module should not be generated"
    );
    assert!(
        !content.contains("pub struct ReqwestTransport"),
        "ReqwestTransport should not be generated"
    );
    assert!(
        !content.contains("pub mod hyper_adapter"),
        "hyper_adapter module should not be generated"
    );
    assert!(
        !content.contains("pub struct HyperTransport"),
        "HyperTransport should not be generated"
    );
}

/// Test that ErrorMapper is NOT generated when RESTful RPCs are disabled
#[test]
fn test_no_error_mapper_when_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify ErrorMapper is NOT generated
    assert!(
        !content.contains("pub trait ErrorMapper"),
        "ErrorMapper trait should not be generated"
    );
    assert!(
        !content.contains("pub struct DefaultErrorMapper"),
        "DefaultErrorMapper should not be generated"
    );
}

/// Test that Cargo.toml is NOT generated when RESTful RPCs are disabled
#[test]
fn test_no_cargo_toml_when_restful_rpcs_disabled() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();

    // Verify Cargo.toml is NOT in the generated files
    let has_cargo_toml = generated
        .files
        .iter()
        .any(|f| f.path.ends_with("Cargo.toml"));
    assert!(
        !has_cargo_toml,
        "Cargo.toml should not be generated when RESTful RPCs are disabled"
    );
}

/// Test that setting namespace mode without enabling RESTful RPCs is rejected
#[test]
fn test_namespace_mode_requires_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    // Try to set namespace mode without enabling RESTful RPCs
    config.restful_namespace_mode(NamespaceMode::Disabled);

    // Validation should fail
    let result = config.validate();
    assert!(
        result.is_err(),
        "Setting namespace mode without enabling RESTful RPCs should fail validation"
    );

    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("enable_restful_rpcs"),
        "Error message should mention enable_restful_rpcs"
    );
    assert!(
        error_msg.contains("restful_namespace_mode"),
        "Error message should mention restful_namespace_mode"
    );
}

/// Test that default config has RESTful RPCs disabled
#[test]
fn test_default_config_has_restful_rpcs_disabled() {
    let config = GeneratorConfig::default();
    assert!(
        !config.enable_restful_rpcs,
        "Default config should have enable_restful_rpcs set to false"
    );
    assert_eq!(
        config.restful_namespace_mode,
        NamespaceMode::Enabled,
        "Default namespace mode should be Enabled"
    );
}

/// Test that generated code compiles without RESTful dependencies when disabled
#[test]
fn test_generated_code_compiles_without_restful_dependencies() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_path.clone(),
        module_name: "test_bindings".to_string(),
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: Some("Test RPC".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "value".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_path.join("test_bindings.rs");
    assert!(generated_file.exists(), "Generated file should exist");

    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify no RESTful-specific dependencies are required
    assert!(
        !content.contains("async_trait"),
        "Should not require async_trait when RESTful RPCs are disabled"
    );
    assert!(
        !content.contains("urlencoding"),
        "Should not require urlencoding when RESTful RPCs are disabled"
    );
    assert!(
        !content.contains("reqwest"),
        "Should not require reqwest when RESTful RPCs are disabled"
    );
    assert!(
        !content.contains("hyper"),
        "Should not require hyper when RESTful RPCs are disabled"
    );

    // Verify only basic dependencies are used (serde is always needed for types)
    // The generated code should be compilable with just serde
    assert!(
        content.contains("use serde::{Deserialize, Serialize};"),
        "Should use serde for type serialization"
    );
}

/// Test that no Cargo.toml with RESTful dependencies is generated when disabled
#[test]
fn test_no_restful_dependencies_in_cargo_toml() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_path.clone(),
        module_name: "test_bindings".to_string(),
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Verify Cargo.toml was not created
    let cargo_toml = output_path.join("Cargo.toml");
    assert!(
        !cargo_toml.exists(),
        "Cargo.toml should not be generated when RESTful RPCs are disabled"
    );
}

/// Test that generated code structure is minimal when RESTful RPCs are disabled
#[test]
fn test_minimal_code_structure_without_restful_rpcs() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "simple-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Count the number of lines - should be minimal
    let line_count = content.lines().count();

    // With RESTful RPCs disabled, the generated code should be much smaller
    // It should only contain: RpcError enum, stub function, and basic structure
    // Let's verify it's under a reasonable threshold (e.g., 200 lines for a simple RPC)
    assert!(
        line_count < 200,
        "Generated code should be minimal without RESTful RPCs (got {} lines)",
        line_count
    );

    // Verify no large documentation blocks for RESTful features
    assert!(
        !content.contains("## Using a built-in transport adapter"),
        "Should not have transport adapter documentation"
    );
    assert!(
        !content.contains("## Implementing a custom transport"),
        "Should not have custom transport documentation"
    );
}

/// Test that compilation succeeds with minimal dependencies
#[test]
fn test_compilation_with_minimal_dependencies() {
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test_crate");
    fs::create_dir_all(&output_path).unwrap();

    // Create a minimal Cargo.toml with only serde
    let cargo_toml = r#"[package]
name = "test_crate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
"#;
    fs::write(output_path.join("Cargo.toml"), cargo_toml).unwrap();

    // Create src directory
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Generate code with RESTful RPCs disabled
    let config = GeneratorConfig {
        output_dir: src_dir.clone(),
        module_name: "lib".to_string(),
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-operation".to_string(),
            description: Some("Test operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "value".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Try to compile the generated code
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(&output_path)
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                panic!("Compilation failed with minimal dependencies:\n{}", stderr);
            }
        }
        Err(e) => {
            // If cargo is not available, skip this test
            eprintln!("Skipping compilation test: cargo not available ({})", e);
        }
    }
}

/// Test that module names are preserved when RESTful RPCs are disabled
#[test]
fn test_module_names_preserved() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
        module_name: "my_custom_module".to_string(),
        ..Default::default()
    };
    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();

    // Verify the file path uses the configured module name
    assert_eq!(
        generated.files[0]
            .path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        "my_custom_module.rs",
        "Module file name should match configured module_name"
    );
}

/// Test that type names and visibility are preserved
#[test]
fn test_type_names_and_visibility_preserved() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "my-operation".to_string(),
            description: Some("My operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "input-field".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "output-field".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify input type name follows naming convention
    assert!(
        content.contains("pub struct MyOperationInput {"),
        "Input type should use PascalCase naming"
    );
    assert!(
        content.contains("pub input_field: String,"),
        "Input field should use snake_case naming and be public"
    );

    // Verify output type name follows naming convention
    assert!(
        content.contains("pub struct MyOperationOutput {"),
        "Output type should use PascalCase naming"
    );
    assert!(
        content.contains("pub output_field: String,"),
        "Output field should use snake_case naming and be public"
    );

    // Verify function name follows naming convention
    assert!(
        content.contains("pub async fn my_operation("),
        "Function should use snake_case naming and be public"
    );

    // Verify RpcError is public
    assert!(
        content.contains("pub enum RpcError {"),
        "RpcError enum should be public"
    );
}

/// Test that no breaking changes are introduced to existing types
#[test]
fn test_no_breaking_changes_to_existing_types() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
        derive_debug: true,
        derive_clone: true,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "value".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify input type has expected derives
    assert!(
        content.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"),
        "Input type should have Debug, Clone, Serialize, Deserialize derives"
    );

    // Verify output type has expected derives
    let output_struct_start = content.find("pub struct TestRpcOutput").unwrap();
    let output_derives = &content[output_struct_start - 100..output_struct_start];
    assert!(
        output_derives.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"),
        "Output type should have Debug, Clone, Serialize, Deserialize derives"
    );

    // Verify RpcError has expected derives
    let error_enum_start = content.find("pub enum RpcError").unwrap();
    let error_derives = &content[error_enum_start - 100..error_enum_start];
    assert!(
        error_derives.contains("#[derive(Debug, Clone)]"),
        "RpcError should have Debug and Clone derives"
    );
}

/// Test that operations module structure is preserved
#[test]
fn test_operations_module_structure_preserved() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify operations module exists
    assert!(
        content.contains("pub mod operations {"),
        "Operations module should exist"
    );

    // Verify RPC function is inside operations module
    let ops_module_start = content.find("pub mod operations {").unwrap();
    let ops_module_end = content[ops_module_start..].find("\n}").unwrap() + ops_module_start;
    let ops_module_content = &content[ops_module_start..ops_module_end];

    assert!(
        ops_module_content.contains("pub async fn test_rpc"),
        "RPC function should be inside operations module"
    );
}

/// Test that serde attributes are preserved on types
#[test]
fn test_serde_attributes_preserved() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "kebab-case-field".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify serde rename attribute is present for kebab-case fields
    assert!(
        content.contains(r#"#[serde(rename = "kebab-case-field")]"#),
        "Serde rename attribute should be preserved for kebab-case fields"
    );
}

/// Test that existing code using the generator continues to work
#[test]
fn test_existing_usage_pattern_still_works() {
    // This test simulates how existing users would use the generator
    // without any knowledge of the new RESTful RPC feature

    // Create config the old way (no RESTful RPC methods called)
    let config = GeneratorConfig::default();

    // Verify the old default behavior is preserved
    assert!(
        !config.enable_restful_rpcs,
        "Default should have RESTful RPCs disabled for backward compatibility"
    );

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "legacy-module".to_string(),
        namespace: "urn:legacy".to_string(),
        prefix: "leg".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "legacy-operation".to_string(),
            description: Some("A legacy operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "param".to_string(),
                description: None,
                type_spec: TypeSpec::Int32 { range: None },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::Int32 { range: None },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    // Generate code - this should work exactly as before
    let result = generator.generate(&module);
    assert!(result.is_ok(), "Generation should succeed");

    let generated = result.unwrap();
    let content = &generated.files[0].content;

    // Verify the generated code has the expected structure
    assert!(
        content.contains("pub struct LegacyOperationInput"),
        "Input type should be generated"
    );
    assert!(
        content.contains("pub struct LegacyOperationOutput"),
        "Output type should be generated"
    );
    assert!(
        content.contains("pub async fn legacy_operation(input: LegacyOperationInput) -> Result<LegacyOperationOutput, RpcError>"),
        "Function should have the expected signature"
    );
    assert!(
        content.contains("Err(RpcError::NotImplemented)"),
        "Function should return NotImplemented"
    );

    // Verify no RESTful-specific code is present
    assert!(
        !content.contains("RestconfClient"),
        "Should not contain RestconfClient"
    );
    assert!(
        !content.contains("HttpTransport"),
        "Should not contain HttpTransport"
    );
}

/// Test that file count remains the same when RESTful RPCs are disabled
#[test]
fn test_file_count_unchanged() {
    let config = GeneratorConfig {
        enable_restful_rpcs: false,
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
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();

    // Should only generate one file (the main module file)
    assert_eq!(
        generated.file_count(),
        1,
        "Should only generate one file when RESTful RPCs are disabled"
    );
}
