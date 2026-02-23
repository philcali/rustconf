//! Tests for RpcError generation (Task 1)

use crate::generator::{operations::OperationsGenerator, GeneratorConfig};

#[test]
fn test_rpc_error_contains_all_variants() {
    let config = GeneratorConfig::default();
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Check that all required variants are present (matching rustconf-runtime)
    assert!(error_code.contains("pub enum RpcError"));
    assert!(error_code.contains("TransportError(String)"));
    assert!(error_code.contains("SerializationError(String)"));
    assert!(error_code.contains("DeserializationError(String)"));
    assert!(error_code.contains("ValidationError(String)"));
    assert!(error_code.contains("HttpError { status_code: u16, message: String }"));
    assert!(error_code.contains("ConfigurationError(String)"));
    assert!(error_code.contains("NotImplemented"));
}

#[test]
fn test_rpc_error_display_implementation() {
    let config = GeneratorConfig::default();
    let ops_gen = OperationsGenerator::new(&config);

    let error_code = ops_gen.generate_rpc_error();

    // Check Display trait implementation (matching rustconf-runtime)
    assert!(error_code.contains("impl std::fmt::Display for RpcError"));
    assert!(error_code
        .contains("RpcError::TransportError(msg) => write!(f, \"Transport error: {}\", msg)"));
    assert!(error_code.contains(
        "RpcError::SerializationError(msg) => write!(f, \"Serialization error: {}\", msg)"
    ));
    assert!(error_code.contains(
        "RpcError::DeserializationError(msg) => write!(f, \"Deserialization error: {}\", msg)"
    ));
    assert!(error_code
        .contains("RpcError::ValidationError(msg) => write!(f, \"Validation error: {}\", msg)"));
    assert!(error_code.contains("RpcError::HttpError { status_code, message } => write!(f, \"HTTP error {}: {}\", status_code, message)"));
    assert!(error_code.contains(
        "RpcError::ConfigurationError(msg) => write!(f, \"Configuration error: {}\", msg)"
    ));
    assert!(
        error_code.contains("RpcError::NotImplemented => write!(f, \"Operation not implemented\")")
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
