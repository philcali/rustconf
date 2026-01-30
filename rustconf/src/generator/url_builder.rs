//! URL builder for RESTCONF operations.
//!
//! This module provides utilities for constructing RESTCONF-compliant URLs
//! with proper namespace handling and URL encoding.

use crate::generator::config::NamespaceMode;

/// Helper struct for building RESTCONF operation URLs.
///
/// The `UrlBuilder` handles URL construction with proper namespace formatting
/// and URL encoding according to the configured namespace mode.
pub struct UrlBuilder {
    /// The namespace mode configuration.
    namespace_mode: NamespaceMode,
}

impl UrlBuilder {
    /// Create a new URL builder with the given namespace mode.
    ///
    /// # Arguments
    ///
    /// * `namespace_mode` - The namespace mode to use for URL generation
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf::generator::config::NamespaceMode;
    /// use rustconf::generator::url_builder::UrlBuilder;
    ///
    /// let builder = UrlBuilder::new(NamespaceMode::Enabled);
    /// ```
    pub fn new(namespace_mode: NamespaceMode) -> Self {
        Self { namespace_mode }
    }

    /// Build a RESTCONF operation URL.
    ///
    /// This method constructs a URL for RESTCONF RPC operations according to RFC 8040.
    /// The URL format depends on the configured namespace mode:
    ///
    /// - `NamespaceMode::Enabled`: `/restconf/operations/{module}:{operation}`
    /// - `NamespaceMode::Disabled`: `/restconf/operations/{operation}`
    ///
    /// Module and operation names are URL-encoded to handle special characters safely.
    /// The base URL's trailing slashes are normalized to prevent double-slash issues.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the RESTCONF server (e.g., "https://device.example.com")
    /// * `module_name` - The YANG module name (e.g., "interface-mgmt")
    /// * `operation_name` - The RPC operation name (e.g., "reset-interface")
    ///
    /// # Returns
    ///
    /// A fully constructed RESTCONF operation URL as a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustconf::generator::config::NamespaceMode;
    /// use rustconf::generator::url_builder::UrlBuilder;
    ///
    /// let builder = UrlBuilder::new(NamespaceMode::Enabled);
    /// let url = builder.build_operation_url(
    ///     "https://device.example.com",
    ///     "interface-mgmt",
    ///     "reset-interface"
    /// );
    /// assert_eq!(url, "https://device.example.com/restconf/operations/interface-mgmt:reset-interface");
    ///
    /// let builder_no_ns = UrlBuilder::new(NamespaceMode::Disabled);
    /// let url_no_ns = builder_no_ns.build_operation_url(
    ///     "https://device.example.com",
    ///     "interface-mgmt",
    ///     "reset-interface"
    /// );
    /// assert_eq!(url_no_ns, "https://device.example.com/restconf/operations/reset-interface");
    /// ```
    ///
    /// # URL Encoding
    ///
    /// Special characters in module and operation names are automatically URL-encoded:
    ///
    /// ```
    /// use rustconf::generator::config::NamespaceMode;
    /// use rustconf::generator::url_builder::UrlBuilder;
    ///
    /// let builder = UrlBuilder::new(NamespaceMode::Enabled);
    /// let url = builder.build_operation_url(
    ///     "https://device.example.com",
    ///     "my-module",
    ///     "operation with spaces"
    /// );
    /// assert_eq!(url, "https://device.example.com/restconf/operations/my-module:operation%20with%20spaces");
    /// ```
    pub fn build_operation_url(
        &self,
        base_url: &str,
        module_name: &str,
        operation_name: &str,
    ) -> String {
        // Normalize base URL by removing trailing slashes
        let base = base_url.trim_end_matches('/');

        match self.namespace_mode {
            NamespaceMode::Enabled => {
                // URL encode the module and operation names
                let encoded_module = urlencoding::encode(module_name);
                let encoded_operation = urlencoding::encode(operation_name);
                format!(
                    "{}/restconf/operations/{}:{}",
                    base, encoded_module, encoded_operation
                )
            }
            NamespaceMode::Disabled => {
                // URL encode only the operation name
                let encoded_operation = urlencoding::encode(operation_name);
                format!("{}/restconf/operations/{}", base, encoded_operation)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_operation_url_with_namespace_enabled() {
        let builder = UrlBuilder::new(NamespaceMode::Enabled);
        let url = builder.build_operation_url(
            "https://device.example.com",
            "interface-mgmt",
            "reset-interface",
        );
        assert_eq!(
            url,
            "https://device.example.com/restconf/operations/interface-mgmt:reset-interface"
        );
    }

    #[test]
    fn test_build_operation_url_with_namespace_disabled() {
        let builder = UrlBuilder::new(NamespaceMode::Disabled);
        let url = builder.build_operation_url(
            "https://device.example.com",
            "interface-mgmt",
            "reset-interface",
        );
        assert_eq!(
            url,
            "https://device.example.com/restconf/operations/reset-interface"
        );
    }

    #[test]
    fn test_build_operation_url_with_trailing_slash() {
        let builder = UrlBuilder::new(NamespaceMode::Enabled);
        let url = builder.build_operation_url(
            "https://device.example.com/",
            "interface-mgmt",
            "reset-interface",
        );
        assert_eq!(
            url,
            "https://device.example.com/restconf/operations/interface-mgmt:reset-interface"
        );
    }

    #[test]
    fn test_build_operation_url_with_multiple_trailing_slashes() {
        let builder = UrlBuilder::new(NamespaceMode::Enabled);
        let url = builder.build_operation_url(
            "https://device.example.com///",
            "interface-mgmt",
            "reset-interface",
        );
        assert_eq!(
            url,
            "https://device.example.com/restconf/operations/interface-mgmt:reset-interface"
        );
    }

    #[test]
    fn test_build_operation_url_with_special_characters() {
        let builder = UrlBuilder::new(NamespaceMode::Enabled);
        let url = builder.build_operation_url(
            "https://device.example.com",
            "my-module",
            "operation with spaces",
        );
        assert_eq!(
            url,
            "https://device.example.com/restconf/operations/my-module:operation%20with%20spaces"
        );
    }

    #[test]
    fn test_build_operation_url_with_unicode_characters() {
        let builder = UrlBuilder::new(NamespaceMode::Enabled);
        let url = builder.build_operation_url(
            "https://device.example.com",
            "module-名前",
            "operation-操作",
        );
        // URL encoding should handle Unicode characters
        assert!(url.contains("/restconf/operations/"));
        assert!(url.contains(":"));
    }

    #[test]
    fn test_build_operation_url_with_special_url_characters() {
        let builder = UrlBuilder::new(NamespaceMode::Enabled);
        let url = builder.build_operation_url(
            "https://device.example.com",
            "module&name",
            "operation?test",
        );
        // Special URL characters should be encoded
        assert!(url.contains("%26")); // & encoded
        assert!(url.contains("%3F")); // ? encoded
    }
}
