//! Notification generation module for YANG notifications.
//!
//! This module handles the generation of Rust types for YANG notification
//! definitions, which are used for event-driven communication in NETCONF/RESTCONF.

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{YangModule, Notification};

/// Generator for YANG notification types.
pub struct NotificationGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> NotificationGenerator<'a> {
    /// Create a new notification generator with the given configuration.
    pub fn new(config: &'a GeneratorConfig) -> Self {
        Self { config }
    }
}
