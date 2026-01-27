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
mod notifications;
mod operations;
mod paths;
mod types;

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
            let ops_gen = operations::OperationsGenerator::new(&self.config);
            content.push_str(&ops_gen.generate_rpc_error());
            content.push('\n');
            content.push_str(&ops_gen.generate_operations_module(module)?);
            content.push('\n');
        }

        // Generate notification types
        if !module.notifications.is_empty() {
            let notif_gen = notifications::NotificationGenerator::new(&self.config);
            content.push_str(&notif_gen.generate_notifications(module)?);
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
            Self::collect_validated_types_from_node(node, &mut types, &type_gen);
        }

        types.into_iter().collect()
    }

    /// Recursively collect validated types from a data node.
    fn collect_validated_types_from_node(
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
                    Self::collect_validated_types_from_node(child, types, type_gen);
                }
            }
            DataNode::List(list) => {
                for child in &list.children {
                    Self::collect_validated_types_from_node(child, types, type_gen);
                }
            }
            DataNode::Choice(choice) => {
                for case in &choice.cases {
                    for child in &case.data_nodes {
                        Self::collect_validated_types_from_node(child, types, type_gen);
                    }
                }
            }
            DataNode::Case(case) => {
                for child in &case.data_nodes {
                    Self::collect_validated_types_from_node(child, types, type_gen);
                }
            }
            _ => {}
        }
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
