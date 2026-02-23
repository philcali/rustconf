//! Test for error mapping logic in generated RESTful RPC functions (Task 5.3)

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};

#[test]
fn test_error_mapping_logic_in_generated_code() {
    // Test that error mapping logic is correctly generated in RESTful RPC functions
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
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
            description: Some("Test RPC operation".to_string()),
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
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify error mapping structure exists
    assert!(
        content.contains("match response.status_code {"),
        "Error mapping should use match on status_code"
    );

    // Verify 200-299 range attempts deserialization
    assert!(
        content.contains("200..=299 => {"),
        "Should handle 200-299 status codes"
    );
    assert!(
        content.contains("serde_json::from_slice(&response.body)"),
        "Should attempt deserialization for success status codes"
    );
    assert!(
        content.contains("RpcError::DeserializationError"),
        "Should map deserialization failures to DeserializationError"
    );

    // Verify all other status codes map to HttpError (matching rustconf-runtime)
    assert!(
        content.contains("_ => Err(RpcError::HttpError {"),
        "All error status codes should map to HttpError"
    );
    assert!(
        content.contains("status_code: response.status_code,"),
        "HttpError should include status code"
    );
    assert!(
        content.contains("message: String::from_utf8_lossy(&response.body).to_string()"),
        "HttpError should include response body as message"
    );
}

#[test]
fn test_error_mapping_with_no_output() {
    // Test error mapping for RPC with no output (should return Ok(()) for success)
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
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
            name: "reset".to_string(),
            description: Some("Reset operation".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let generated = generator.generate(&module).unwrap();
    let content = &generated.files[0].content;

    // Verify 200-299 returns Ok(()) when no output expected
    assert!(
        content.contains("200..=299 => {"),
        "Should handle 200-299 status codes"
    );
    assert!(
        content.contains("// Success - no output expected"),
        "Should have comment for no output case"
    );
    assert!(
        content.contains("Ok(())"),
        "Should return Ok(()) when no output expected"
    );

    // Verify error mappings use HttpError (matching rustconf-runtime)
    assert!(
        content.contains("_ => Err(RpcError::HttpError {"),
        "All error status codes should map to HttpError"
    );
}

#[test]
fn test_all_rpc_error_variants_exist() {
    // Verify all required RpcError variants are generated (matching rustconf-runtime)
    let config = GeneratorConfig::default();
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

    // Verify all error variants used in error mapping exist (matching rustconf-runtime)
    assert!(
        content.contains("TransportError(String)"),
        "TransportError variant should exist"
    );
    assert!(
        content.contains("SerializationError(String)"),
        "SerializationError variant should exist"
    );
    assert!(
        content.contains("DeserializationError(String)"),
        "DeserializationError variant should exist"
    );
    assert!(
        content.contains("ValidationError(String)"),
        "ValidationError variant should exist"
    );
    assert!(
        content.contains("HttpError { status_code: u16, message: String }"),
        "HttpError variant should exist with status_code and message fields"
    );
    assert!(
        content.contains("ConfigurationError(String)"),
        "ConfigurationError variant should exist"
    );
    assert!(
        content.contains("NotImplemented"),
        "NotImplemented variant should exist"
    );
}
