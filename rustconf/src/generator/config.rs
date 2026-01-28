//! Configuration types for code generation.

use std::path::PathBuf;

/// Namespace mode for RESTful RPC URL generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamespaceMode {
    /// Include YANG module namespace in URLs (default).
    /// Example: /restconf/operations/module:operation
    #[default]
    Enabled,

    /// Omit namespace from URLs.
    /// Example: /restconf/operations/operation
    Disabled,
}

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

    /// Enable RESTful RPC generation.
    /// When enabled, generates functional HTTP client implementations for RPCs.
    /// When disabled, generates stub functions returning NotImplemented errors.
    pub enable_restful_rpcs: bool,

    /// Namespace mode for RESTful RPC URL generation.
    /// Controls whether YANG module namespaces are included in generated URLs.
    pub restful_namespace_mode: NamespaceMode,
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
            enable_restful_rpcs: false,
            restful_namespace_mode: NamespaceMode::default(),
        }
    }
}

impl GeneratorConfig {
    /// Enable RESTful RPC generation.
    ///
    /// When enabled, generates functional HTTP client implementations for RPCs.
    /// When disabled (default), generates stub functions returning NotImplemented errors.
    pub fn enable_restful_rpcs(&mut self) -> &mut Self {
        self.enable_restful_rpcs = true;
        self
    }

    /// Set the namespace mode for RESTful RPC URL generation.
    ///
    /// Controls whether YANG module namespaces are included in generated URLs.
    ///
    /// # Arguments
    ///
    /// * `mode` - The namespace mode to use
    pub fn restful_namespace_mode(&mut self, mode: NamespaceMode) -> &mut Self {
        self.restful_namespace_mode = mode;
        self
    }

    /// Validate the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate(&self) -> Result<(), String> {
        // Validate that restful_namespace_mode is only set when enable_restful_rpcs is true
        if !self.enable_restful_rpcs && self.restful_namespace_mode != NamespaceMode::default() {
            return Err(
                "restful_namespace_mode can only be set when enable_restful_rpcs is true. \
                 Call enable_restful_rpcs() before setting restful_namespace_mode()."
                    .to_string(),
            );
        }

        Ok(())
    }
}
