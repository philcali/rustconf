//! Integration tests for RestconfClient generation (Task 3)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_restconf_client_generation() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let mut config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "restconf_test".to_string(),
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
    let generated_file = output_dir.join("restconf_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify HTTP abstractions are generated
    assert!(
        content.contains("pub enum HttpMethod"),
        "HttpMethod enum should be generated"
    );
    assert!(
        content.contains("pub struct HttpRequest"),
        "HttpRequest struct should be generated"
    );
    assert!(
        content.contains("pub struct HttpResponse"),
        "HttpResponse struct should be generated"
    );
    assert!(
        content.contains("pub trait HttpTransport"),
        "HttpTransport trait should be generated"
    );
    assert!(
        content.contains("pub trait RequestInterceptor"),
        "RequestInterceptor trait should be generated"
    );

    // Verify RestconfClient struct is generated (Task 3.1)
    assert!(
        content.contains("pub struct RestconfClient<T: HttpTransport>"),
        "RestconfClient struct should be generated"
    );
    assert!(
        content.contains("base_url: String"),
        "RestconfClient should have base_url field"
    );
    assert!(
        content.contains("transport: T"),
        "RestconfClient should have transport field"
    );
    assert!(
        content.contains("interceptor: Option<Box<dyn RequestInterceptor>>"),
        "RestconfClient should have interceptor field"
    );

    // Verify RestconfClient implementation is generated
    assert!(
        content.contains("impl<T: HttpTransport> RestconfClient<T>"),
        "RestconfClient implementation should be generated"
    );

    // Verify constructor method is generated (Task 3.2)
    assert!(
        content.contains(
            "pub fn new(base_url: impl Into<String>, transport: T) -> Result<Self, RpcError>"
        ),
        "RestconfClient::new() constructor should be generated"
    );
    assert!(
        content.contains("if base_url.is_empty()"),
        "Constructor should validate empty base URL"
    );
    assert!(
        content.contains(
            r#"if !base_url.starts_with("http://") && !base_url.starts_with("https://")"#
        ),
        "Constructor should validate URL scheme"
    );

    // Verify with_interceptor method is generated (Task 3.3)
    assert!(
        content.contains("pub fn with_interceptor(mut self, interceptor: impl RequestInterceptor + 'static) -> Self"),
        "RestconfClient::with_interceptor() method should be generated"
    );
    assert!(
        content.contains("self.interceptor = Some(Box::new(interceptor))"),
        "with_interceptor should store the interceptor"
    );

    // Verify execute_request method is generated (Task 3.4)
    assert!(
        content.contains("pub(crate) async fn execute_request(&self, mut request: HttpRequest) -> Result<HttpResponse, RpcError>"),
        "RestconfClient::execute_request() method should be generated"
    );
    assert!(
        content.contains("interceptor.before_request(&mut request).await?"),
        "execute_request should call before_request hook"
    );
    assert!(
        content.contains("self.transport.execute(request).await?"),
        "execute_request should call transport.execute()"
    );
    assert!(
        content.contains("interceptor.after_response(&response).await?"),
        "execute_request should call after_response hook"
    );

    // Verify base_url getter is generated (Task 3.5)
    assert!(
        content.contains("pub(crate) fn base_url(&self) -> &str"),
        "RestconfClient::base_url() getter should be generated"
    );
    assert!(
        content.contains("&self.base_url"),
        "base_url getter should return reference to base_url field"
    );

    // Verify comprehensive documentation is present (Task 3.6)
    assert!(
        content.contains("/// RESTCONF client for executing RESTful RPC operations."),
        "RestconfClient should have comprehensive documentation"
    );
    assert!(
        content.contains("/// # Examples"),
        "RestconfClient documentation should include examples"
    );
    assert!(
        content.contains("/// ## Basic usage with reqwest transport"),
        "RestconfClient documentation should include basic usage example"
    );
    assert!(
        content.contains("/// ## Using with an interceptor for authentication"),
        "RestconfClient documentation should include interceptor example"
    );
    assert!(
        content.contains("/// ## Managing multiple devices"),
        "RestconfClient documentation should include multiple devices example"
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
fn test_restconf_client_not_generated_when_disabled() {
    // Create a temporary directory for generated code
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_dir.clone(),
        module_name: "no_restconf_test".to_string(),
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
    let generated_file = output_dir.join("no_restconf_test.rs");
    let content = fs::read_to_string(&generated_file).unwrap();

    // Verify RestconfClient is NOT generated when disabled
    assert!(
        !content.contains("pub struct RestconfClient"),
        "RestconfClient should not be generated when enable_restful_rpcs is false"
    );
    assert!(
        !content.contains("pub trait HttpTransport"),
        "HttpTransport should not be generated when enable_restful_rpcs is false"
    );
    assert!(
        !content.contains("pub trait RequestInterceptor"),
        "RequestInterceptor should not be generated when enable_restful_rpcs is false"
    );

    // Verify RpcError is still generated (it's always needed)
    assert!(
        content.contains("pub enum RpcError"),
        "RpcError should still be generated"
    );
}
