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
        // Validate configuration
        if self.yang_files.is_empty() {
            return Err(BuildError::ConfigurationError {
                message: "No YANG files specified. Use yang_file() to add at least one YANG file."
                    .to_string(),
            });
        }

        // Create YANG parser
        let mut parser = crate::parser::YangParser::new();

        // Add search paths
        for search_path in &self.search_paths {
            parser.add_search_path(search_path.clone());
        }

        // Parse all YANG files
        let mut modules = Vec::new();
        for yang_file in &self.yang_files {
            let module = parser.parse_file(yang_file)?;
            modules.push(module);
        }

        // Create code generator
        let generator = crate::generator::CodeGenerator::new(self.config);

        // Generate code for each module
        for module in &modules {
            let generated = generator.generate(module)?;

            // Write generated files to output directory
            for file in &generated.files {
                // Ensure parent directory exists
                if let Some(parent) = file.path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Write the file
                std::fs::write(&file.path, &file.content)?;
            }
        }

        // Emit cargo:rerun-if-changed directives for all input files
        for yang_file in &self.yang_files {
            println!(
                "cargo:rerun-if-changed={}",
                yang_file.to_string_lossy()
            );
        }

        // Also emit directives for all loaded modules (imports)
        for (_, module) in parser.get_all_loaded_modules() {
            // Try to find the file path for this module
            for search_path in &self.search_paths {
                let module_path = search_path.join(format!("{}.yang", module.name));
                if module_path.exists() {
                    println!(
                        "cargo:rerun-if-changed={}",
                        module_path.to_string_lossy()
                    );
                }
            }
        }

        Ok(())
    }
}

impl Default for RustconfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
