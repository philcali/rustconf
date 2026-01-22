//! Error types for YANG parsing.

use std::io;
use thiserror::Error;

/// Errors that can occur during YANG parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Syntax error at {line}:{column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Semantic error: {message}")]
    SemanticError { message: String },

    #[error("Unresolved import: {module}")]
    UnresolvedImport { module: String },
}
