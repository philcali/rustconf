//! Tests for feature flag compilation.

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_generated_code_compiles_without_transport_features() {
    // Create a temporary directory for the generated code
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("generated");

    // Configure generator with RESTful RPCs enabled
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    config.output_dir = output_path.clone();
    config.module_name = "test_bindings".to_string();

    let generator = CodeGenerator::new(config);

    // Create a simple YANG module with an RPC
    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-operation".to_string(),
            description: Some("Test RPC operation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "input-param".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "output-param".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: true,
            })]),
        }],
        notifications: vec![],
    };

    // Generate the code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Verify Cargo.toml was generated
    let cargo_toml_path = output_path.join("Cargo.toml");
    assert!(cargo_toml_path.exists(), "Cargo.toml should be generated");

    // Create a lib.rs file that includes the generated module
    let lib_rs_path = output_path.join("src");
    fs::create_dir_all(&lib_rs_path).unwrap();
    let lib_rs_file = lib_rs_path.join("lib.rs");

    // Copy the generated module file to src/lib.rs
    let module_file = output_path.join("test_bindings.rs");
    fs::copy(&module_file, &lib_rs_file).unwrap();

    // Try to compile the generated code without any features
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(cargo_toml_path.to_str().unwrap())
        .arg("--no-default-features")
        .output()
        .expect("Failed to execute cargo check");

    // Print output for debugging if compilation fails
    if !output.status.success() {
        eprintln!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
    }

    // Verify compilation succeeded
    assert!(
        output.status.success(),
        "Generated code should compile without transport features"
    );
}

#[test]
fn test_generated_code_has_core_traits_without_features() {
    // Create a temporary directory for the generated code
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    // Configure generator with RESTful RPCs enabled
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    config.output_dir = output_path.to_path_buf();
    config.module_name = "test_bindings".to_string();

    let generator = CodeGenerator::new(config);

    // Create a YANG module with an RPC so HTTP abstractions are generated
    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-operation".to_string(),
            description: Some("Test RPC operation".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    // Generate the code
    let generated = generator.generate(&module).unwrap();

    // Find the module file
    let module_file = generated
        .files
        .iter()
        .find(|f| f.path.file_name().unwrap() == "test_bindings.rs")
        .expect("Module file should be generated");

    // Verify core traits and types are present
    assert!(
        module_file.content.contains("trait HttpTransport"),
        "HttpTransport trait should be generated"
    );
    assert!(
        module_file.content.contains("trait RequestInterceptor"),
        "RequestInterceptor trait should be generated"
    );
    assert!(
        module_file.content.contains("struct HttpRequest"),
        "HttpRequest struct should be generated"
    );
    assert!(
        module_file.content.contains("struct HttpResponse"),
        "HttpResponse struct should be generated"
    );
    assert!(
        module_file.content.contains("struct RestconfClient"),
        "RestconfClient struct should be generated"
    );
    assert!(
        module_file.content.contains("enum RpcError"),
        "RpcError enum should be generated"
    );

    // Verify transport adapters are feature-gated
    assert!(
        module_file
            .content
            .contains("#[cfg(feature = \"reqwest-client\")]"),
        "Reqwest adapter should be feature-gated"
    );
    assert!(
        module_file
            .content
            .contains("#[cfg(feature = \"hyper-client\")]"),
        "Hyper adapter should be feature-gated"
    );
}
