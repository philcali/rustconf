//! Integration tests for hyper transport adapter generation (Task 8)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_hyper_adapter_generation() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let mut config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "hyper_test".to_string(),
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
    let generated_file = output_dir.join("hyper_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify hyper adapter module is generated with feature gate (Task 8.1)
    assert!(
        content.contains("#[cfg(feature = \"hyper-client\")]"),
        "hyper adapter should be feature-gated"
    );
    assert!(
        content.contains("pub mod hyper_adapter"),
        "hyper_adapter module should be generated"
    );

    // Verify HyperTransport struct is generated (Task 8.2)
    assert!(
        content.contains("pub struct HyperTransport"),
        "HyperTransport struct should be generated"
    );
    assert!(
        content.contains("client: hyper::Client<hyper::client::HttpConnector>"),
        "HyperTransport should have hyper::Client field"
    );

    // Verify constructor method is generated (Task 8.3)
    assert!(
        content.contains("pub fn new() -> Self"),
        "HyperTransport::new() constructor should be generated"
    );
    assert!(
        content.contains("hyper::Client::new()"),
        "new() should create default hyper client"
    );

    // Verify HttpTransport trait implementation (Task 8.4)
    assert!(
        content.contains("#[async_trait]"),
        "HttpTransport implementation should use async_trait"
    );
    assert!(
        content.contains("impl HttpTransport for HyperTransport"),
        "HyperTransport should implement HttpTransport trait"
    );
    assert!(
        content.contains(
            "async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>"
        ),
        "execute method should be implemented"
    );

    // Verify HTTP method conversion
    assert!(
        content.contains("HttpMethod::GET => hyper::Method::GET"),
        "Should convert HttpMethod::GET to hyper::Method::GET"
    );
    assert!(
        content.contains("HttpMethod::POST => hyper::Method::POST"),
        "Should convert HttpMethod::POST to hyper::Method::POST"
    );
    assert!(
        content.contains("HttpMethod::PUT => hyper::Method::PUT"),
        "Should convert HttpMethod::PUT to hyper::Method::PUT"
    );
    assert!(
        content.contains("HttpMethod::DELETE => hyper::Method::DELETE"),
        "Should convert HttpMethod::DELETE to hyper::Method::DELETE"
    );
    assert!(
        content.contains("HttpMethod::PATCH => hyper::Method::PATCH"),
        "Should convert HttpMethod::PATCH to hyper::Method::PATCH"
    );

    // Verify request building
    assert!(
        content.contains("hyper::Request::builder()"),
        "Should use hyper request builder"
    );
    assert!(
        content.contains(".method(method)"),
        "Should set method on request builder"
    );
    assert!(
        content.contains(".uri(&request.url)"),
        "Should set URI on request builder"
    );
    assert!(
        content.contains("req_builder = req_builder.header(key, value)"),
        "Should add headers to request"
    );
    assert!(
        content.contains("request.body.map(hyper::Body::from).unwrap_or_else(hyper::Body::empty)"),
        "Should handle optional body"
    );
    assert!(
        content.contains("req_builder.body(body)"),
        "Should build request with body"
    );

    // Verify request execution
    assert!(
        content.contains("self.client.request(req).await"),
        "Should execute request asynchronously"
    );

    // Verify error conversion (Task 8.4)
    assert!(
        content.contains("RpcError::TransportError(format!(\"Failed to build request: {}\", e))"),
        "Should convert request build errors to RpcError::TransportError"
    );
    assert!(
        content.contains("RpcError::TransportError(format!(\"HTTP request failed: {}\", e))"),
        "Should convert hyper errors to RpcError::TransportError"
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
        content.contains("hyper::body::to_bytes(response.into_body()).await"),
        "Should extract body bytes from response"
    );
    assert!(
        content.contains("Ok(HttpResponse {"),
        "Should return HttpResponse on success"
    );

    // Verify comprehensive documentation (Task 8.5)
    assert!(
        content.contains("/// HTTP transport implementation using hyper."),
        "HyperTransport should have comprehensive documentation"
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
        content.contains("[dependencies]"),
        "Documentation should show how to enable feature in Cargo.toml"
    );
    assert!(
        content.contains("features = [\"hyper-client\"]"),
        "Documentation should show hyper-client feature"
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

#[test]
fn test_hyper_adapter_respects_derive_config() {
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

    // Find the HyperTransport struct definition
    let hyper_section = content
        .split("pub struct HyperTransport")
        .next()
        .expect("HyperTransport struct should exist");

    // Verify derives are applied when enabled
    assert!(
        hyper_section.contains("#[derive(Debug, Clone)]")
            || hyper_section.contains("#[derive(Clone, Debug)]"),
        "HyperTransport should derive Debug and Clone when enabled"
    );
}
