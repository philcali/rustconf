//! Integration tests for hyper transport adapter generation (Task 8)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{Rpc, YangModule};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_hyper_adapter_not_generated_when_disabled() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "no_hyper_test".to_string(),
        enable_restful_rpcs: false, // Explicitly disabled
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);

    // Create a YANG module with an RPC
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

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_dir.join("no_hyper_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify hyper adapter is NOT generated when disabled
    assert!(
        !content.contains("pub mod hyper_adapter"),
        "hyper_adapter module should not be generated when enable_restful_rpcs is false"
    );
    assert!(
        !content.contains("pub struct HyperTransport"),
        "HyperTransport should not be generated when enable_restful_rpcs is false"
    );
}
