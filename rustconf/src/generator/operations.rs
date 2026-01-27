//! Operations generation module for RESTCONF CRUD and RPC operations.
//!
//! This module handles the generation of RESTCONF operations including:
//! - CRUD operations (GET, POST, PUT, PATCH, DELETE) for containers and lists
//! - RPC function definitions and types
//! - Error types for operations

use crate::generator::{GeneratorConfig, GeneratorError};
use crate::parser::{Rpc, YangModule};

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

use crate::parser::DataNode;

impl<'a> OperationsGenerator<'a> {
    /// Generate RPC error type.
    pub fn generate_rpc_error(&self) -> String {
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
    pub fn generate_operations_module(
        &self,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let path_gen = crate::generator::paths::PathGenerator::new(self.config);

        output.push_str("/// RESTCONF operations.\n");
        output.push_str("pub mod operations {\n");
        output.push_str("    use super::*;\n");
        output.push('\n');

        // Generate percent encoding helper function
        output.push_str(&path_gen.generate_percent_encode_helper());

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
        node: &DataNode,
        module: &YangModule,
    ) -> Result<String, GeneratorError> {
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
        let type_name = crate::generator::naming::to_type_name(&container.name);
        let function_prefix = crate::generator::naming::to_field_name(&container.name);
        let path_gen = crate::generator::paths::PathGenerator::new(self.config);

        // Generate path helper function
        output.push_str(&path_gen.generate_container_path_helper(container, module)?);
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
        let type_name = crate::generator::naming::to_type_name(&list.name);
        let function_prefix = crate::generator::naming::to_field_name(&list.name);
        let path_gen = crate::generator::paths::PathGenerator::new(self.config);

        // Determine item type name (singular)
        let item_type_name = if type_name.ends_with('s') && type_name.len() > 1 {
            &type_name[..type_name.len() - 1]
        } else {
            &type_name
        };

        // Generate path helper functions
        output.push_str(&path_gen.generate_list_path_helpers(list, module)?);
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
        let key_params = path_gen.generate_list_key_params(list);

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
            path_gen.generate_key_param_names(list)
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
                path_gen.generate_key_param_names(list)
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
                path_gen.generate_key_param_names(list)
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
                path_gen.generate_key_param_names(list)
            ));
            output.push_str("            // TODO: Implement DELETE request to RESTCONF server\n");
            output
                .push_str("            unimplemented!(\"DELETE operation not yet implemented\")\n");
            output.push_str("        }\n\n");
        }

        Ok(output)
    }

    /// Generate input and output types for an RPC.
    fn generate_rpc_types(&self, rpc: &Rpc, module: &YangModule) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let rpc_type_name = crate::generator::naming::to_type_name(&rpc.name);
        let type_gen = crate::generator::types::TypeGenerator::new(self.config);

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
    fn generate_rpc_function(&self, rpc: &Rpc) -> Result<String, GeneratorError> {
        let mut output = String::new();
        let rpc_type_name = crate::generator::naming::to_type_name(&rpc.name);
        let function_name = crate::generator::naming::to_field_name(&rpc.name);

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
