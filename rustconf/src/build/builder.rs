//! Builder API for rustconf build integration.

use std::path::PathBuf;

use super::BuildError;
use crate::generator::GeneratorConfig;

/// Builder for configuring and running rustconf code generation.
pub struct RustconfBuilder {
    yang_files: Vec<PathBuf>,
    search_paths: Vec<PathBuf>,
    output_dir: PathBuf,
    config: GeneratorConfig,
}

impl RustconfBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            yang_files: Vec::new(),
            search_paths: Vec::new(),
            output_dir: PathBuf::from(
                std::env::var("OUT_DIR").unwrap_or_else(|_| "generated".to_string()),
            ),
            config: GeneratorConfig::default(),
        }
    }

    /// Add a YANG file to process.
    pub fn yang_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.yang_files.push(path.into());
        self
    }

    /// Add a search path for resolving YANG imports.
    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Set the output directory for generated code.
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_dir = path.into();
        self.config.output_dir = self.output_dir.clone();
        self
    }

    /// Enable or disable XML serialization support.
    pub fn enable_xml(mut self, enable: bool) -> Self {
        self.config.enable_xml = enable;
        self
    }

    /// Enable or disable validation in generated code.
    pub fn enable_validation(mut self, enable: bool) -> Self {
        self.config.enable_validation = enable;
        self
    }

    /// Set the generated module name.
    pub fn module_name(mut self, name: impl Into<String>) -> Self {
        self.config.module_name = name.into();
        self
    }

    /// Generate Rust bindings from configured YANG files.
    pub fn generate(self) -> Result<(), BuildError> {
        // TODO: Implement in task 12
        unimplemented!("Build generation will be implemented in task 12")
    }
}

impl Default for RustconfBuilder {
    fn default() -> Self {
        Self::new()
    }
}
