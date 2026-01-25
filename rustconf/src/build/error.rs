//! Error types for build integration.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

use crate::generator::GeneratorError;
use crate::parser::ParseError;

/// Errors that can occur during build integration.
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Generator error: {0}")]
    GeneratorError(#[from] GeneratorError),

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

impl BuildError {
    /// Report this error through cargo's build script protocol.
    ///
    /// This method prints the error to stdout in a format that cargo will
    /// display to the user during the build process. It includes:
    /// - File paths and line numbers when available
    /// - Actionable error messages
    /// - Suggestions for fixing the error when applicable
    pub fn report_to_cargo(&self) {
        match self {
            BuildError::ParseError(parse_err) => {
                Self::report_parse_error(parse_err);
            }
            BuildError::GeneratorError(gen_err) => {
                Self::report_generator_error(gen_err);
            }
            BuildError::IoError(io_err) => {
                Self::report_io_error(io_err);
            }
            BuildError::ConfigurationError { message } => {
                Self::report_configuration_error(message);
            }
        }
    }

    /// Report a parse error with location information.
    fn report_parse_error(error: &ParseError) {
        println!("cargo:warning=rustconf: YANG parsing failed");

        match error {
            ParseError::SyntaxError {
                line,
                column,
                message,
            } => {
                println!(
                    "cargo:warning=  Syntax error at line {}, column {}",
                    line, column
                );
                println!("cargo:warning=  {}", message);
                println!("cargo:warning=");
                println!(
                    "cargo:warning=  Suggestion: Check the YANG syntax at the indicated location."
                );
                println!(
                    "cargo:warning=  Make sure all statements are properly terminated and nested."
                );
            }
            ParseError::SemanticError { message } => {
                println!("cargo:warning=  Semantic error: {}", message);
                println!("cargo:warning=");
                println!("cargo:warning=  Suggestion: Verify that all references are defined and");
                println!("cargo:warning=  constraints are well-formed.");
            }
            ParseError::UnresolvedImport { module } => {
                println!("cargo:warning=  Unresolved import: {}", module);
                println!("cargo:warning=");
                println!(
                    "cargo:warning=  Suggestion: Ensure the imported module '{}' is available",
                    module
                );
                println!("cargo:warning=  in one of the search paths. Use search_path() to add");
                println!("cargo:warning=  directories where YANG modules can be found.");
            }
            ParseError::IoError(io_err) => {
                println!("cargo:warning=  I/O error during parsing: {}", io_err);
                println!("cargo:warning=");
                println!(
                    "cargo:warning=  Suggestion: Check that the YANG file exists and is readable."
                );
            }
        }
    }

    /// Report a generator error.
    fn report_generator_error(error: &GeneratorError) {
        println!("cargo:warning=rustconf: Code generation failed");

        match error {
            GeneratorError::UnsupportedFeature { feature } => {
                println!("cargo:warning=  Unsupported feature: {}", feature);
                println!("cargo:warning=");
                println!("cargo:warning=  Suggestion: This YANG feature is not yet supported by rustconf.");
                println!(
                    "cargo:warning=  Consider simplifying your YANG model or filing an issue at:"
                );
                println!("cargo:warning=  https://github.com/your-repo/rustconf/issues");
            }
            GeneratorError::InvalidConfiguration { message } => {
                println!("cargo:warning=  Invalid configuration: {}", message);
                println!("cargo:warning=");
                println!("cargo:warning=  Suggestion: Check your RustconfBuilder configuration in build.rs.");
            }
            GeneratorError::IoError(io_err) => {
                println!("cargo:warning=  I/O error during generation: {}", io_err);
                println!("cargo:warning=");
                println!("cargo:warning=  Suggestion: Ensure the output directory is writable.");
            }
        }
    }

    /// Report an I/O error.
    fn report_io_error(error: &io::Error) {
        println!("cargo:warning=rustconf: I/O error");
        println!("cargo:warning=  {}", error);
        println!("cargo:warning=");

        match error.kind() {
            io::ErrorKind::NotFound => {
                println!("cargo:warning=  Suggestion: Check that all file paths are correct.");
            }
            io::ErrorKind::PermissionDenied => {
                println!("cargo:warning=  Suggestion: Ensure you have permission to read/write the files.");
            }
            io::ErrorKind::AlreadyExists => {
                println!("cargo:warning=  Suggestion: The file or directory already exists.");
            }
            _ => {
                println!("cargo:warning=  Suggestion: Check file system permissions and available space.");
            }
        }
    }

    /// Report a configuration error.
    fn report_configuration_error(message: &str) {
        println!("cargo:warning=rustconf: Configuration error");
        println!("cargo:warning=  {}", message);
        println!("cargo:warning=");
        println!(
            "cargo:warning=  Suggestion: Review your RustconfBuilder configuration in build.rs."
        );
        println!("cargo:warning=  Ensure all required fields are set and paths are valid.");
    }

    /// Create a BuildError with file context for better error reporting.
    pub fn with_file_context(self, file_path: PathBuf) -> BuildErrorWithContext {
        BuildErrorWithContext {
            error: self,
            file_path: Some(file_path),
        }
    }
}

/// A BuildError with additional file context for enhanced error reporting.
#[derive(Debug)]
pub struct BuildErrorWithContext {
    pub(crate) error: BuildError,
    file_path: Option<PathBuf>,
}

impl BuildErrorWithContext {
    /// Report this error with file context through cargo's build script protocol.
    pub fn report_to_cargo(&self) {
        if let Some(file_path) = &self.file_path {
            println!(
                "cargo:warning=rustconf: Error in file: {}",
                file_path.display()
            );
        }
        self.error.report_to_cargo();
    }

    /// Extract the underlying BuildError.
    pub fn into_inner(self) -> BuildError {
        self.error
    }
}

impl std::fmt::Display for BuildErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(file_path) = &self.file_path {
            write!(f, "Error in file {}: {}", file_path.display(), self.error)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

impl std::error::Error for BuildErrorWithContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
