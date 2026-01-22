//! Error types for build integration.

use std::io;
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
