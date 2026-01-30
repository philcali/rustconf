//! Integration tests for reqwest transport adapter generation (Task 7)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_reqwest_adapter_generation() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let mut config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "reqwest_test".to_string(),
        ..Default::default()
    };

    // Enable RESTful RPC generation
    config.enable_restful_rpcs();

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
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "input-param".to_string(),
                description: Some("Input parameter".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "output-param".to_string(),
                description: Some("Output parameter".to_string()),
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

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_dir.join("reqwest_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify reqwest adapter module is generated with feature gate (Task 7.1)
    assert!(
        content.contains("#[cfg(feature = \"reqwest-client\")]"),
        "reqwest adapter should be feature-gated"
    );
    assert!(
        content.contains("pub mod reqwest_adapter"),
        "reqwest_adapter module should be generated"
    );

    // Verify ReqwestTransport struct is generated (Task 7.2)
    assert!(
        content.contains("pub struct ReqwestTransport"),
        "ReqwestTransport struct should be generated"
    );
    assert!(
        content.contains("client: reqwest::Client"),
        "ReqwestTransport should have reqwest::Client field"
    );

    // Verify constructor methods are generated (Task 7.3)
    assert!(
        content.contains("pub fn new() -> Self"),
        "ReqwestTransport::new() constructor should be generated"
    );
    assert!(
        content.contains("reqwest::Client::new()"),
        "new() should create default reqwest client"
    );
    assert!(
        content.contains("pub fn with_client(client: reqwest::Client) -> Self"),
        "ReqwestTransport::with_client() constructor should be generated"
    );

    // Verify HttpTransport trait implementation (Task 7.4)
    assert!(
        content.contains("#[async_trait]"),
        "HttpTransport implementation should use async_trait"
    );
    assert!(
        content.contains("impl HttpTransport for ReqwestTransport"),
        "ReqwestTransport should implement HttpTransport trait"
    );
    assert!(
        content.contains(
            "async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>"
        ),
        "execute method should be implemented"
    );

    // Verify HTTP method conversion
    assert!(
        content.contains("HttpMethod::GET => reqwest::Method::GET"),
        "Should convert HttpMethod::GET to reqwest::Method::GET"
    );
    assert!(
        content.contains("HttpMethod::POST => reqwest::Method::POST"),
        "Should convert HttpMethod::POST to reqwest::Method::POST"
    );
    assert!(
        content.contains("HttpMethod::PUT => reqwest::Method::PUT"),
        "Should convert HttpMethod::PUT to reqwest::Method::PUT"
    );
    assert!(
        content.contains("HttpMethod::DELETE => reqwest::Method::DELETE"),
        "Should convert HttpMethod::DELETE to reqwest::Method::DELETE"
    );
    assert!(
        content.contains("HttpMethod::PATCH => reqwest::Method::PATCH"),
        "Should convert HttpMethod::PATCH to reqwest::Method::PATCH"
    );

    // Verify request building
    assert!(
        content.contains("self.client.request(method, &request.url)"),
        "Should build reqwest request with method and URL"
    );
    assert!(
        content.contains("req_builder = req_builder.header(key, value)"),
        "Should add headers to request"
    );
    assert!(
        content.contains("req_builder = req_builder.body(body)"),
        "Should add body to request if present"
    );

    // Verify request execution
    assert!(
        content.contains("req_builder.send().await"),
        "Should execute request asynchronously"
    );

    // Verify error conversion (Task 7.4)
    assert!(
        content.contains("RpcError::TransportError(format!(\"HTTP request failed: {}\", e))"),
        "Should convert reqwest errors to RpcError::TransportError"
    );
    assert!(
        content
            .contains("RpcError::TransportError(format!(\"Failed to read response body: {}\", e))"),
        "Should convert body read errors to RpcError::TransportError"
    );

    // Verify response extraction
    assert!(
        content.contains("response.status().as_u16()"),
        "Should extract status code from response"
    );
    assert!(
        content.contains("response.headers()"),
        "Should extract headers from response"
    );
    assert!(
        content.contains("response.bytes().await"),
        "Should extract body bytes from response"
    );
    assert!(
        content.contains("Ok(HttpResponse {"),
        "Should return HttpResponse on success"
    );

    // Verify comprehensive documentation (Task 7.5)
    assert!(
        content.contains("/// HTTP transport implementation using reqwest."),
        "ReqwestTransport should have comprehensive documentation"
    );
    assert!(
        content.contains("/// # Feature Flag"),
        "Documentation should mention feature flag requirement"
    );
    assert!(
        content.contains("/// # Examples"),
        "Documentation should include examples"
    );
    assert!(
        content.contains("/// ## Basic usage with default client"),
        "Documentation should include basic usage example"
    );
    assert!(
        content.contains("/// ## Using with a custom reqwest client"),
        "Documentation should include custom client example"
    );
    assert!(
        content.contains("[dependencies]"),
        "Documentation should show how to enable feature in Cargo.toml"
    );
    assert!(
        content.contains("features = [\"reqwest-client\"]"),
        "Documentation should show reqwest-client feature"
    );

    // Verify the code structure is correct (basic syntax check)
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces should be balanced in generated code"
    );
}

#[test]
fn test_reqwest_adapter_not_generated_when_disabled() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "no_reqwest_test".to_string(),
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
    let generated_file = output_dir.join("no_reqwest_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify reqwest adapter is NOT generated when disabled
    assert!(
        !content.contains("pub mod reqwest_adapter"),
        "reqwest_adapter module should not be generated when enable_restful_rpcs is false"
    );
    assert!(
        !content.contains("pub struct ReqwestTransport"),
        "ReqwestTransport should not be generated when enable_restful_rpcs is false"
    );
}

#[test]
fn test_reqwest_adapter_respects_derive_config() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let mut config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "derive_test".to_string(),
        derive_debug: true,
        derive_clone: true,
        ..Default::default()
    };

    // Enable RESTful RPC generation
    config.enable_restful_rpcs();

    let generator = CodeGenerator::new(config);

    // Create a minimal YANG module
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
            name: "test-op".to_string(),
            description: None,
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    // Generate code
    let generated = generator.generate(&module).unwrap();
    generator.write_files(&generated).unwrap();

    // Read the generated file
    let generated_file = output_dir.join("derive_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Find the ReqwestTransport struct definition
    let reqwest_section = content
        .split("pub struct ReqwestTransport")
        .next()
        .expect("ReqwestTransport struct should exist");

    // Verify derives are applied when enabled
    assert!(
        reqwest_section.contains("#[derive(Debug, Clone)]")
            || reqwest_section.contains("#[derive(Clone, Debug)]"),
        "ReqwestTransport should derive Debug and Clone when enabled"
    );
}
