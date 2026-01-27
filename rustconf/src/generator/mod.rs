//! Code generator module for transforming YANG AST into Rust code.

use std::fs;
use std::path::PathBuf;

use crate::parser::YangModule;

pub mod config;
pub mod error;
pub mod formatting;
pub mod naming;
pub mod validation;

// Sub-generators for modular code generation
mod types;
mod operations;
mod notifications;
mod paths;

pub use config::GeneratorConfig;
pub use error::GeneratorError;

/// Code generator that transforms YANG AST into Rust code.
pub struct CodeGenerator {
    config: GeneratorConfig,
}

impl CodeGenerator {
    /// Create a new code generator with the given configuration.
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate Rust code from a YANG module.
    pub fn generate(&self, module: &YangModule) -> Result<GeneratedCode, GeneratorError> {
        let mut files = Vec::new();

        // Generate the main module file
        let module_content = self.generate_module_content(module)?;
        let module_path = self
            .config
            .output_dir
            .join(format!("{}.rs", self.config.module_name));

        files.push(GeneratedFile {
            path: module_path,
            content: module_content,
        });

        Ok(GeneratedCode { files })
    }

    /// Generate the content for the main module file.
    fn generate_module_content(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut content = String::new();

        // Add file header comment with generation metadata
        content.push_str(&self.generate_file_header(module));
        content.push('\n');

        // Add use statements
        content.push_str(&self.generate_use_statements());
        content.push('\n');

        // Add module documentation
        if !module.namespace.is_empty() {
            content.push_str(&format!(
                "// Generated Rust bindings for YANG module: {}\n",
                module.name
            ));
            content.push_str(&format!("// Namespace: {}\n", module.namespace));
            content.push('\n');
        }

        // Generate ValidationError type if validation is enabled
        if self.config.enable_validation {
            content.push_str(&validation::generate_validation_error(
                self.config.derive_debug,
                self.config.derive_clone,
            ));
            content.push('\n');
        }

        // Collect all validated types needed
        let validated_types = self.collect_validated_types(module);

        // Generate validated type definitions
        for (type_name, type_spec) in validated_types {
            if let Some(validated_type) = validation::generate_validated_type(
                &type_name,
                &type_spec,
                self.config.derive_debug,
                self.config.derive_clone,
            ) {
                content.push_str(&validated_type);
                content.push('\n');
            }
        }

        // Create type generator
        let type_gen = types::TypeGenerator::new(&self.config);

        // Generate typedef type aliases
        for typedef in &module.typedefs {
            content.push_str(&type_gen.generate_typedef(typedef)?);
            content.push('\n');
        }

        // Generate type definitions from data nodes
        for data_node in &module.data_nodes {
            content.push_str(&type_gen.generate_data_node(data_node, module)?);
            content.push('\n');
        }

        // Generate RPC operations and CRUD operations
        if !module.rpcs.is_empty() || !module.data_nodes.is_empty() {
            content.push_str(&self.generate_rpc_error());
            content.push('\n');
            content.push_str(&self.generate_operations_module(module)?);
            content.push('\n');
        }

        // Generate notification types
        if !module.notifications.is_empty() {
            content.push_str(&self.generate_notifications(module)?);
            content.push('\n');
        }

        Ok(content)
    }

    /// Collect all validated types needed for the module.
    fn collect_validated_types(
        &self,
        module: &YangModule,
    ) -> Vec<(String, crate::parser::TypeSpec)> {
        use std::collections::HashMap;

        let mut types = HashMap::new();

        if !self.config.enable_validation {
            return Vec::new();
        }

        // Create type generator for validation checks
        let type_gen = types::TypeGenerator::new(&self.config);

        // Collect from typedefs
        for typedef in &module.typedefs {
            if type_gen.needs_validation(&typedef.type_spec) {
                let type_name = type_gen.get_validated_type_name(&typedef.type_spec);
                types.insert(type_name, typedef.type_spec.clone());
            }
        }

        // Collect from data nodes
        for node in &module.data_nodes {
            self.collect_validated_types_from_node(node, &mut types, &type_gen);
        }

        types.into_iter().collect()
    }

    /// Recursively collect validated types from a data node.
    fn collect_validated_types_from_node(
        &self,
        node: &crate::parser::DataNode,
        types: &mut std::collections::HashMap<String, crate::parser::TypeSpec>,
        type_gen: &types::TypeGenerator,
    ) {
        use crate::parser::DataNode;

        match node {
            DataNode::Leaf(leaf) => {
                if type_gen.needs_validation(&leaf.type_spec) {
                    let type_name = type_gen.get_validated_type_name(&leaf.type_spec);
                    types.insert(type_name, leaf.type_spec.clone());
                }
            }
            DataNode::Container(container) => {
                for child in &container.children {
                    self.collect_validated_types_from_node(child, types, type_gen);
                }
            }
            DataNode::List(list) => {
                for child in &list.children {
                    self.collect_validated_types_from_node(child, types, type_gen);
                }
            }
            DataNode::Choice(choice) => {
                for case in &choice.cases {
                    for child in &case.data_nodes {
                        self.collect_validated_types_from_node(child, types, type_gen);
                    }
                }
            }
            DataNode::Case(case) => {
                for child in &case.data_nodes {
                    self.collect_validated_types_from_node(child, types, type_gen);
                }
            }
            _ => {}
        }
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

    /// Get the JSON field name for a YANG node, with optional namespace prefix.
    ///
    /// For RESTCONF JSON compliance (RFC 8040), field names can be prefixed with
    /// the module prefix when namespace prefixes are enabled.
    fn get_json_field_name(&self, yang_name: &str, module: &YangModule) -> String {
        if self.config.enable_namespace_prefixes {
            format!("{}:{}", module.prefix, yang_name)
        } else {
            yang_name.to_string()
        }
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

    /// Generate file header comment with metadata.
    fn generate_file_header(&self, module: &YangModule) -> String {
        let mut header = String::new();

        header.push_str("// This file is automatically generated by rustconf.\n");
        header.push_str("// DO NOT EDIT MANUALLY.\n");
        header.push_str("//\n");
        header.push_str(&format!("// Source YANG module: {}\n", module.name));
        header.push_str(&format!("// Namespace: {}\n", module.namespace));
        header.push_str(&format!("// Prefix: {}\n", module.prefix));

        if let Some(ref version) = module.yang_version {
            let version_str = match version {
                crate::parser::YangVersion::V1_0 => "1.0",
                crate::parser::YangVersion::V1_1 => "1.1",
            };
            header.push_str(&format!("// YANG version: {}\n", version_str));
        }

        header.push_str(&format!(
            "// Generated at: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        header
    }

    /// Generate use statements for the module.
    fn generate_use_statements(&self) -> String {
        let mut uses = String::new();

        // Always include serde for serialization
        uses.push_str("use serde::{Deserialize, Serialize};\n");

        // Add XML support if enabled
        if self.config.enable_xml {
            uses.push_str("#[cfg(feature = \"xml\")]\n");
            uses.push_str("use serde_xml_rs;\n");
        }

        uses
    }

    /// Generate RPC error type.
    fn generate_rpc_error(&self) -> String {
        let mut output = String::new();

        output.push_str("/// Error type for RPC operations.\n");

        let mut derives = vec![];
        if self.config.derive_debug {
            derives.push("Debug");
        }
        if self.config.derive_clone {
            derives.push("Clone");
        }
        output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

        output.push_str("pub enum RpcError {\n");
        output.push_str("    /// Network or communication error.\n");
        output.push_str("    NetworkError(String),\n");
        output.push_str("    /// Server returned an error response.\n");
        output.push_str("    ServerError { code: u16, message: String },\n");
        output.push_str("    /// Serialization or deserialization error.\n");
        output.push_str("    SerializationError(String),\n");
        output.push_str("    /// Invalid input parameters.\n");
        output.push_str("    InvalidInput(String),\n");
        output.push_str("    /// Operation not implemented.\n");
        output.push_str("    NotImplemented,\n");
        output.push_str("}\n\n");

        output.push_str("impl std::fmt::Display for RpcError {\n");
        output
            .push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        output.push_str("        match self {\n");
        output.push_str(
            "            RpcError::NetworkError(msg) => write!(f, \"Network error: {}\", msg),\n",
        );
        output.push_str("            RpcError::ServerError { code, message } => write!(f, \"Server error {}: {}\", code, message),\n");
        output.push_str("            RpcError::SerializationError(msg) => write!(f, \"Serialization error: {}\", msg),\n");
        output.push_str(
            "            RpcError::InvalidInput(msg) => write!(f, \"Invalid input: {}\", msg),\n",
        );
        output.push_str(
            "            RpcError::NotImplemented => write!(f, \"Operation not implemented\"),\n",
        );
        output.push_str("        }\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");

        output.push_str("impl std::error::Error for RpcError {}\n");

        output
    }

    /// Generate operations module (RPC and CRUD operations).
    fn generate_operations_module(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();

        output.push_str("/// RESTCONF operations.\n");
        output.push_str("pub mod operations {\n");
        output.push_str("    use super::*;\n");
        output.push('\n');

        // Generate percent encoding helper function
        output.push_str(&self.generate_percent_encode_helper());

        // Generate input/output types and functions for each RPC
        if !module.rpcs.is_empty() {
            for rpc in &module.rpcs {
                let types = self.generate_rpc_types(rpc, module)?;
                if !types.is_empty() {
                    output.push_str(&types);
                }
                output.push_str(&self.generate_rpc_function(rpc)?);
                output.push('\n');
            }
        }

        // Generate RESTCONF CRUD operations for data nodes
        if !module.data_nodes.is_empty() {
            output.push_str(&self.generate_crud_operations(module)?);
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate RESTCONF CRUD operations for data nodes.
    fn generate_crud_operations(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();

        output.push_str("    /// RESTCONF CRUD operations for data resources.\n");
        output.push_str("    pub mod crud {\n");
        output.push_str("        use super::*;\n");
        output.push('\n');

        // Generate CRUD operations for each top-level data node
        for node in &module.data_nodes {
            output.push_str(&self.generate_node_crud_operations(node, module)?);
        }

        output.push_str("    }\n");

        Ok(output)
    }

    /// Generate CRUD operations for a specific data node.
    fn generate_node_crud_operations(
        &self,
        node: &crate::parser::DataNode,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        use crate::parser::DataNode;

        match node {
            DataNode::Container(container) => {
                self.generate_container_crud_operations(container, module)
            }
            DataNode::List(list) => self.generate_list_crud_operations(list, module),
            DataNode::Leaf(_) => Ok(String::new()), // Top-level leaves are rare
            DataNode::LeafList(_) => Ok(String::new()),
            DataNode::Choice(_) => Ok(String::new()),
            DataNode::Case(_) => Ok(String::new()),
            DataNode::Uses(_) => Ok(String::new()),
        }
    }

    /// Generate CRUD operations for a container.
    fn generate_container_crud_operations(
        &self,
        container: &crate::parser::Container,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let type_name = naming::to_type_name(&container.name);
        let function_prefix = naming::to_field_name(&container.name);

        // Generate path helper function
        output.push_str(&self.generate_container_path_helper(container, module)?);
        output.push('\n');

        // Generate GET operation (always available for containers)
        output.push_str(&format!(
            "        /// Retrieve the {} container.\n",
            container.name
        ));
        output.push_str("        ///\n");
        output.push_str("        /// # Errors\n");
        output.push_str("        ///\n");
        output.push_str("        /// Returns an error if the operation fails.\n");
        output.push_str(&format!(
            "        pub async fn get_{}() -> Result<{}, RpcError> {{\n",
            function_prefix, type_name
        ));
        output.push_str(&format!(
            "            let _path = {}_path();\n",
            function_prefix
        ));
        output.push_str("            // TODO: Implement GET request to RESTCONF server\n");
        output.push_str("            unimplemented!(\"GET operation not yet implemented\")\n");
        output.push_str("        }\n\n");

        // Generate config-based operations (PUT, PATCH, DELETE) only if config is true
        if container.config {
            // PUT operation - replace entire container
            output.push_str(&format!(
                "        /// Replace the {} container.\n",
                container.name
            ));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn put_{}(_data: {}) -> Result<(), RpcError> {{\n",
                function_prefix, type_name
            ));
            output.push_str(&format!(
                "            let _path = {}_path();\n",
                function_prefix
            ));
            output.push_str("            // TODO: Implement PUT request to RESTCONF server\n");
            output.push_str("            unimplemented!(\"PUT operation not yet implemented\")\n");
            output.push_str("        }\n\n");

            // PATCH operation - partial update
            output.push_str(&format!(
                "        /// Partially update the {} container.\n",
                container.name
            ));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn patch_{}(_data: {}) -> Result<(), RpcError> {{\n",
                function_prefix, type_name
            ));
            output.push_str(&format!(
                "            let _path = {}_path();\n",
                function_prefix
            ));
            output.push_str("            // TODO: Implement PATCH request to RESTCONF server\n");
            output
                .push_str("            unimplemented!(\"PATCH operation not yet implemented\")\n");
            output.push_str("        }\n\n");

            // DELETE operation - remove container
            output.push_str(&format!(
                "        /// Delete the {} container.\n",
                container.name
            ));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn delete_{}() -> Result<(), RpcError> {{\n",
                function_prefix
            ));
            output.push_str(&format!(
                "            let _path = {}_path();\n",
                function_prefix
            ));
            output.push_str("            // TODO: Implement DELETE request to RESTCONF server\n");
            output
                .push_str("            unimplemented!(\"DELETE operation not yet implemented\")\n");
            output.push_str("        }\n\n");
        }

        Ok(output)
    }

    /// Generate CRUD operations for a list.
    fn generate_list_crud_operations(
        &self,
        list: &crate::parser::List,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let type_name = naming::to_type_name(&list.name);
        let function_prefix = naming::to_field_name(&list.name);

        // Determine item type name (singular)
        let item_type_name = if type_name.ends_with('s') && type_name.len() > 1 {
            &type_name[..type_name.len() - 1]
        } else {
            &type_name
        };

        // Generate path helper functions
        output.push_str(&self.generate_list_path_helpers(list, module)?);
        output.push('\n');

        // Generate GET operation for entire list
        output.push_str(&format!("        /// Retrieve all {} items.\n", list.name));
        output.push_str("        ///\n");
        output.push_str("        /// # Errors\n");
        output.push_str("        ///\n");
        output.push_str("        /// Returns an error if the operation fails.\n");
        output.push_str(&format!(
            "        pub async fn get_{}() -> Result<Vec<{}>, RpcError> {{\n",
            function_prefix, item_type_name
        ));
        output.push_str(&format!(
            "            let _path = {}_path();\n",
            function_prefix
        ));
        output.push_str("            // TODO: Implement GET request to RESTCONF server\n");
        output.push_str("            unimplemented!(\"GET operation not yet implemented\")\n");
        output.push_str("        }\n\n");

        // Generate key parameter types for operations that need them
        let key_params = self.generate_list_key_params(list);

        // GET operation for single item by key
        output.push_str(&format!(
            "        /// Retrieve a single {} item by key.\n",
            list.name
        ));
        output.push_str("        ///\n");
        output.push_str("        /// # Errors\n");
        output.push_str("        ///\n");
        output.push_str("        /// Returns an error if the operation fails.\n");
        output.push_str(&format!(
            "        pub async fn get_{}_by_key({}) -> Result<{}, RpcError> {{\n",
            function_prefix, key_params, item_type_name
        ));
        output.push_str(&format!(
            "            let _path = {}_item_path({});\n",
            function_prefix,
            self.generate_key_param_names(list)
        ));
        output.push_str("            // TODO: Implement GET request to RESTCONF server\n");
        output.push_str("            unimplemented!(\"GET operation not yet implemented\")\n");
        output.push_str("        }\n\n");

        // Generate config-based operations only if config is true
        if list.config {
            // POST operation - create new item
            output.push_str(&format!("        /// Create a new {} item.\n", list.name));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn create_{}(_data: {}) -> Result<(), RpcError> {{\n",
                function_prefix, item_type_name
            ));
            output.push_str(&format!(
                "            let _path = {}_path();\n",
                function_prefix
            ));
            output.push_str("            // TODO: Implement POST request to RESTCONF server\n");
            output.push_str("            unimplemented!(\"POST operation not yet implemented\")\n");
            output.push_str("        }\n\n");

            // PUT operation - replace item by key
            output.push_str(&format!(
                "        /// Replace a {} item by key.\n",
                list.name
            ));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn put_{}({}, _data: {}) -> Result<(), RpcError> {{\n",
                function_prefix, key_params, item_type_name
            ));
            output.push_str(&format!(
                "            let _path = {}_item_path({});\n",
                function_prefix,
                self.generate_key_param_names(list)
            ));
            output.push_str("            // TODO: Implement PUT request to RESTCONF server\n");
            output.push_str("            unimplemented!(\"PUT operation not yet implemented\")\n");
            output.push_str("        }\n\n");

            // PATCH operation - partial update by key
            output.push_str(&format!(
                "        /// Partially update a {} item by key.\n",
                list.name
            ));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn patch_{}({}, _data: {}) -> Result<(), RpcError> {{\n",
                function_prefix, key_params, item_type_name
            ));
            output.push_str(&format!(
                "            let _path = {}_item_path({});\n",
                function_prefix,
                self.generate_key_param_names(list)
            ));
            output.push_str("            // TODO: Implement PATCH request to RESTCONF server\n");
            output
                .push_str("            unimplemented!(\"PATCH operation not yet implemented\")\n");
            output.push_str("        }\n\n");

            // DELETE operation - remove item by key
            output.push_str(&format!(
                "        /// Delete a {} item by key.\n",
                list.name
            ));
            output.push_str("        ///\n");
            output.push_str("        /// # Errors\n");
            output.push_str("        ///\n");
            output.push_str("        /// Returns an error if the operation fails.\n");
            output.push_str(&format!(
                "        pub async fn delete_{}({}) -> Result<(), RpcError> {{\n",
                function_prefix, key_params
            ));
            output.push_str(&format!(
                "            let _path = {}_item_path({});\n",
                function_prefix,
                self.generate_key_param_names(list)
            ));
            output.push_str("            // TODO: Implement DELETE request to RESTCONF server\n");
            output
                .push_str("            unimplemented!(\"DELETE operation not yet implemented\")\n");
            output.push_str("        }\n\n");
        }

        Ok(output)
    }

    /// Generate parameter list for list key fields.
    fn generate_list_key_params(&self, list: &crate::parser::List) -> String {
        let mut params = Vec::new();

        for key in &list.keys {
            // Find the key field in the list's children to get its type
            let key_type = self.find_key_type(key, &list.children);
            let param_name = naming::to_field_name(key);
            params.push(format!("{}: {}", param_name, key_type));
        }

        params.join(", ")
    }

    /// Find the type of a key field in a list's children.
    fn find_key_type(&self, key_name: &str, children: &[crate::parser::DataNode]) -> String {
        use crate::parser::DataNode;

        let type_gen = types::TypeGenerator::new(&self.config);

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
    fn generate_key_param_names(&self, list: &crate::parser::List) -> String {
        list.keys
            .iter()
            .map(|key| naming::to_field_name(key))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate path helper function for a container.
    fn generate_container_path_helper(
        &self,
        container: &crate::parser::Container,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let function_name = format!("{}_path", naming::to_field_name(&container.name));

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
    fn generate_list_path_helpers(
        &self,
        list: &crate::parser::List,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let function_prefix = naming::to_field_name(&list.name);

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
            let key_param = naming::to_field_name(key);
            output.push_str(&format!(
                "            path.push_str(&format!(\"={{}}=\", percent_encode(&{}.to_string())));\n",
                key_param
            ));
        }

        output.push_str("            path\n");
        output.push_str("        }\n");

        Ok(output)
    }

    /// Generate percent encoding helper function in the operations module.
    fn generate_percent_encode_helper(&self) -> String {
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

    /// Generate input and output types for an RPC.
    fn generate_rpc_types(
        &self,
        rpc: &crate::parser::Rpc,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let rpc_type_name = naming::to_type_name(&rpc.name);
        let type_gen = types::TypeGenerator::new(&self.config);

        // Generate input type if RPC has input
        if let Some(ref input_nodes) = rpc.input {
            if !input_nodes.is_empty() {
                output.push_str(&format!("    /// Input parameters for {} RPC.\n", rpc.name));
                output.push_str(&format!("    {}", self.generate_derive_attributes()));
                output.push_str(&format!("    pub struct {}Input {{\n", rpc_type_name));

                // Generate fields from input nodes
                for node in input_nodes {
                    let field = type_gen.generate_field(node, module, None)?;
                    // Add indentation for nested struct
                    for line in field.lines() {
                        output.push_str(&format!("    {}\n", line));
                    }
                }

                output.push_str("    }\n\n");
            }
        }

        // Generate output type if RPC has output
        if let Some(ref output_nodes) = rpc.output {
            if !output_nodes.is_empty() {
                output.push_str(&format!("    /// Output result for {} RPC.\n", rpc.name));
                output.push_str(&format!("    {}", self.generate_derive_attributes()));
                output.push_str(&format!("    pub struct {}Output {{\n", rpc_type_name));

                // Generate fields from output nodes
                for node in output_nodes {
                    let field = type_gen.generate_field(node, module, None)?;
                    // Add indentation for nested struct
                    for line in field.lines() {
                        output.push_str(&format!("    {}\n", line));
                    }
                }

                output.push_str("    }\n\n");
            }
        }

        Ok(output)
    }

    /// Generate an async function for an RPC operation.
    fn generate_rpc_function(&self, rpc: &crate::parser::Rpc) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let rpc_type_name = naming::to_type_name(&rpc.name);
        let function_name = naming::to_field_name(&rpc.name);

        // Generate rustdoc comment from RPC description
        if let Some(ref description) = rpc.description {
            output.push_str(&format!("    {}", self.generate_rustdoc(description)));
        } else {
            output.push_str(&format!(
                "    /// Execute the {} RPC operation.\n",
                rpc.name
            ));
        }

        // Add error handling documentation
        output.push_str("    ///\n");
        output.push_str("    /// # Errors\n");
        output.push_str("    ///\n");
        output.push_str("    /// Returns an error if the RPC operation fails.\n");

        // Determine input parameter type
        let input_param = if let Some(ref input_nodes) = rpc.input {
            if !input_nodes.is_empty() {
                format!("input: {}Input", rpc_type_name)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Determine return type
        let return_type = if let Some(ref output_nodes) = rpc.output {
            if !output_nodes.is_empty() {
                format!("Result<{}Output, RpcError>", rpc_type_name)
            } else {
                "Result<(), RpcError>".to_string()
            }
        } else {
            "Result<(), RpcError>".to_string()
        };

        // Generate function signature
        if input_param.is_empty() {
            output.push_str(&format!(
                "    pub async fn {}() -> {} {{\n",
                function_name, return_type
            ));
        } else {
            output.push_str(&format!(
                "    pub async fn {}({}) -> {} {{\n",
                function_name, input_param, return_type
            ));
        }

        // Generate function body (placeholder implementation)
        output.push_str("        // TODO: Implement RPC call logic\n");
        output.push_str("        // This is a placeholder that should be replaced with actual RESTCONF client implementation\n");
        output.push_str("        unimplemented!(\"RPC operation not yet implemented\")\n");
        output.push_str("    }\n");

        Ok(output)
    }

    /// Generate notification types.
    fn generate_notifications(&self, module: &YangModule) -> Result<String, GeneratorError> {
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
        notification: &crate::parser::Notification,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let notification_type_name = naming::to_type_name(&notification.name);
        let type_gen = types::TypeGenerator::new(&self.config);

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

    /// Write generated files to the output directory.
    pub fn write_files(&self, generated: &GeneratedCode) -> Result<(), GeneratorError> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.config.output_dir)?;

        // Write each generated file
        for file in &generated.files {
            fs::write(&file.path, &file.content)?;
        }

        Ok(())
    }
}

/// Generated code output.
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub files: Vec<GeneratedFile>,
}

impl GeneratedCode {
    /// Get the total number of generated files.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get the total size of all generated content in bytes.
    pub fn total_size(&self) -> usize {
        self.files.iter().map(|f| f.content.len()).sum()
    }
}

/// A single generated file.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod rpc_tests;

#[cfg(test)]
mod rpc_integration_test;

#[cfg(test)]
mod notification_tests;

#[cfg(test)]
mod notification_integration_test;

#[cfg(test)]
mod crud_tests;

#[cfg(test)]
mod url_path_tests;

#[cfg(test)]
mod url_path_example_test;
