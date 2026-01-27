//! Operations generation module for RESTCONF CRUD and RPC operations.
//!
//! This module handles the generation of RESTCONF operations including:
//! - CRUD operations (GET, POST, PUT, PATCH, DELETE) for containers and lists
//! - RPC function definitions and types
//! - Error types for operations

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{YangModule, Rpc};

/// Generator for RESTCONF operations and RPC functions.
pub struct OperationsGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> OperationsGenerator<'a> {
    /// Create a new operations generator with the given configuration.
    pub fn new(config: &'a GeneratorConfig) -> Self {
        Self { config }
    }
}
