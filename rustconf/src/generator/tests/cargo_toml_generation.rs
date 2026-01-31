//! Tests for Cargo.toml template generation.

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::YangModule;

#[test]
fn test_cargo_toml_generated_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    config.module_name = "test_bindings".to_string();

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module).unwrap();

    // Should generate 2 files: the module file and Cargo.toml
    assert_eq!(result.file_count(), 2);

    // Find the Cargo.toml file
    let cargo_toml = result
        .files
        .iter()
        .find(|f| f.path.file_name().unwrap() == "Cargo.toml")
        .expect("Cargo.toml should be generated");

    // Verify package name
    assert!(cargo_toml.content.contains("name = \"test_bindings\""));

    // Verify core dependencies
    assert!(cargo_toml
        .content
        .contains("serde = { version = \"1.0\", features = [\"derive\"] }"));
    assert!(cargo_toml.content.contains("serde_json = \"1.0\""));
    assert!(cargo_toml.content.contains("async-trait = \"0.1\""));
    assert!(cargo_toml.content.contains("urlencoding = \"2.1\""));

    // Verify optional dependencies
    assert!(cargo_toml
        .content
        .contains("reqwest = { version = \"0.11\", features = [\"json\"], optional = true }"));
    assert!(cargo_toml
        .content
        .contains("hyper = { version = \"0.14\", optional = true }"));
    assert!(cargo_toml
        .content
        .contains("hyper-tls = { version = \"0.5\", optional = true }"));

    // Verify features
    assert!(cargo_toml
        .content
        .contains("reqwest-client = [\"reqwest\"]"));
    assert!(cargo_toml
        .content
        .contains("hyper-client = [\"hyper\", \"hyper-tls\"]"));
}

#[test]
fn test_cargo_toml_not_generated_when_restful_rpcs_disabled() {
    let config = GeneratorConfig::default(); // RESTful RPCs disabled by default

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module).unwrap();

    // Should only generate 1 file: the module file
    assert_eq!(result.file_count(), 1);

    // Verify no Cargo.toml file
    let cargo_toml = result
        .files
        .iter()
        .find(|f| f.path.file_name().unwrap() == "Cargo.toml");

    assert!(cargo_toml.is_none());
}

#[test]
fn test_cargo_toml_has_correct_structure() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();

    let generator = CodeGenerator::new(config);

    let module = YangModule {
        name: "test-module".to_string(),
        namespace: "urn:test:module".to_string(),
        prefix: "test".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate(&module).unwrap();

    let cargo_toml = result
        .files
        .iter()
        .find(|f| f.path.file_name().unwrap() == "Cargo.toml")
        .expect("Cargo.toml should be generated");

    // Verify sections are present in correct order
    assert!(cargo_toml.content.contains("[package]"));
    assert!(cargo_toml.content.contains("[dependencies]"));
    assert!(cargo_toml.content.contains("[features]"));

    // Verify package section comes before dependencies
    let package_pos = cargo_toml.content.find("[package]").unwrap();
    let deps_pos = cargo_toml.content.find("[dependencies]").unwrap();
    let features_pos = cargo_toml.content.find("[features]").unwrap();

    assert!(package_pos < deps_pos);
    assert!(deps_pos < features_pos);

    // Verify edition is 2021
    assert!(cargo_toml.content.contains("edition = \"2021\""));

    // Verify version is 0.1.0
    assert!(cargo_toml.content.contains("version = \"0.1.0\""));
}
