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

// Server generation configuration tests

#[test]
fn test_enable_server_generation_builder() {
    let mut config = GeneratorConfig::default();
    assert!(!config.enable_server_generation);

    config.enable_server_generation();
    assert!(config.enable_server_generation);
}

#[test]
fn test_server_output_subdir_builder() {
    let mut config = GeneratorConfig::default();
    assert_eq!(config.server_output_subdir, "server");

    config.server_output_subdir("custom_server");
    assert_eq!(config.server_output_subdir, "custom_server");
}

#[test]
fn test_server_generation_default_values() {
    let config = GeneratorConfig::default();
    assert!(!config.enable_server_generation);
    assert_eq!(config.server_output_subdir, "server");
}

#[test]
fn test_server_generation_requires_modular_output() {
    let mut config = GeneratorConfig::default();
    config.enable_server_generation();
    // modular_output is false by default

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("modular_output must be enabled"));
}

#[test]
fn test_server_generation_validation_passes_with_modular_output() {
    let mut config = GeneratorConfig::default();
    config.enable_server_generation();
    config.modular_output = true;

    assert!(config.validate().is_ok());
}

#[test]
fn test_server_output_subdir_cannot_be_empty() {
    let mut config = GeneratorConfig::default();
    config.enable_server_generation();
    config.modular_output = true;
    config.server_output_subdir = "".to_string();

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_server_output_subdir_rejects_path_separators() {
    let mut config = GeneratorConfig::default();
    config.enable_server_generation();
    config.modular_output = true;

    // Test forward slash
    config.server_output_subdir = "server/subdir".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid path characters"));

    // Test backslash
    config.server_output_subdir = "server\\subdir".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid path characters"));
}

#[test]
fn test_server_output_subdir_rejects_reserved_names() {
    let mut config = GeneratorConfig::default();
    config.enable_server_generation();
    config.modular_output = true;

    let reserved_names = [".", "..", "types", "operations", "validation"];
    for name in reserved_names {
        config.server_output_subdir = name.to_string();
        let result = config.validate();
        assert!(
            result.is_err(),
            "Expected error for reserved name: {}",
            name
        );
        assert!(result.unwrap_err().contains("reserved module names"));
    }
}

#[test]
fn test_server_generation_builder_chaining() {
    let mut config = GeneratorConfig::default();
    config
        .enable_server_generation()
        .server_output_subdir("my_server");
    config.modular_output = true;

    assert!(config.enable_server_generation);
    assert_eq!(config.server_output_subdir, "my_server");
    assert!(config.validate().is_ok());
}

#[test]
fn test_server_generation_validation_only_when_enabled() {
    // Server generation disabled, so invalid subdir should not cause error
    let config = GeneratorConfig {
        server_output_subdir: "".to_string(),
        ..Default::default()
    };

    assert!(config.validate().is_ok());
}
