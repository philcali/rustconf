//! Path generation module for RESTCONF URL paths.
//!
//! This module handles the generation of URL path helper functions for
//! RESTCONF operations, including path construction and key encoding.

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{Container, List, YangModule};

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

impl<'a> PathGenerator<'a> {
    /// Generate path helper function for a container.
    pub fn generate_container_path_helper(
        &self,
        container: &Container,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let function_name = format!(
            "{}_path",
            crate::generator::naming::to_field_name(&container.name)
        );

        output.push_str(&format!(
            "        /// Build the RESTCONF URL path for the {} container.\n",
            container.name
        ));
        output.push_str("        #[allow(dead_code)]\n");
        output.push_str(&format!("        fn {}() -> String {{\n", function_name));

        // Build the path: /restconf/data/{module}:{container}
        let path = if self.config.enable_namespace_prefixes {
            format!("/restconf/data/{}:{}", module.prefix, container.name)
        } else {
            format!("/restconf/data/{}", container.name)
        };

        output.push_str(&format!("            \"{}\".to_string()\n", path));
        output.push_str("        }\n");

        Ok(output)
    }

    /// Generate path helper functions for a list (collection and item paths).
    pub fn generate_list_path_helpers(
        &self,
        list: &List,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let function_prefix = crate::generator::naming::to_field_name(&list.name);

        // Generate collection path helper (for entire list)
        output.push_str(&format!(
            "        /// Build the RESTCONF URL path for the {} collection.\n",
            list.name
        ));
        output.push_str("        #[allow(dead_code)]\n");
        output.push_str(&format!(
            "        fn {}_path() -> String {{\n",
            function_prefix
        ));

        let collection_path = if self.config.enable_namespace_prefixes {
            format!("/restconf/data/{}:{}", module.prefix, list.name)
        } else {
            format!("/restconf/data/{}", list.name)
        };

        output.push_str(&format!(
            "            \"{}\".to_string()\n",
            collection_path
        ));
        output.push_str("        }\n\n");

        // Generate item path helper (for specific list item by key)
        let key_params = self.generate_list_key_params(list);

        output.push_str(&format!(
            "        /// Build the RESTCONF URL path for a specific {} item.\n",
            list.name
        ));
        output.push_str("        ///\n");
        output.push_str("        /// Keys are percent-encoded for URL safety.\n");
        output.push_str("        #[allow(dead_code)]\n");
        output.push_str(&format!(
            "        fn {}_item_path({}) -> String {{\n",
            function_prefix, key_params
        ));

        // Build the base path
        let base_path = if self.config.enable_namespace_prefixes {
            format!("/restconf/data/{}:{}", module.prefix, list.name)
        } else {
            format!("/restconf/data/{}", list.name)
        };

        output.push_str(&format!(
            "            let mut path = \"{}\".to_string();\n",
            base_path
        ));

        // Add key encoding for each key
        for key in &list.keys {
            let key_param = crate::generator::naming::to_field_name(key);
            output.push_str(&format!(
                "            path.push_str(&format!(\"={{}}=\", percent_encode(&{}.to_string())));\n",
                key_param
            ));
        }

        output.push_str("            path\n");
        output.push_str("        }\n");

        Ok(output)
    }

    /// Generate parameter list for list key fields.
    pub fn generate_list_key_params(&self, list: &List) -> String {
        let mut params = Vec::new();

        for key in &list.keys {
            // Find the key field in the list's children to get its type
            let key_type = self.find_key_type(key, &list.children);
            let param_name = crate::generator::naming::to_field_name(key);
            params.push(format!("{}: {}", param_name, key_type));
        }

        params.join(", ")
    }

    /// Find the type of a key field in a list's children.
    fn find_key_type(&self, key_name: &str, children: &[crate::parser::DataNode]) -> String {
        use crate::parser::DataNode;

        let type_gen = crate::generator::types::TypeGenerator::new(self.config);

        for child in children {
            if let DataNode::Leaf(leaf) = child {
                if leaf.name == key_name {
                    // Key fields are always mandatory
                    return type_gen.generate_leaf_type(&leaf.type_spec, true);
                }
            }
        }

        // Default to String if key type not found
        "String".to_string()
    }

    /// Generate key parameter names for list operations (comma-separated).
    pub fn generate_key_param_names(&self, list: &List) -> String {
        list.keys
            .iter()
            .map(|key| crate::generator::naming::to_field_name(key))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate percent encoding helper function in the operations module.
    pub fn generate_percent_encode_helper(&self) -> String {
        let mut output = String::new();

        output.push_str("    /// Percent-encode a string for use in URLs.\n");
        output.push_str("    ///\n");
        output
            .push_str("    /// This function encodes special characters according to RFC 3986.\n");
        output.push_str("    #[allow(dead_code)]\n");
        output.push_str("    fn percent_encode(s: &str) -> String {\n");
        output.push_str("        s.chars()\n");
        output.push_str("            .map(|c| match c {\n");
        output.push_str("                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),\n");
        output.push_str("                _ => format!(\"%{:02X}\", c as u8),\n");
        output.push_str("            })\n");
        output.push_str("            .collect()\n");
        output.push_str("    }\n\n");

        output
    }
}
