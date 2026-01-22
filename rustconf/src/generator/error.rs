//! Error types for code generation.

use std::io;
use thiserror::Error;

/// Errors that can occur during code generation.
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature { feature: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}
