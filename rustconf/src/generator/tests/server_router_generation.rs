//! Tests for server router generation.

use crate::generator::server_router::RouterGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{Container, DataNode, Leaf, Rpc, TypeSpec, YangModule, YangVersion};

#[test]
fn test_router_generation_basic() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let module = YangModule {
        name: "test-module".to_string(),
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

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("pub struct RestconfRouter"));
    assert!(code.contains("pub fn new"));
    assert!(code.contains("pub async fn route"));
}

#[test]
fn test_router_generation_with_rpc() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let rpc = Rpc {
        name: "restart-device".to_string(),
        description: Some("Restart the device".to_string()),
        input: Some(vec![DataNode::Leaf(Leaf {
            name: "delay".to_string(),
            description: None,
            type_spec: TypeSpec::Uint32 { range: None },
            mandatory: false,
            default: None,
            config: true,
        })]),
        output: Some(vec![DataNode::Leaf(Leaf {
            name: "success".to_string(),
            description: None,
            type_spec: TypeSpec::Boolean,
            mandatory: true,
            default: None,
            config: false,
        })]),
    };

    let module = YangModule {
        name: "device-management".to_string(),
        namespace: "http://example.com/device".to_string(),
        prefix: "dm".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![rpc],
        notifications: vec![],
    };

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("route_rpc"));
    assert!(code.contains("\"restart-device\""));
    assert!(code.contains("restart_device"));
}

#[test]
fn test_router_generation_with_container() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

    let container = Container {
        name: "interfaces".to_string(),
        description: Some("Network interfaces".to_string()),
        config: true,
        mandatory: false,
        children: vec![DataNode::Leaf(Leaf {
            name: "name".to_string(),
            description: None,
            type_spec: TypeSpec::String {
                length: None,
                pattern: None,
            },
            mandatory: true,
            default: None,
            config: true,
        })],
    };

    let module = YangModule {
        name: "network".to_string(),
        namespace: "http://example.com/network".to_string(),
        prefix: "net".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(container)],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("route_data"));
    assert!(code.contains("\"interfaces\""));
    assert!(code.contains("get_interfaces"));
    assert!(code.contains("put_interfaces"));
    assert!(code.contains("patch_interfaces"));
    assert!(code.contains("delete_interfaces"));
}

#[test]
fn test_router_includes_percent_decoding() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

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

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("fn percent_decode"));
    assert!(code.contains("from_str_radix"));
}

#[test]
fn test_router_includes_serialization_helpers() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

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

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    assert!(code.contains("fn deserialize_body"));
    assert!(code.contains("fn serialize_response"));
    assert!(code.contains("serde_json"));
}

#[test]
fn test_router_includes_validation_in_deserialization() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

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

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    // Verify validation error handling in deserialization
    assert!(code.contains("Request validation failed"));
    assert!(code.contains("outside allowed range"));
    assert!(code.contains("invalid length"));
    assert!(code.contains("does not match pattern"));
    assert!(code.contains("ServerError::ValidationError"));
}

#[test]
fn test_router_includes_validation_in_serialization() {
    let config = GeneratorConfig::default();
    let generator = RouterGenerator::new(&config);

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

    let result = generator.generate_router(&module);
    assert!(result.is_ok());

    let code = result.unwrap();
    // Verify validation error handling in serialization
    assert!(code.contains("Response validation failed"));
    assert!(code.contains("outside allowed range"));
    assert!(code.contains("invalid length"));
    assert!(code.contains("does not match pattern"));
    // Verify it checks for validation errors in serialize_response
    let serialize_fn_start = code
        .find("fn serialize_response")
        .expect("serialize_response not found");
    let serialize_fn_end = code[serialize_fn_start..]
        .find("\n    }")
        .expect("end of serialize_response not found")
        + serialize_fn_start;
    let serialize_fn = &code[serialize_fn_start..serialize_fn_end];
    assert!(serialize_fn.contains("ServerError::ValidationError"));
}
