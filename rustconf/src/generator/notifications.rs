//! Notification generation module for YANG notifications.
//!
//! This module handles the generation of Rust types for YANG notification
//! definitions, which are used for event-driven communication in NETCONF/RESTCONF.

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{Notification, YangModule};

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

impl<'a> NotificationGenerator<'a> {
    /// Generate notification types module.
    pub fn generate_notifications(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();

        output.push_str("/// RESTCONF notification types.\n");
        output.push_str("pub mod notifications {\n");
        output.push_str("    use super::*;\n");
        output.push('\n');

        // Generate struct for each notification
        for notification in &module.notifications {
            output.push_str(&self.generate_notification_type(notification, module)?);
            output.push('\n');
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate a struct type for a notification.
    fn generate_notification_type(
        &self,
        notification: &Notification,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let notification_type_name = crate::generator::naming::to_type_name(&notification.name);
        let type_gen = crate::generator::types::TypeGenerator::new(self.config);

        // Generate rustdoc comment from notification description
        if let Some(ref description) = notification.description {
            output.push_str(&format!("    {}", self.generate_rustdoc(description)));
        } else {
            output.push_str(&format!(
                "    /// Notification payload for {}.\n",
                notification.name
            ));
        }

        // Generate derive attributes
        output.push_str(&format!("    {}", self.generate_derive_attributes()));

        // Generate struct definition
        output.push_str(&format!("    pub struct {} {{\n", notification_type_name));

        // Generate fields from notification data nodes
        for node in &notification.data_nodes {
            let field = type_gen.generate_field(node, module, None)?;
            // Add indentation for nested struct
            for line in field.lines() {
                output.push_str(&format!("    {}\n", line));
            }
        }

        output.push_str("    }\n");

        Ok(output)
    }

    /// Generate rustdoc comments from a YANG description.
    fn generate_rustdoc(&self, description: &str) -> String {
        let mut rustdoc = String::new();

        // Split description into lines and format as rustdoc comments
        for line in description.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                rustdoc.push_str("///\n");
            } else {
                rustdoc.push_str(&format!("/// {}\n", trimmed));
            }
        }

        rustdoc
    }

    /// Generate derive attributes based on configuration.
    fn generate_derive_attributes(&self) -> String {
        let mut derives = vec!["Serialize", "Deserialize"];

        if self.config.derive_debug {
            derives.insert(0, "Debug");
        }

        if self.config.derive_clone {
            derives.insert(if self.config.derive_debug { 1 } else { 0 }, "Clone");
        }

        format!("#[derive({})]\n", derives.join(", "))
    }
}
