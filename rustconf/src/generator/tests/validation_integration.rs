//! Integration tests for validation in server-side code generation.
//!
//! These tests verify that validation is properly integrated into the
//! request/response handling pipeline.

use crate::generator::server_router::RouterGenerator;
use crate::generator::types::TypeGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{
    Container, DataNode, Leaf, LengthConstraint, LengthRange, PatternConstraint, Range,
    RangeConstraint, Rpc, TypeSpec, YangModule, YangVersion,
};

#[test]
fn test_validation_in_request_deserialization() {
    let mut config = GeneratorConfig::default();
    config.enable_validation = true;

    let router_gen = RouterGenerator::new(&config);

    // Create a module with constrained types
    let module = YangModule {
        name: "test-validation".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: Some("Test RPC with validation".to_string()),
            input: Some(vec![DataNode::Leaf(Leaf {
                name: "port".to_string(),
                description: Some("Port number (1-65535)".to_string()),
                type_spec: TypeSpec::Uint16 {
                    range: Some(RangeConstraint {
                        ranges: vec![Range { min: 1, max: 65535 }],
                    }),
                },
                mandatory: true,
                default: None,
                config: true,
            })]),
            output: None,
        }],
        notifications: vec![],
    };

    let result = router_gen.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();

    // Verify validation error handling is present in deserialization
    assert!(code.contains("Request validation failed"));
    assert!(code.contains("ServerError::ValidationError"));

    // Verify the deserialize_body function checks for validation errors
    assert!(code.contains("outside allowed range"));
    assert!(code.contains("invalid length"));
    assert!(code.contains("does not match pattern"));
}

#[test]
fn test_validation_in_response_serialization() {
    let mut config = GeneratorConfig::default();
    config.enable_validation = true;

    let router_gen = RouterGenerator::new(&config);

    // Create a module with constrained output types
    let module = YangModule {
        name: "test-validation".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![Rpc {
            name: "test-rpc".to_string(),
            description: Some("Test RPC with validation".to_string()),
            input: None,
            output: Some(vec![DataNode::Leaf(Leaf {
                name: "status-code".to_string(),
                description: Some("HTTP status code (100-599)".to_string()),
                type_spec: TypeSpec::Uint16 {
                    range: Some(RangeConstraint {
                        ranges: vec![Range { min: 100, max: 599 }],
                    }),
                },
                mandatory: true,
                default: None,
                config: false,
            })]),
        }],
        notifications: vec![],
    };

    let result = router_gen.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();

    // Verify validation error handling is present in serialization
    assert!(code.contains("Response validation failed"));
    assert!(code.contains("ServerError::ValidationError"));

    // Verify the serialize_response function checks for validation errors
    let serialize_start = code
        .find("fn serialize_response")
        .expect("serialize_response not found");
    let remaining_code = &code[serialize_start..];
    let serialize_end = remaining_code
        .find("\n    }\n")
        .map(|pos| serialize_start + pos + 6)
        .unwrap_or(code.len());
    let serialize_section = &code[serialize_start..serialize_end];
    assert!(serialize_section.contains("outside allowed range"));
    assert!(serialize_section.contains("invalid length"));
    assert!(serialize_section.contains("does not match pattern"));
}

#[test]
fn test_validation_with_string_constraints() {
    let mut config = GeneratorConfig::default();
    config.enable_validation = true;

    let type_gen = TypeGenerator::new(&config);
    let router_gen = RouterGenerator::new(&config);

    // Create a module with string length and pattern constraints
    let module = YangModule {
        name: "test-validation".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
            description: Some("Configuration with validated strings".to_string()),
            config: true,
            mandatory: false,
            children: vec![
                DataNode::Leaf(Leaf {
                    name: "hostname".to_string(),
                    description: Some("Hostname (1-255 chars)".to_string()),
                    type_spec: TypeSpec::String {
                        length: Some(LengthConstraint {
                            lengths: vec![LengthRange { min: 1, max: 255 }],
                        }),
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
                DataNode::Leaf(Leaf {
                    name: "ip-address".to_string(),
                    description: Some("IPv4 address".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: Some(PatternConstraint {
                            pattern: r"^(\d{1,3}\.){3}\d{1,3}$".to_string(),
                        }),
                    },
                    mandatory: true,
                    default: None,
                    config: true,
                }),
            ],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    // Generate types to ensure validated types are created
    let container = match &module.data_nodes[0] {
        DataNode::Container(c) => c,
        _ => panic!("Expected container"),
    };
    let type_result = type_gen.generate_container(container, &module);
    assert!(type_result.is_ok());

    // Generate router to ensure validation is integrated
    let router_result = router_gen.generate_router(&module);
    assert!(router_result.is_ok());

    let router_code = router_result.unwrap();

    // Verify validation error handling for string constraints
    assert!(router_code.contains("invalid length"));
    assert!(router_code.contains("does not match pattern"));
}

#[test]
fn test_validation_error_status_codes() {
    let config = GeneratorConfig::default();
    let router_gen = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = router_gen.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();

    // Verify that validation errors in requests return 400
    let deserialize_start = code
        .find("fn deserialize_body")
        .expect("deserialize_body not found");
    let remaining_code = &code[deserialize_start..];
    let deserialize_end = remaining_code
        .find("\n    }\n")
        .map(|pos| deserialize_start + pos + 6)
        .unwrap_or(code.len());
    let deserialize_section = &code[deserialize_start..deserialize_end];
    assert!(deserialize_section.contains("ServerError::ValidationError"));
    assert!(deserialize_section.contains("Request validation failed"));

    // Verify that validation errors in responses return 500
    let serialize_start = code
        .find("fn serialize_response")
        .expect("serialize_response not found");
    let remaining_code2 = &code[serialize_start..];
    let serialize_end = remaining_code2
        .find("\n    }\n")
        .map(|pos| serialize_start + pos + 6)
        .unwrap_or(code.len());
    let serialize_section = &code[serialize_start..serialize_end];
    assert!(serialize_section.contains("ServerError::ValidationError"));
    assert!(serialize_section.contains("Response validation failed"));
}

#[test]
fn test_validation_preserves_handler_invocation_flow() {
    let mut config = GeneratorConfig::default();
    config.enable_validation = true;

    let router_gen = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "data".to_string(),
            description: None,
            config: true,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "value".to_string(),
                description: None,
                type_spec: TypeSpec::Uint32 {
                    range: Some(RangeConstraint {
                        ranges: vec![Range { min: 0, max: 100 }],
                    }),
                },
                mandatory: true,
                default: None,
                config: true,
            })],
        })],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = router_gen.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();

    // Verify the flow: deserialize (with validation) -> handler -> serialize (with validation)
    assert!(code.contains("deserialize_body"));
    assert!(code.contains("self.handler."));
    assert!(code.contains("serialize_response"));

    // Verify error handling doesn't skip handler invocation
    // The handler should only be called after successful deserialization
    let data_routing = code.find("fn route_data").expect("route_data not found");
    let data_section = &code[data_routing..];

    // Find PUT handler section
    if let Some(put_start) = data_section.find("HttpMethod::PUT") {
        let put_section = &data_section[put_start..put_start + 500];
        // Verify: deserialize -> handler call
        assert!(put_section.contains("deserialize_body"));
        assert!(put_section.contains("self.handler.put_"));
    }
}
