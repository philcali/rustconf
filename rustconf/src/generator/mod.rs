//! Code generator module for transforming YANG AST into Rust code.

use std::path::PathBuf;

use crate::parser::YangModule;

pub mod config;
pub mod error;

pub use config::GeneratorConfig;
pub use error::GeneratorError;

/// Code generator that transforms YANG AST into Rust code.
pub struct CodeGenerator {
    #[allow(dead_code)]
    config: GeneratorConfig,
}

impl CodeGenerator {
    /// Create a new code generator with the given configuration.
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate Rust code from a YANG module.
    pub fn generate(&self, _module: &YangModule) -> Result<GeneratedCode, GeneratorError> {
        // TODO: Implement in task 7
        unimplemented!("Code generation will be implemented in task 7")
    }
}

/// Generated code output.
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub files: Vec<GeneratedFile>,
}

/// A single generated file.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}
