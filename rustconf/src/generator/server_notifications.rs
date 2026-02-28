//! Server-side notification publisher generation.
//!
//! This module generates notification publisher types and methods for server-side
//! YANG notification support. It creates type-safe publisher structs that allow
//! servers to send notifications to subscribed clients.

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{Notification, YangModule};

/// Generator for server-side notification publishers.
#[allow(dead_code)]
pub struct ServerNotificationGenerator<'a> {
    config: &'a GeneratorConfig,
}

#[allow(dead_code)]
impl<'a> ServerNotificationGenerator<'a> {
    /// Create a new server notification generator with the given configuration.
    pub fn new(config: &'a GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate the notification publisher module.
    pub fn generate_notification_publisher(
        &self,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        if module.notifications.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::new();

        // Add use statements
        output.push_str("use std::sync::Arc;\n");
        output.push_str("use tokio::sync::RwLock;\n");
        output.push_str("use serde::{Deserialize, Serialize};\n");
        output.push_str("use super::types::*;\n");
        output.push('\n');

        // Generate notification data structures (reuse from types)
        output.push_str("/// Notification data types.\n");
        output.push_str("pub mod notifications {\n");
        output.push_str("    use super::*;\n");
        output.push('\n');

        for notification in &module.notifications {
            output.push_str(&self.generate_notification_type(notification, module)?);
            output.push('\n');
        }

        output.push_str("}\n\n");

        // Generate subscriber trait
        output.push_str(&self.generate_subscriber_trait()?);
        output.push('\n');

        // Generate notification publisher
        output.push_str(&self.generate_publisher_struct(module)?);
        output.push('\n');

        Ok(output)
    }

    /// Generate a notification data type.
    fn generate_notification_type(
        &self,
        notification: &Notification,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let notification_type_name = crate::generator::naming::to_type_name(&notification.name);
        let type_gen = crate::generator::types::TypeGenerator::new(self.config);

        // Generate rustdoc comment
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

    /// Generate the NotificationSubscriber trait.
    fn generate_subscriber_trait(&self) -> Result<String, GeneratorError> {
        let mut output = String::new();

        output.push_str("/// Trait for notification subscribers.\n");
        output.push_str("///\n");
        output.push_str("/// Implement this trait to receive notifications from the publisher.\n");
        output.push_str("#[async_trait::async_trait]\n");
        output.push_str("pub trait NotificationSubscriber: Send + Sync {\n");
        output.push_str("    /// Receive a notification.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `notification_name` - The name of the notification type\n");
        output.push_str("    /// * `payload` - The serialized notification payload as JSON\n");
        output.push_str("    async fn on_notification(\n");
        output.push_str("        &self,\n");
        output.push_str("        notification_name: &str,\n");
        output.push_str("        payload: Vec<u8>,\n");
        output.push_str("    ) -> Result<(), String>;\n");
        output.push_str("}\n");

        Ok(output)
    }

    /// Generate the NotificationPublisher struct.
    fn generate_publisher_struct(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();

        // Generate publisher struct
        output.push_str("/// Notification publisher for sending notifications to subscribers.\n");
        output.push_str("///\n");
        output.push_str(
            "/// This struct manages subscriber registration and notification delivery.\n",
        );
        if self.config.derive_debug {
            output.push_str("#[derive(Debug)]\n");
        }
        output.push_str("pub struct NotificationPublisher {\n");
        output.push_str("    subscribers: Arc<RwLock<Vec<Arc<dyn NotificationSubscriber>>>>,\n");
        output.push_str("}\n\n");

        // Generate implementation
        output.push_str("impl NotificationPublisher {\n");

        // Constructor
        output.push_str("    /// Create a new notification publisher.\n");
        output.push_str("    pub fn new() -> Self {\n");
        output.push_str("        Self {\n");
        output.push_str("            subscribers: Arc::new(RwLock::new(Vec::new())),\n");
        output.push_str("        }\n");
        output.push_str("    }\n\n");

        // Subscribe method
        output.push_str("    /// Register a new subscriber.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `subscriber` - The subscriber to register\n");
        output.push_str(
            "    pub async fn subscribe(&self, subscriber: Arc<dyn NotificationSubscriber>) {\n",
        );
        output.push_str("        let mut subs = self.subscribers.write().await;\n");
        output.push_str("        subs.push(subscriber);\n");
        output.push_str("    }\n\n");

        // Unsubscribe method (by count for simplicity)
        output.push_str("    /// Get the number of active subscribers.\n");
        output.push_str("    pub async fn subscriber_count(&self) -> usize {\n");
        output.push_str("        self.subscribers.read().await.len()\n");
        output.push_str("    }\n\n");

        // Clear all subscribers
        output.push_str("    /// Remove all subscribers.\n");
        output.push_str("    pub async fn clear_subscribers(&self) {\n");
        output.push_str("        let mut subs = self.subscribers.write().await;\n");
        output.push_str("        subs.clear();\n");
        output.push_str("    }\n\n");

        // Generate publish methods for each notification
        for notification in &module.notifications {
            output.push_str(&self.generate_publish_method(notification)?);
            output.push('\n');
        }

        output.push_str("}\n");

        // Generate Default impl
        output.push_str("\nimpl Default for NotificationPublisher {\n");
        output.push_str("    fn default() -> Self {\n");
        output.push_str("        Self::new()\n");
        output.push_str("    }\n");
        output.push_str("}\n");

        Ok(output)
    }

    /// Generate a publish method for a specific notification.
    fn generate_publish_method(
        &self,
        notification: &Notification,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let notification_type_name = crate::generator::naming::to_type_name(&notification.name);
        let method_name = format!(
            "publish_{}",
            crate::generator::naming::to_snake_case(&notification.name)
        );

        // Generate rustdoc
        if let Some(ref description) = notification.description {
            output.push_str(&format!(
                "    /// Publish {} notification.\n",
                notification.name
            ));
            output.push_str("    ///\n");
            for line in description.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    output.push_str(&format!("    /// {}\n", trimmed));
                }
            }
        } else {
            output.push_str(&format!(
                "    /// Publish {} notification.\n",
                notification.name
            ));
        }
        output.push_str("    ///\n");
        output.push_str("    /// # Arguments\n");
        output.push_str("    ///\n");
        output.push_str("    /// * `notification` - The notification data to publish\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Returns\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns Ok(()) if all subscribers were notified successfully.\n");
        output.push_str("    /// Individual subscriber failures are logged but do not cause the method to fail.\n");
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns an error only if serialization fails.\n");

        // Generate method signature
        output.push_str(&format!(
            "    pub async fn {}(&self, notification: notifications::{}) -> Result<(), String> {{\n",
            method_name, notification_type_name
        ));

        // Serialize notification
        output.push_str("        // Serialize notification to JSON according to YANG schema\n");
        output.push_str("        let payload = serde_json::to_vec(&notification)\n");
        output.push_str(
            "            .map_err(|e| format!(\"Failed to serialize notification: {}\", e))?;\n\n",
        );

        // Notify all subscribers
        output.push_str("        // Notify all subscribers concurrently\n");
        output.push_str("        let subscribers = self.subscribers.read().await;\n");
        output.push_str("        let notification_name = \"");
        output.push_str(&notification.name);
        output.push_str("\";\n\n");

        output.push_str("        // Deliver to all subscribers, handling failures gracefully\n");
        output.push_str("        let mut delivery_errors = Vec::new();\n");
        output.push_str("        for (idx, subscriber) in subscribers.iter().enumerate() {\n");
        output.push_str("            // Deliver to subscriber\n");
        output.push_str("            if let Err(e) = subscriber.on_notification(notification_name, payload.clone()).await {\n");
        output.push_str("                let error_msg = format!(\"Subscriber {} delivery failed: {}\", idx, e);\n");
        output.push_str("                eprintln!(\"{}\", error_msg);\n");
        output.push_str("                delivery_errors.push(error_msg);\n");
        output.push_str("            }\n");
        output.push_str("        }\n\n");

        output.push_str("        // Log summary if there were any failures\n");
        output.push_str("        if !delivery_errors.is_empty() {\n");
        output.push_str("            eprintln!(\"Notification delivery completed with {} failures out of {} subscribers\",\n");
        output.push_str("                delivery_errors.len(), subscribers.len());\n");
        output.push_str("        }\n\n");

        output.push_str("        Ok(())\n");
        output.push_str("    }\n");

        Ok(output)
    }
}
