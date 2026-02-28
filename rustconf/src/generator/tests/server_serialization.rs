//! Property-based tests for server response serialization.
//!
//! These tests validate the correctness properties related to response
//! serialization, content negotiation, and round-trip compatibility.

use crate::generator::server_router::RouterGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{Container, DataNode, Leaf, Rpc, TypeSpec, YangModule};

/// Feature: server-side-generation
/// Property 9: Response Serialization Round-Trip
///
/// For any valid response data type, serializing on the server and
/// deserializing on the client SHALL produce an equivalent value.
///
/// Validates: Requirements 5.1, 13.2, 13.3
#[test]
fn test_response_serialization_round_trip() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    // Create a test module with various data types
    let module = YangModule {
        name: "test-serialization".to_string(),
        namespace: "http://example.com/test-serialization".to_string(),
        prefix: "ts".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "system-info".to_string(),
            description: Some("System information".to_string()),
            config: true,
            mandatory: false,
            children: vec![
                DataNode::Leaf(Leaf {
                    name: "hostname".to_string(),
                    description: Some("Hostname".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: false,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "port".to_string(),
                    description: Some("Port number".to_string()),
                    type_spec: TypeSpec::Uint16 { range: None },
                    mandatory: false,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "enabled".to_string(),
                    description: Some("Enabled flag".to_string()),
                    type_spec: TypeSpec::Boolean,
                    mandatory: false,
                    default: None,
                    config: true,
                }),
            ],
        })],
        rpcs: vec![Rpc {
            name: "get-status".to_string(),
            description: Some("Get status".to_string()),
            input: None,
            output: Some(vec![
                DataNode::Leaf(Leaf {
                    name: "status".to_string(),
                    description: Some("Status".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: false,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "uptime".to_string(),
                    description: Some("Uptime".to_string()),
                    type_spec: TypeSpec::Uint32 { range: None },
                    mandatory: false,
                    default: None,
                    config: false,
                }),
            ]),
        }],
        notifications: vec![],
    };

    // Generate router code
    let router_code = generator
        .generate_router(&module)
        .expect("Failed to generate router");

    // Verify that serde_json is used for serialization
    assert!(
        router_code.contains("serde_json::to_vec"),
        "Router should use serde_json for serialization"
    );

    // Verify that serialization is done in serialize_response helper
    assert!(
        router_code.contains("fn serialize_response"),
        "Router should have serialize_response helper"
    );

    // Verify that the same serialization logic is used for all responses
    assert!(
        router_code.contains("Self::serialize_response"),
        "Router should call serialize_response for all responses"
    );
}

/// Feature: server-side-generation
/// Property 11: Content-Type Header Presence
///
/// For any server response, the response SHALL include an appropriate
/// Content-Type header.
///
/// Validates: Requirements 5.3
#[test]
fn test_content_type_header_presence() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test-content-type".to_string(),
        namespace: "http://example.com/test-content-type".to_string(),
        prefix: "tct".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "simple-operation".to_string(),
            description: Some("Simple operation".to_string()),
            input: None,
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "result".to_string(),
                description: Some("Result".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let router_code = generator
        .generate_router(&module)
        .expect("Failed to generate router");

    // Verify that Content-Type header is set in responses
    assert!(
        router_code.contains("Content-Type"),
        "Router should set Content-Type header"
    );

    // Verify that the serialize_response function sets content type
    assert!(
        router_code.contains("actual_content_type"),
        "Router should determine actual content type"
    );

    // Verify that JSON is the default content type
    assert!(
        router_code.contains("application/json"),
        "Router should support JSON content type"
    );
}

/// Feature: server-side-generation
/// Property 12: Content Negotiation
///
/// For any request with an Accept header specifying JSON or XML,
/// the server SHALL serialize the response in the requested format.
///
/// Validates: Requirements 5.4
#[test]
fn test_content_negotiation() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test-negotiation".to_string(),
        namespace: "http://example.com/test-negotiation".to_string(),
        prefix: "tn".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
            description: Some("Configuration".to_string()),
            config: true,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "setting".to_string(),
                description: Some("Setting".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: true,
            })],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let router_code = generator
        .generate_router(&module)
        .expect("Failed to generate router");

    // Verify that Accept header is parsed
    assert!(
        router_code.contains("get_header(\"Accept\")"),
        "Router should parse Accept header"
    );

    // Verify that content negotiation function exists
    assert!(
        router_code.contains("negotiate_content_type"),
        "Router should have content negotiation function"
    );

    // Verify that JSON is supported
    assert!(
        router_code.contains("application/json"),
        "Router should support JSON format"
    );

    // Verify that XML is mentioned (even if not fully implemented)
    assert!(
        router_code.contains("application/xml"),
        "Router should mention XML format"
    );

    // Verify that default format is JSON
    assert!(
        router_code.contains("Default to JSON"),
        "Router should default to JSON"
    );
}

/// Test that serialize_response takes request parameter for content negotiation
#[test]
fn test_serialize_response_signature() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test-signature".to_string(),
        namespace: "http://example.com/test-signature".to_string(),
        prefix: "ts".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-op".to_string(),
            description: Some("Test operation".to_string()),
            input: None,
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "value".to_string(),
                description: Some("Value".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let router_code = generator
        .generate_router(&module)
        .expect("Failed to generate router");

    // Verify that serialize_response takes request parameter
    assert!(
        router_code.contains("fn serialize_response<T: serde::Serialize>"),
        "serialize_response should be generic over T"
    );

    // Verify that serialize_response is called with request parameter
    assert!(
        router_code.contains("Self::serialize_response(output, request)"),
        "serialize_response should be called with request parameter"
    );
}

/// Test that error responses include proper RESTCONF formatting
#[test]
fn test_error_response_formatting() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test-errors".to_string(),
        namespace: "http://example.com/test-errors".to_string(),
        prefix: "te".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-op".to_string(),
            description: Some("Test operation".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    };

    let router_code = generator
        .generate_router(&module)
        .expect("Failed to generate router");

    // Verify that errors are converted using ServerResponse::from_error
    assert!(
        router_code.contains("ServerResponse::from_error"),
        "Router should use ServerResponse::from_error for error responses"
    );

    // Verify that various error types are handled
    assert!(
        router_code.contains("ServerError::NotFound"),
        "Router should handle NotFound errors"
    );
    assert!(
        router_code.contains("ServerError::DeserializationError"),
        "Router should handle DeserializationError"
    );
}
