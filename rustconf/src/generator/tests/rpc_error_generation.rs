//! Tests for RpcError generation (Task 1)

use crate::generator::{operations::OperationsGenerator, GeneratorConfig};

#[test]
fn test_rpc_error_contains_all_variants() {
    let config = GeneratorConfig::default();
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Check that all required variants are present
    assert!(error_code.contains("pub enum RpcError"));
    assert!(error_code.contains("NetworkError(String)"));
    assert!(error_code.contains("ServerError { code: u16, message: String }"));
    assert!(error_code.contains("SerializationError(String)"));
    assert!(error_code.contains("InvalidInput(String)"));
    assert!(error_code.contains("NotImplemented"));

    // Check new variants for RESTful RPCs
    assert!(error_code.contains("TransportError(String)"));
    assert!(error_code.contains("DeserializationError(String)"));
    assert!(error_code.contains("Unauthorized(String)"));
    assert!(error_code.contains("NotFound(String)"));
    assert!(error_code.contains("UnknownError(String)"));
}

#[test]
fn test_rpc_error_display_implementation() {
    let config = GeneratorConfig::default();
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Check Display trait implementation
    assert!(error_code.contains("impl std::fmt::Display for RpcError"));
    assert!(
        error_code.contains("RpcError::NetworkError(msg) => write!(f, \"Network error: {}\", msg)")
    );
    assert!(error_code
        .contains("RpcError::TransportError(msg) => write!(f, \"Transport error: {}\", msg)"));
    assert!(error_code.contains(
        "RpcError::DeserializationError(msg) => write!(f, \"Deserialization error: {}\", msg)"
    ));
    assert!(
        error_code.contains("RpcError::Unauthorized(msg) => write!(f, \"Unauthorized: {}\", msg)")
    );
    assert!(error_code.contains("RpcError::NotFound(msg) => write!(f, \"Not found: {}\", msg)"));
    assert!(
        error_code.contains("RpcError::UnknownError(msg) => write!(f, \"Unknown error: {}\", msg)")
    );
}

#[test]
fn test_rpc_error_implements_std_error() {
    let config = GeneratorConfig::default();
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Check that std::error::Error trait is implemented
    assert!(error_code.contains("impl std::error::Error for RpcError"));
}

#[test]
fn test_rpc_error_derives_debug_and_clone() {
    let config = GeneratorConfig {
        derive_debug: true,
        derive_clone: true,
        ..Default::default()
    };
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Check that Debug and Clone are derived
    assert!(error_code.contains("#[derive(Debug, Clone)]"));
}

#[test]
fn test_rpc_error_respects_derive_config() {
    let config = GeneratorConfig {
        derive_debug: false,
        derive_clone: false,
        ..Default::default()
    };
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Should not have any derives when both are disabled
    assert!(!error_code.contains("#[derive(Debug, Clone)]"));
    assert!(!error_code.contains("#[derive(Debug)]"));
    assert!(!error_code.contains("#[derive(Clone)]"));
}
