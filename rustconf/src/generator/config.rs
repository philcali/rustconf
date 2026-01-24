//! Configuration types for code generation.

use std::path::PathBuf;

/// Configuration for code generation.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Output directory for generated code.
    pub output_dir: PathBuf,

    /// Name of the generated module.
    pub module_name: String,

    /// Enable XML serialization support.
    pub enable_xml: bool,

    /// Enable validation in generated code.
    pub enable_validation: bool,

    /// Derive Debug trait for generated types.
    pub derive_debug: bool,

    /// Derive Clone trait for generated types.
    pub derive_clone: bool,

    /// Enable namespace prefixes in JSON field names for RESTCONF compliance.
    /// When enabled, field names will be prefixed with the module prefix (e.g., "prefix:field-name").
    pub enable_namespace_prefixes: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("generated"),
            module_name: "yang_bindings".to_string(),
            enable_xml: false,
            enable_validation: true,
            derive_debug: true,
            derive_clone: true,
            enable_namespace_prefixes: false,
        }
    }
}
