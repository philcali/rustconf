//! Tests for modular server code generation.

use crate::generator::{CodeGenerator, GeneratorConfig};
use crate::parser::{
    Container, DataNode, Leaf, Notification, Rpc, TypeSpec, YangModule, YangVersion,
};

/// Helper to create a test YANG module with data nodes and RPCs.
fn create_test_module() -> YangModule {
    YangModule {
        name: "test-module".to_string(),
        namespace: "http://example.com/test".to_string(),
        prefix: "test".to_string(),
        yang_version: Some(YangVersion::V1_1),
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![DataNode::Container(Container {
            name: "config".to_string(),
            description: Some("Configuration container".to_string()),
            config: true,
            mandatory: false,
            children: vec![DataNode::Leaf(Leaf {
                name: "hostname".to_string(),
                description: Some("Device hostname".to_string()),
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: false,
                default: None,
                config: true,
            })],
        })],
        rpcs: vec![Rpc {
            name: "restart-device".to_string(),
            description: Some("Restart the device".to_string()),
            input: None,
            output: None,
        }],
        notifications: vec![],
    }
}

/// Helper to create a server-enabled config.
fn server_config() -> GeneratorConfig {
    GeneratorConfig {
        modular_output: true,
        enable_server_generation: true,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    }
}

#[test]
fn test_modular_server_generation_creates_server_files() {
    let config = server_config();
    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let result = generator.generate(&module);
    assert!(result.is_ok());

    let generated = result.unwrap();

    // Should have: mod.rs, types.rs, operations.rs, validation.rs,
    // + server/mod.rs, server/handlers.rs, server/stubs.rs, server/router.rs, server/registry.rs
    // = 9 files total
    assert_eq!(generated.file_count(), 9);

    // Verify server files exist
    let server_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/mod.rs"));
    assert!(server_mod.is_some(), "server/mod.rs should exist");

    let server_handlers = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/handlers.rs"));
    assert!(server_handlers.is_some(), "server/handlers.rs should exist");

    let server_stubs = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/stubs.rs"));
    assert!(server_stubs.is_some(), "server/stubs.rs should exist");

    let server_router = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/router.rs"));
    assert!(server_router.is_some(), "server/router.rs should exist");

    let server_registry = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/registry.rs"));
    assert!(server_registry.is_some(), "server/registry.rs should exist");
}

#[test]
fn test_server_mod_has_correct_module_declarations() {
    let config = server_config();
    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let generated = generator.generate(&module).unwrap();

    let server_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/mod.rs"))
        .unwrap();

    let content = &server_mod.content;
    assert!(content.contains("pub mod handlers;"));
    assert!(content.contains("pub mod stubs;"));
    assert!(content.contains("pub mod router;"));
    assert!(content.contains("pub mod registry;"));

    // No notifications in this module, so notifications module should not be declared
    assert!(!content.contains("pub mod notifications;"));

    // Check re-exports
    assert!(content.contains("pub use handlers::*;"));
    assert!(content.contains("pub use stubs::*;"));
    assert!(content.contains("pub use router::*;"));
    assert!(content.contains("pub use registry::*;"));
}

#[test]
fn test_top_level_mod_includes_server_module() {
    let config = server_config();
    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let generated = generator.generate(&module).unwrap();

    let top_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("mod.rs") && !f.path.to_string_lossy().contains("server"))
        .unwrap();

    let content = &top_mod.content;
    assert!(
        content.contains("pub mod server;"),
        "Top-level mod.rs should declare pub mod server"
    );
    assert!(
        content.contains("pub use server::*;"),
        "Top-level mod.rs should re-export server types"
    );
}

#[test]
fn test_server_mod_includes_notifications_when_present() {
    let mut module = create_test_module();
    module.notifications.push(Notification {
        name: "config-change".to_string(),
        description: Some("Configuration changed".to_string()),
        data_nodes: vec![DataNode::Leaf(Leaf {
            name: "change-type".to_string(),
            description: Some("Type of change".to_string()),
            type_spec: TypeSpec::String {
                length: None,
                pattern: None,
            },
            mandatory: false,
            default: None,
            config: false,
        })],
    });

    let config = server_config();
    let generator = CodeGenerator::new(config);

    let generated = generator.generate(&module).unwrap();

    // Should have 10 files (9 + server/notifications.rs)
    assert_eq!(generated.file_count(), 10);

    let server_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/mod.rs"))
        .unwrap();

    assert!(server_mod.content.contains("pub mod notifications;"));
    assert!(server_mod.content.contains("pub use notifications::*;"));

    // Verify notifications file exists and has header
    let notif_file = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/notifications.rs"));
    assert!(notif_file.is_some());
    assert!(notif_file
        .unwrap()
        .content
        .contains("// This file is automatically generated by rustconf."));
}

#[test]
fn test_server_files_use_configured_subdir() {
    let config = GeneratorConfig {
        modular_output: true,
        enable_server_generation: true,
        enable_validation: true,
        enable_restful_rpcs: true,
        server_output_subdir: "my_server".to_string(),
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let generated = generator.generate(&module).unwrap();

    // Server files should be under my_server/ not server/
    let server_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("my_server/mod.rs"));
    assert!(
        server_mod.is_some(),
        "Server mod.rs should be under configured subdir"
    );

    // Top-level mod.rs should use configured subdir name
    let top_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("mod.rs") && !f.path.to_string_lossy().contains("my_server"))
        .unwrap();
    assert!(top_mod.content.contains("pub mod my_server;"));
}

#[test]
fn test_no_server_files_when_disabled() {
    let config = GeneratorConfig {
        modular_output: true,
        enable_server_generation: false,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let generated = generator.generate(&module).unwrap();

    // Should only have client files: mod.rs, types.rs, operations.rs, validation.rs
    assert_eq!(generated.file_count(), 4);

    // No server files
    let server_files: Vec<_> = generated
        .files
        .iter()
        .filter(|f| f.path.to_string_lossy().contains("server"))
        .collect();
    assert!(server_files.is_empty());

    // Top-level mod.rs should NOT contain server module
    let top_mod = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("mod.rs"))
        .unwrap();
    assert!(!top_mod.content.contains("pub mod server;"));
}

#[test]
fn test_server_handler_file_has_auto_generated_header() {
    let config = server_config();
    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let generated = generator.generate(&module).unwrap();

    let handlers = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/handlers.rs"))
        .unwrap();
    assert!(handlers
        .content
        .contains("// This file is automatically generated by rustconf."));

    let stubs = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/stubs.rs"))
        .unwrap();
    assert!(stubs
        .content
        .contains("// This file is automatically generated by rustconf."));

    let router = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/router.rs"))
        .unwrap();
    assert!(router
        .content
        .contains("// This file is automatically generated by rustconf."));

    let registry = generated
        .files
        .iter()
        .find(|f| f.path.ends_with("server/registry.rs"))
        .unwrap();
    assert!(registry
        .content
        .contains("// This file is automatically generated by rustconf."));
}

#[test]
fn test_write_files_creates_server_subdirectory() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let output_path = temp_dir.path().join("generated");

    let config = GeneratorConfig {
        output_dir: output_path.clone(),
        modular_output: true,
        enable_server_generation: true,
        enable_validation: true,
        enable_restful_rpcs: true,
        ..Default::default()
    };

    let generator = CodeGenerator::new(config);
    let module = create_test_module();

    let generated = generator.generate(&module).unwrap();
    let result = generator.write_files(&generated);
    assert!(result.is_ok());

    // Verify server directory was created
    let server_dir = output_path.join("server");
    assert!(server_dir.exists(), "server/ directory should be created");
    assert!(server_dir.is_dir());

    // Verify server files were written
    assert!(server_dir.join("mod.rs").exists());
    assert!(server_dir.join("handlers.rs").exists());
    assert!(server_dir.join("stubs.rs").exists());
    assert!(server_dir.join("router.rs").exists());
    assert!(server_dir.join("registry.rs").exists());
}
