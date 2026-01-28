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

    /// Enable or disable RESTful RPC generation.
    ///
    /// When enabled, generates functional HTTP client implementations for RPCs.
    /// When disabled (default), generates stub functions returning NotImplemented errors.
    pub fn enable_restful_rpcs(mut self, enable: bool) -> Self {
        if enable {
            self.config.enable_restful_rpcs();
        }
        self
    }

    /// Generate Rust bindings from configured YANG files.
    pub fn generate(self) -> Result<(), BuildError> {
        // Validate configuration
        if let Err(e) = self.validate_configuration() {
            e.report_to_cargo();
            return Err(e);
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
            match parser.parse_file(yang_file) {
                Ok(module) => modules.push(module),
                Err(e) => {
                    let build_error = BuildError::from(e);
                    let error_with_context = build_error.with_file_context(yang_file.clone());
                    error_with_context.report_to_cargo();
                    return Err(error_with_context.into_inner());
                }
            }
        }

        // Create code generator
        let generator = crate::generator::CodeGenerator::new(self.config);

        // Generate code for each module
        for module in &modules {
            match generator.generate(module) {
                Ok(generated) => {
                    // Write generated files to output directory
                    for file in &generated.files {
                        // Ensure parent directory exists
                        if let Some(parent) = file.path.parent() {
                            if let Err(e) = std::fs::create_dir_all(parent) {
                                let build_error = BuildError::from(e);
                                build_error.report_to_cargo();
                                return Err(build_error);
                            }
                        }

                        // Write the file
                        if let Err(e) = std::fs::write(&file.path, &file.content) {
                            let build_error = BuildError::from(e);
                            let error_with_context =
                                build_error.with_file_context(file.path.clone());
                            error_with_context.report_to_cargo();
                            return Err(error_with_context.into_inner());
                        }
                    }
                }
                Err(e) => {
                    let build_error = BuildError::from(e);
                    build_error.report_to_cargo();
                    return Err(build_error);
                }
            }
        }

        // Emit cargo:rerun-if-changed directives for all input files
        for yang_file in &self.yang_files {
            println!("cargo:rerun-if-changed={}", yang_file.to_string_lossy());
        }

        // Also emit directives for all loaded modules (imports)
        for module in parser.get_all_loaded_modules().values() {
            // Try to find the file path for this module
            for search_path in &self.search_paths {
                let module_path = search_path.join(format!("{}.yang", module.name));
                if module_path.exists() {
                    println!("cargo:rerun-if-changed={}", module_path.to_string_lossy());
                }
            }
        }

        Ok(())
    }

    /// Validate the builder configuration.
    fn validate_configuration(&self) -> Result<(), BuildError> {
        // 1. Validate required fields: at least one YANG file must be specified
        if self.yang_files.is_empty() {
            return Err(BuildError::ConfigurationError {
                message: "No YANG files specified. Use yang_file() to add at least one YANG file."
                    .to_string(),
            });
        }

        // 2. Check that all YANG files exist and are accessible
        for yang_file in &self.yang_files {
            if !yang_file.exists() {
                return Err(BuildError::ConfigurationError {
                    message: format!("YANG file does not exist: {}", yang_file.display()),
                });
            }

            if !yang_file.is_file() {
                return Err(BuildError::ConfigurationError {
                    message: format!("YANG file path is not a file: {}", yang_file.display()),
                });
            }

            // Check if file is readable by attempting to get metadata
            if let Err(e) = std::fs::metadata(yang_file) {
                return Err(BuildError::ConfigurationError {
                    message: format!("Cannot access YANG file {}: {}", yang_file.display(), e),
                });
            }
        }

        // 3. Check that all search paths exist and are accessible
        for search_path in &self.search_paths {
            if !search_path.exists() {
                return Err(BuildError::ConfigurationError {
                    message: format!("Search path does not exist: {}", search_path.display()),
                });
            }

            if !search_path.is_dir() {
                return Err(BuildError::ConfigurationError {
                    message: format!("Search path is not a directory: {}", search_path.display()),
                });
            }

            // Check if directory is readable
            if let Err(e) = std::fs::read_dir(search_path) {
                return Err(BuildError::ConfigurationError {
                    message: format!("Cannot access search path {}: {}", search_path.display(), e),
                });
            }
        }

        // 4. Validate output directory parent exists or can be created
        // We allow the output directory itself to not exist (we'll create it)
        // But we need at least one ancestor that exists
        if !self.output_dir.as_os_str().is_empty() {
            let mut ancestor = self.output_dir.as_path();
            let mut found_existing = false;

            // Walk up the directory tree to find an existing ancestor
            while let Some(parent) = ancestor.parent() {
                if parent.as_os_str().is_empty() {
                    // Reached root or relative path base
                    found_existing = true;
                    break;
                }
                if parent.exists() {
                    found_existing = true;
                    break;
                }
                ancestor = parent;
            }

            // If we have an absolute path that doesn't exist anywhere, that's an error
            if self.output_dir.is_absolute() && !found_existing {
                return Err(BuildError::ConfigurationError {
                    message: format!(
                        "Output directory path is not accessible: {}",
                        self.output_dir.display()
                    ),
                });
            }
        }

        // 5. Validate module name is a valid Rust identifier
        let module_name = &self.config.module_name;
        if module_name.is_empty() {
            return Err(BuildError::ConfigurationError {
                message: "Module name cannot be empty".to_string(),
            });
        }

        // Check if module name starts with a digit
        if module_name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
        {
            return Err(BuildError::ConfigurationError {
                message: format!("Module name '{}' cannot start with a digit", module_name),
            });
        }

        // Check if module name contains only valid characters (alphanumeric and underscore)
        if !module_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(BuildError::ConfigurationError {
                message: format!(
                    "Module name '{}' contains invalid characters. Only alphanumeric characters and underscores are allowed",
                    module_name
                ),
            });
        }

        // 6. Detect conflicting options
        // Check if XML is enabled but validation is disabled (potential issue)
        // This is more of a warning scenario, but we can document it
        // For now, no strict conflicts to detect

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
