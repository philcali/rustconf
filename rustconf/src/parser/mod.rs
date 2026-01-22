//! YANG specification parser module.
//!
//! This module provides functionality to parse YANG 1.0 and 1.1 specification files
//! into an abstract syntax tree (AST).

use std::path::{Path, PathBuf};

pub mod ast;
pub mod error;

pub use ast::*;
pub use error::ParseError;

/// YANG parser with configurable search paths for module resolution.
pub struct YangParser {
    search_paths: Vec<PathBuf>,
}

impl YangParser {
    /// Create a new YANG parser with default settings.
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    /// Add a search path for resolving YANG module imports.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Parse a YANG file from the given path.
    pub fn parse_file(&self, _path: &Path) -> Result<YangModule, ParseError> {
        // TODO: Implement in task 2
        unimplemented!("YANG file parsing will be implemented in task 2")
    }

    /// Parse YANG content from a string.
    pub fn parse_string(&self, _content: &str, _filename: &str) -> Result<YangModule, ParseError> {
        // TODO: Implement in task 2
        unimplemented!("YANG string parsing will be implemented in task 2")
    }
}

impl Default for YangParser {
    fn default() -> Self {
        Self::new()
    }
}
