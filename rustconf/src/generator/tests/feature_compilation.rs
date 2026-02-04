//! Tests for feature flag compilation.

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{Rpc, YangModule};
use tempfile::TempDir;

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
