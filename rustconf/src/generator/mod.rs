//! Code generator module for transforming YANG AST into Rust code.

use std::fs;
use std::path::PathBuf;

use crate::parser::YangModule;

pub mod config;
pub mod error;
pub mod formatting;
pub mod naming;
pub mod validation;

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
                "//! Generated Rust bindings for YANG module: {}\n",
                module.name
            ));
            content.push_str(&format!("//! Namespace: {}\n", module.namespace));
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

        // Generate type definitions from data nodes
        for data_node in &module.data_nodes {
            content.push_str(&self.generate_data_node(data_node)?);
            content.push('\n');
        }

        Ok(content)
    }

    /// Generate code for a data node.
    fn generate_data_node(&self, node: &crate::parser::DataNode) -> Result<String, GeneratorError> {
        use crate::parser::DataNode;

        match node {
            DataNode::Container(container) => self.generate_container(container),
            DataNode::List(list) => self.generate_list(list),
            DataNode::Leaf(_) => Ok(String::new()), // Leaves are handled as struct fields
            DataNode::LeafList(_) => Ok(String::new()), // Will be implemented later
            DataNode::Choice(choice) => self.generate_choice(choice),
            DataNode::Case(_) => Ok(String::new()), // Cases are handled within choices
            DataNode::Uses(_) => Ok(String::new()), // Uses should be expanded during parsing
        }
    }

    /// Generate a Rust struct from a YANG container.
    fn generate_container(
        &self,
        container: &crate::parser::Container,
    ) -> Result<String, GeneratorError> {
        let mut output = String::new();

        // Generate rustdoc comment from YANG description
        if let Some(ref description) = container.description {
            output.push_str(&self.generate_rustdoc(description));
        }

        // Generate derive attributes
        output.push_str(&self.generate_derive_attributes());

        // Generate struct definition
        let type_name = naming::to_type_name(&container.name);
        output.push_str(&format!("pub struct {} {{\n", type_name));

        // Generate fields from child nodes
        for child in &container.children {
            output.push_str(&self.generate_struct_field(child)?);
        }

        output.push_str("}\n");

        // Recursively generate types for nested containers, lists, and choices
        for child in &container.children {
            match child {
                crate::parser::DataNode::Container(nested) => {
                    output.push('\n');
                    output.push_str(&self.generate_container(nested)?);
                }
                crate::parser::DataNode::List(nested) => {
                    output.push('\n');
                    output.push_str(&self.generate_list(nested)?);
                }
                crate::parser::DataNode::Choice(nested) => {
                    output.push('\n');
                    output.push_str(&self.generate_choice(nested)?);
                }
                _ => {}
            }
        }

        Ok(output)
    }

    /// Generate a Rust enum from a YANG choice.
    fn generate_choice(&self, choice: &crate::parser::Choice) -> Result<String, GeneratorError> {
        let mut output = String::new();

        // Generate rustdoc comment from YANG description
        if let Some(ref description) = choice.description {
            output.push_str(&self.generate_rustdoc(description));
        }

        // Generate derive attributes
        output.push_str(&self.generate_derive_attributes());

        // Add serde attribute for kebab-case serialization
        output.push_str("#[serde(rename_all = \"kebab-case\")]\n");

        // Generate enum definition
        let type_name = naming::to_type_name(&choice.name);
        output.push_str(&format!("pub enum {} {{\n", type_name));

        // Generate variants from cases
        for case in &choice.cases {
            // Add rustdoc comment for case if description exists
            if let Some(ref description) = case.description {
                output.push_str(&format!("    {}", self.generate_rustdoc(description)));
            }

            let variant_name = naming::to_type_name(&case.name);

            // Determine the variant type based on case contents
            if case.data_nodes.is_empty() {
                // Empty case - unit variant
                output.push_str(&format!("    {},\n", variant_name));
            } else if case.data_nodes.len() == 1 {
                // Single data node - check if it's a leaf or complex type
                match &case.data_nodes[0] {
                    crate::parser::DataNode::Leaf(leaf) => {
                        // Single leaf - use tuple variant with the leaf type
                        let leaf_type = self.generate_leaf_type(&leaf.type_spec, true);
                        output.push_str(&format!("    {}({}),\n", variant_name, leaf_type));
                    }
                    _ => {
                        // Complex type - use named struct variant
                        let case_type_name = format!("{}Data", variant_name);
                        output.push_str(&format!("    {}({}),\n", variant_name, case_type_name));
                    }
                }
            } else {
                // Multiple data nodes - use named struct variant
                let case_type_name = format!("{}Data", variant_name);
                output.push_str(&format!("    {}({}),\n", variant_name, case_type_name));
            }
        }

        output.push_str("}\n");

        // Generate struct types for cases with multiple or complex data nodes
        for case in &choice.cases {
            if case.data_nodes.len() > 1
                || (case.data_nodes.len() == 1
                    && !matches!(case.data_nodes[0], crate::parser::DataNode::Leaf(_)))
            {
                output.push('\n');
                output.push_str(&self.generate_case_struct(case)?);
            }
        }

        // Recursively generate types for nested containers and lists within cases
        for case in &choice.cases {
            for node in &case.data_nodes {
                match node {
                    crate::parser::DataNode::Container(nested) => {
                        output.push('\n');
                        output.push_str(&self.generate_container(nested)?);
                    }
                    crate::parser::DataNode::List(nested) => {
                        output.push('\n');
                        output.push_str(&self.generate_list(nested)?);
                    }
                    _ => {}
                }
            }
        }

        Ok(output)
    }

    /// Generate a struct for a case with multiple data nodes.
    fn generate_case_struct(&self, case: &crate::parser::Case) -> Result<String, GeneratorError> {
        let mut output = String::new();

        // Generate rustdoc comment from case description
        if let Some(ref description) = case.description {
            output.push_str(&self.generate_rustdoc(description));
        }

        // Generate derive attributes
        output.push_str(&self.generate_derive_attributes());

        // Generate struct definition
        let variant_name = naming::to_type_name(&case.name);
        let struct_name = format!("{}Data", variant_name);
        output.push_str(&format!("pub struct {} {{\n", struct_name));

        // Generate fields from data nodes
        for node in &case.data_nodes {
            output.push_str(&self.generate_struct_field(node)?);
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate a Rust struct and Vec type alias from a YANG list.
    fn generate_list(&self, list: &crate::parser::List) -> Result<String, GeneratorError> {
        let mut output = String::new();

        // Generate rustdoc comment from YANG description
        if let Some(ref description) = list.description {
            output.push_str(&self.generate_rustdoc(description));
        }

        // Generate derive attributes
        output.push_str(&self.generate_derive_attributes());

        // Generate struct definition for list items
        let type_name = naming::to_type_name(&list.name);
        // Remove trailing 's' for singular item type name if present
        let item_type_name = if type_name.ends_with('s') && type_name.len() > 1 {
            &type_name[..type_name.len() - 1]
        } else {
            &type_name
        };

        output.push_str(&format!("pub struct {} {{\n", item_type_name));

        // Generate fields from child nodes
        // Key fields must be non-optional
        for child in &list.children {
            output.push_str(&self.generate_list_field(child, &list.keys)?);
        }

        output.push_str("}\n\n");

        // Generate Vec type alias for the collection
        output.push_str(&format!("/// Collection of {} items.\n", item_type_name));
        output.push_str(&format!(
            "pub type {} = Vec<{}>;\n",
            type_name, item_type_name
        ));

        // Recursively generate types for nested containers, lists, and choices
        for child in &list.children {
            match child {
                crate::parser::DataNode::Container(nested) => {
                    output.push('\n');
                    output.push_str(&self.generate_container(nested)?);
                }
                crate::parser::DataNode::List(nested) => {
                    output.push('\n');
                    output.push_str(&self.generate_list(nested)?);
                }
                crate::parser::DataNode::Choice(nested) => {
                    output.push('\n');
                    output.push_str(&self.generate_choice(nested)?);
                }
                _ => {}
            }
        }

        Ok(output)
    }

    /// Generate a struct field from a data node within a list.
    /// Key fields are always mandatory (non-optional).
    fn generate_list_field(
        &self,
        node: &crate::parser::DataNode,
        keys: &[String],
    ) -> Result<String, GeneratorError> {
        use crate::parser::DataNode;

        match node {
            DataNode::Leaf(leaf) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = leaf.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Check if this leaf is a key field
                let is_key = keys.contains(&leaf.name);

                // Build serde attributes
                let mut serde_attrs = vec![format!("rename = \"{}\"", leaf.name)];
                // Key fields are always mandatory, so only add skip_serializing_if for non-key optional fields
                if !is_key && !leaf.mandatory {
                    serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
                }
                field.push_str(&format!("    #[serde({})]\n", serde_attrs.join(", ")));

                // Generate field name and type
                let field_name = naming::to_field_name(&leaf.name);
                // Key fields are always non-optional
                let field_type = if is_key {
                    self.generate_leaf_type(&leaf.type_spec, true)
                } else {
                    self.generate_leaf_type(&leaf.type_spec, leaf.mandatory)
                };
                field.push_str(&format!("    pub {}: {},\n", field_name, field_type));

                Ok(field)
            }
            DataNode::Container(container) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = container.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                let mut serde_attrs = vec![format!("rename = \"{}\"", container.name)];
                if !container.mandatory {
                    serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
                }
                field.push_str(&format!("    #[serde({})]\n", serde_attrs.join(", ")));

                // Generate field name and type
                let field_name = naming::to_field_name(&container.name);
                let type_name = naming::to_type_name(&container.name);
                let field_type = if container.mandatory {
                    type_name
                } else {
                    format!("Option<{}>", type_name)
                };
                field.push_str(&format!("    pub {}: {},\n", field_name, field_type));

                Ok(field)
            }
            DataNode::List(nested_list) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = nested_list.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                field.push_str(&format!(
                    "    #[serde(rename = \"{}\")]\n",
                    nested_list.name
                ));

                // Generate field name and type
                let field_name = naming::to_field_name(&nested_list.name);
                let type_name = naming::to_type_name(&nested_list.name);
                // Lists are always collections (Vec)
                field.push_str(&format!("    pub {}: {},\n", field_name, type_name));

                Ok(field)
            }
            DataNode::LeafList(_) => Ok(String::new()), // Will be implemented later
            DataNode::Choice(choice) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = choice.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                let mut serde_attrs = vec![format!("rename = \"{}\"", choice.name)];
                if !choice.mandatory {
                    serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
                }
                field.push_str(&format!("    #[serde({})]\n", serde_attrs.join(", ")));

                // Generate field name and type
                let field_name = naming::to_field_name(&choice.name);
                let type_name = naming::to_type_name(&choice.name);
                let field_type = if choice.mandatory {
                    type_name
                } else {
                    format!("Option<{}>", type_name)
                };
                field.push_str(&format!("    pub {}: {},\n", field_name, field_type));

                Ok(field)
            }
            DataNode::Case(_) => Ok(String::new()), // Cases are handled within choices
            DataNode::Uses(_) => Ok(String::new()), // Uses should be expanded during parsing
        }
    }

    /// Generate a struct field from a data node.
    fn generate_struct_field(
        &self,
        node: &crate::parser::DataNode,
    ) -> Result<String, GeneratorError> {
        use crate::parser::DataNode;

        match node {
            DataNode::Leaf(leaf) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = leaf.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                let mut serde_attrs = vec![format!("rename = \"{}\"", leaf.name)];
                if !leaf.mandatory {
                    serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
                }
                field.push_str(&format!("    #[serde({})]\n", serde_attrs.join(", ")));

                // Generate field name and type
                let field_name = naming::to_field_name(&leaf.name);
                let field_type = self.generate_leaf_type(&leaf.type_spec, leaf.mandatory);
                field.push_str(&format!("    pub {}: {},\n", field_name, field_type));

                Ok(field)
            }
            DataNode::Container(container) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = container.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                let mut serde_attrs = vec![format!("rename = \"{}\"", container.name)];
                if !container.mandatory {
                    serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
                }
                field.push_str(&format!("    #[serde({})]\n", serde_attrs.join(", ")));

                // Generate field name and type
                let field_name = naming::to_field_name(&container.name);
                let type_name = naming::to_type_name(&container.name);
                let field_type = if container.mandatory {
                    type_name
                } else {
                    format!("Option<{}>", type_name)
                };
                field.push_str(&format!("    pub {}: {},\n", field_name, field_type));

                Ok(field)
            }
            DataNode::List(list) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = list.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                field.push_str(&format!("    #[serde(rename = \"{}\")]\n", list.name));

                // Generate field name and type
                let field_name = naming::to_field_name(&list.name);
                let type_name = naming::to_type_name(&list.name);
                // Lists are always collections (Vec)
                field.push_str(&format!("    pub {}: {},\n", field_name, type_name));

                Ok(field)
            }
            DataNode::LeafList(_) => Ok(String::new()), // Will be implemented later
            DataNode::Choice(choice) => {
                let mut field = String::new();

                // Add rustdoc comment if description exists
                if let Some(ref description) = choice.description {
                    field.push_str(&format!("    {}", self.generate_rustdoc(description)));
                }

                // Build serde attributes
                let mut serde_attrs = vec![format!("rename = \"{}\"", choice.name)];
                if !choice.mandatory {
                    serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_string());
                }
                field.push_str(&format!("    #[serde({})]\n", serde_attrs.join(", ")));

                // Generate field name and type
                let field_name = naming::to_field_name(&choice.name);
                let type_name = naming::to_type_name(&choice.name);
                let field_type = if choice.mandatory {
                    type_name
                } else {
                    format!("Option<{}>", type_name)
                };
                field.push_str(&format!("    pub {}: {},\n", field_name, field_type));

                Ok(field)
            }
            DataNode::Case(_) => Ok(String::new()), // Cases are handled within choices
            DataNode::Uses(_) => Ok(String::new()), // Uses should be expanded during parsing
        }
    }

    /// Generate a Rust type from a YANG leaf type specification.
    fn generate_leaf_type(&self, type_spec: &crate::parser::TypeSpec, mandatory: bool) -> String {
        use crate::parser::TypeSpec;

        // Check if we should generate a validated type
        if self.config.enable_validation && self.needs_validation(type_spec) {
            let validated_type_name = self.get_validated_type_name(type_spec);
            if mandatory {
                return validated_type_name;
            } else {
                return format!("Option<{}>", validated_type_name);
            }
        }

        let base_type = match type_spec {
            TypeSpec::Int8 { .. } => "i8",
            TypeSpec::Int16 { .. } => "i16",
            TypeSpec::Int32 { .. } => "i32",
            TypeSpec::Int64 { .. } => "i64",
            TypeSpec::Uint8 { .. } => "u8",
            TypeSpec::Uint16 { .. } => "u16",
            TypeSpec::Uint32 { .. } => "u32",
            TypeSpec::Uint64 { .. } => "u64",
            TypeSpec::String { .. } => "String",
            TypeSpec::Boolean => "bool",
            TypeSpec::Empty => "()",
            TypeSpec::Binary { .. } => "Vec<u8>",
            TypeSpec::Enumeration { .. } => "String", // Will be improved in later tasks
            TypeSpec::Union { .. } => "String",       // Will be improved in later tasks
            TypeSpec::LeafRef { .. } => "String",     // Will be improved in later tasks
            TypeSpec::TypedefRef { name } => {
                // Use the typedef name as the type
                &naming::to_type_name(name)
            }
        };

        if mandatory {
            base_type.to_string()
        } else {
            format!("Option<{}>", base_type)
        }
    }

    /// Check if a type specification needs validation.
    fn needs_validation(&self, type_spec: &crate::parser::TypeSpec) -> bool {
        use crate::parser::TypeSpec;

        match type_spec {
            TypeSpec::Int8 { range } => range.is_some(),
            TypeSpec::Int16 { range } => range.is_some(),
            TypeSpec::Int32 { range } => range.is_some(),
            TypeSpec::Int64 { range } => range.is_some(),
            TypeSpec::Uint8 { range } => range.is_some(),
            TypeSpec::Uint16 { range } => range.is_some(),
            TypeSpec::Uint32 { range } => range.is_some(),
            TypeSpec::Uint64 { range } => range.is_some(),
            TypeSpec::String { length, pattern } => length.is_some() || pattern.is_some(),
            TypeSpec::Binary { length } => length.is_some(),
            _ => false,
        }
    }

    /// Get the validated type name for a type specification.
    fn get_validated_type_name(&self, type_spec: &crate::parser::TypeSpec) -> String {
        use crate::parser::TypeSpec;

        match type_spec {
            TypeSpec::Int8 { range } if range.is_some() => {
                format!("ValidatedInt8_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Int16 { range } if range.is_some() => {
                format!("ValidatedInt16_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Int32 { range } if range.is_some() => {
                format!("ValidatedInt32_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Int64 { range } if range.is_some() => {
                format!("ValidatedInt64_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Uint8 { range } if range.is_some() => {
                format!("ValidatedUint8_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Uint16 { range } if range.is_some() => {
                format!("ValidatedUint16_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Uint32 { range } if range.is_some() => {
                format!("ValidatedUint32_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Uint64 { range } if range.is_some() => {
                format!("ValidatedUint64_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::String { length, pattern } if length.is_some() || pattern.is_some() => {
                format!("ValidatedString_{}", self.constraint_hash(type_spec))
            }
            TypeSpec::Binary { length } if length.is_some() => {
                format!("ValidatedBinary_{}", self.constraint_hash(type_spec))
            }
            _ => "Unknown".to_string(),
        }
    }

    /// Generate a hash for constraint uniqueness.
    fn constraint_hash(&self, type_spec: &crate::parser::TypeSpec) -> String {
        use crate::parser::TypeSpec;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        match type_spec {
            TypeSpec::Int8 { range: Some(r) }
            | TypeSpec::Int16 { range: Some(r) }
            | TypeSpec::Int32 { range: Some(r) }
            | TypeSpec::Int64 { range: Some(r) }
            | TypeSpec::Uint8 { range: Some(r) }
            | TypeSpec::Uint16 { range: Some(r) }
            | TypeSpec::Uint32 { range: Some(r) }
            | TypeSpec::Uint64 { range: Some(r) } => {
                for range in &r.ranges {
                    range.min.hash(&mut hasher);
                    range.max.hash(&mut hasher);
                }
            }
            TypeSpec::String { length, pattern } => {
                if let Some(l) = length {
                    for len in &l.lengths {
                        len.min.hash(&mut hasher);
                        len.max.hash(&mut hasher);
                    }
                }
                if let Some(p) = pattern {
                    p.pattern.hash(&mut hasher);
                }
            }
            TypeSpec::Binary { length: Some(l) } => {
                for len in &l.lengths {
                    len.min.hash(&mut hasher);
                    len.max.hash(&mut hasher);
                }
            }
            _ => {}
        }

        format!("{:x}", hasher.finish())
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

        // Collect from data nodes
        for node in &module.data_nodes {
            self.collect_validated_types_from_node(node, &mut types);
        }

        types.into_iter().collect()
    }

    /// Recursively collect validated types from a data node.
    fn collect_validated_types_from_node(
        &self,
        node: &crate::parser::DataNode,
        types: &mut std::collections::HashMap<String, crate::parser::TypeSpec>,
    ) {
        use crate::parser::DataNode;

        match node {
            DataNode::Leaf(leaf) => {
                if self.needs_validation(&leaf.type_spec) {
                    let type_name = self.get_validated_type_name(&leaf.type_spec);
                    types.insert(type_name, leaf.type_spec.clone());
                }
            }
            DataNode::Container(container) => {
                for child in &container.children {
                    self.collect_validated_types_from_node(child, types);
                }
            }
            DataNode::List(list) => {
                for child in &list.children {
                    self.collect_validated_types_from_node(child, types);
                }
            }
            DataNode::Choice(choice) => {
                for case in &choice.cases {
                    for child in &case.data_nodes {
                        self.collect_validated_types_from_node(child, types);
                    }
                }
            }
            DataNode::Case(case) => {
                for child in &case.data_nodes {
                    self.collect_validated_types_from_node(child, types);
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
