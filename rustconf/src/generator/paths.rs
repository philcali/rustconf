//! Path generation module for RESTCONF URL paths.
//!
//! This module handles the generation of URL path helper functions for
//! RESTCONF operations, including path construction and key encoding.

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{YangModule, Container, List};

/// Generator for RESTCONF URL path helpers.
pub struct PathGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> PathGenerator<'a> {
    /// Create a new path generator with the given configuration.
    pub fn new(config: &'a GeneratorConfig) -> Self {
        Self { config }
    }
}
