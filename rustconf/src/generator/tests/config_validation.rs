//! Tests for configuration validation (Task 1)

use crate::generator::{GeneratorConfig, NamespaceMode};

#[test]
fn test_enable_restful_rpcs_builder() {
    let mut config = GeneratorConfig::default();
    assert!(!config.enable_restful_rpcs);

    config.enable_restful_rpcs();
    assert!(config.enable_restful_rpcs);
}

#[test]
fn test_restful_namespace_mode_builder() {
    let mut config = GeneratorConfig::default();
    assert_eq!(config.restful_namespace_mode, NamespaceMode::Enabled);

    config.restful_namespace_mode(NamespaceMode::Disabled);
    assert_eq!(config.restful_namespace_mode, NamespaceMode::Disabled);
}

#[test]
fn test_namespace_mode_default() {
    assert_eq!(NamespaceMode::default(), NamespaceMode::Enabled);
}

#[test]
fn test_config_validation_passes_when_restful_rpcs_enabled() {
    let mut config = GeneratorConfig::default();
    config.enable_restful_rpcs();
    config.restful_namespace_mode(NamespaceMode::Disabled);

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_fails_when_namespace_mode_set_without_restful_rpcs() {
    let mut config = GeneratorConfig::default();
    // Don't enable restful_rpcs
    config.restful_namespace_mode(NamespaceMode::Disabled);

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("enable_restful_rpcs"));
}

#[test]
fn test_config_validation_passes_with_default_namespace_mode() {
    let config = GeneratorConfig::default();
    // Default namespace mode should not trigger validation error
    assert!(config.validate().is_ok());
}

#[test]
fn test_builder_chaining() {
    let mut config = GeneratorConfig::default();
    config
        .enable_restful_rpcs()
        .restful_namespace_mode(NamespaceMode::Disabled);

    assert!(config.enable_restful_rpcs);
    assert_eq!(config.restful_namespace_mode, NamespaceMode::Disabled);
    assert!(config.validate().is_ok());
}
