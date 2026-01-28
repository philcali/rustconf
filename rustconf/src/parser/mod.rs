//! YANG specification parser module.
//!
//! This module provides functionality to parse YANG 1.0 and 1.1 specification files
//! into an abstract syntax tree (AST).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod ast;
pub mod error;
pub mod lexer;

pub use ast::*;
pub use error::ParseError;
pub use lexer::{Lexer, Token};

/// Visitor for validating typedef references in data nodes.
///
/// This visitor traverses the data node tree and validates that all typedef
/// references point to defined typedefs.
struct TypedefRefValidator<'a> {
    typedefs: &'a [TypeDef],
}

impl<'a> DataNodeVisitor for TypedefRefValidator<'a> {
    type Error = ParseError;

    fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
        YangParser::validate_typespec_references(&leaf.type_spec, self.typedefs)
    }

    fn visit_leaf_list(&mut self, leaf_list: &LeafList) -> Result<(), Self::Error> {
        YangParser::validate_typespec_references(&leaf_list.type_spec, self.typedefs)
    }

    // Use default implementations for container, list, choice, case
    // which will recursively visit children
}

/// Visitor for validating grouping references in data nodes.
///
/// This visitor traverses the data node tree and validates that all uses
/// statements reference defined groupings.
struct GroupingRefValidator<'a> {
    groupings: &'a [Grouping],
}

impl<'a> DataNodeVisitor for GroupingRefValidator<'a> {
    type Error = ParseError;

    fn visit_uses(&mut self, uses: &Uses) -> Result<(), Self::Error> {
        // Check if the grouping is defined
        if !self.groupings.iter().any(|g| g.name == uses.name) {
            return Err(ParseError::SemanticError {
                message: format!("Undefined grouping reference: {}", uses.name),
            });
        }
        Ok(())
    }

    // Use default implementations for container, list, choice, case
    // which will recursively visit children
}

/// Visitor for validating leafref paths in data nodes.
///
/// This visitor traverses the data node tree and validates that all leafref
/// paths are non-empty.
struct LeafrefPathValidator;

impl DataNodeVisitor for LeafrefPathValidator {
    type Error = ParseError;

    fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
        YangParser::validate_typespec_leafref_paths(&leaf.type_spec)
    }

    fn visit_leaf_list(&mut self, leaf_list: &LeafList) -> Result<(), Self::Error> {
        YangParser::validate_typespec_leafref_paths(&leaf_list.type_spec)
    }

    // Use default implementations for container, list, choice, case
    // which will recursively visit children
}

/// Visitor for validating type constraints in data nodes.
///
/// This visitor traverses the data node tree and validates that all type
/// constraints (range, length) are well-formed (min <= max).
struct ConstraintValidator<'a> {
    parser: &'a YangParser,
}

impl<'a> DataNodeVisitor for ConstraintValidator<'a> {
    type Error = ParseError;

    fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
        self.parser.validate_typespec_constraints(&leaf.type_spec)
    }

    fn visit_leaf_list(&mut self, leaf_list: &LeafList) -> Result<(), Self::Error> {
        self.parser
            .validate_typespec_constraints(&leaf_list.type_spec)
    }

    // Use default implementations for container, list, choice, case
    // which will recursively visit children
}

/// YANG parser with configurable search paths for module resolution.
pub struct YangParser {
    search_paths: Vec<PathBuf>,
    loaded_modules: HashMap<String, YangModule>,
}

impl YangParser {
    /// Create a new YANG parser with default settings.
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            loaded_modules: HashMap::new(),
        }
    }

    /// Add a search path for resolving YANG module imports.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Parse a YANG file from the given path.
    pub fn parse_file(&mut self, path: &Path) -> Result<YangModule, ParseError> {
        let content = fs::read_to_string(path)?;
        let filename = path.to_string_lossy().to_string();
        self.parse_string(&content, &filename)
    }

    /// Parse YANG content from a string.
    pub fn parse_string(
        &mut self,
        content: &str,
        filename: &str,
    ) -> Result<YangModule, ParseError> {
        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize().map_err(|e| ParseError::SyntaxError {
            line: 1,
            column: 1,
            message: e,
        })?;

        let mut parser = ModuleParser::new(tokens, filename);
        let module = parser.parse_module()?;

        // Try to resolve imports recursively (non-fatal if imports can't be found)
        // This allows parsing modules without having all dependencies available
        let _ = self.resolve_imports(&module);

        // Store the parsed module
        self.loaded_modules
            .insert(module.name.clone(), module.clone());

        Ok(module)
    }

    /// Resolve all imports for a module by recursively loading imported modules.
    fn resolve_imports(&mut self, module: &YangModule) -> Result<(), ParseError> {
        for import in &module.imports {
            // Skip if already loaded
            if self.loaded_modules.contains_key(&import.module) {
                continue;
            }

            // Try to find and load the imported module
            let imported_module = self.find_and_load_module(&import.module)?;

            // Recursively resolve imports of the imported module
            self.resolve_imports(&imported_module)?;

            // Store the loaded module
            self.loaded_modules
                .insert(import.module.clone(), imported_module);
        }

        Ok(())
    }

    /// Find and load a module by searching through the search paths.
    fn find_and_load_module(&mut self, module_name: &str) -> Result<YangModule, ParseError> {
        // Try each search path
        for search_path in &self.search_paths {
            // Try with .yang extension
            let module_path = search_path.join(format!("{}.yang", module_name));
            if module_path.exists() {
                let content = fs::read_to_string(&module_path)?;
                let filename = module_path.to_string_lossy().to_string();

                let mut lexer = Lexer::new(&content);
                let tokens = lexer.tokenize().map_err(|e| ParseError::SyntaxError {
                    line: 1,
                    column: 1,
                    message: e,
                })?;

                let mut parser = ModuleParser::new(tokens, &filename);
                return parser.parse_module();
            }
        }

        // Module not found in any search path
        Err(ParseError::UnresolvedImport {
            module: module_name.to_string(),
        })
    }

    /// Get a loaded module by name.
    pub fn get_loaded_module(&self, name: &str) -> Option<&YangModule> {
        self.loaded_modules.get(name)
    }

    /// Get all loaded modules.
    pub fn get_all_loaded_modules(&self) -> &HashMap<String, YangModule> {
        &self.loaded_modules
    }

    /// Perform semantic validation on a module.
    /// This validates:
    /// - All typedef references are defined
    /// - All grouping references are defined
    /// - All leafref paths are valid
    /// - No circular dependencies in imports
    /// - No circular dependencies in groupings
    /// - Type constraints are well-formed (range min < max, etc.)
    pub fn validate_module(&self, module: &YangModule) -> Result<(), ParseError> {
        // Validate circular import dependencies
        self.validate_no_circular_imports(module, &mut Vec::new())?;

        // Validate circular grouping dependencies
        self.validate_no_circular_groupings(module)?;

        // Validate all references are defined
        self.validate_typedef_references(module)?;
        self.validate_grouping_references(module)?;
        self.validate_leafref_paths(module)?;

        // Validate type constraints are well-formed
        self.validate_type_constraints(module)?;

        Ok(())
    }

    /// Validate that there are no circular import dependencies.
    fn validate_no_circular_imports(
        &self,
        module: &YangModule,
        visited: &mut Vec<String>,
    ) -> Result<(), ParseError> {
        // Check if we've already visited this module (circular dependency)
        if visited.contains(&module.name) {
            return Err(ParseError::SemanticError {
                message: format!(
                    "Circular import dependency detected: {} -> {}",
                    visited.join(" -> "),
                    module.name
                ),
            });
        }

        // Add current module to visited path
        visited.push(module.name.clone());

        // Check all imports recursively
        for import in &module.imports {
            if let Some(imported_module) = self.loaded_modules.get(&import.module) {
                self.validate_no_circular_imports(imported_module, visited)?;
            }
        }

        // Remove current module from visited path (backtrack)
        visited.pop();

        Ok(())
    }

    /// Validate that there are no circular grouping dependencies.
    fn validate_no_circular_groupings(&self, module: &YangModule) -> Result<(), ParseError> {
        for grouping in &module.groupings {
            let mut visited = Vec::new();
            self.validate_grouping_no_cycles(
                &grouping.name,
                &grouping.data_nodes,
                &module.groupings,
                &mut visited,
            )?;
        }
        Ok(())
    }

    /// Recursively validate that a grouping doesn't have circular dependencies.
    fn validate_grouping_no_cycles(
        &self,
        grouping_name: &str,
        data_nodes: &[DataNode],
        all_groupings: &[Grouping],
        visited: &mut Vec<String>,
    ) -> Result<(), ParseError> {
        // Check if we've already visited this grouping (circular dependency)
        if visited.contains(&grouping_name.to_string()) {
            return Err(ParseError::SemanticError {
                message: format!(
                    "Circular grouping dependency detected: {} -> {}",
                    visited.join(" -> "),
                    grouping_name
                ),
            });
        }

        // Add current grouping to visited path
        visited.push(grouping_name.to_string());

        // Check all uses statements in the data nodes
        for data_node in data_nodes {
            self.check_data_node_for_circular_uses(data_node, all_groupings, visited)?;
        }

        // Remove current grouping from visited path (backtrack)
        visited.pop();

        Ok(())
    }

    /// Check a data node for uses statements that might create circular dependencies.
    fn check_data_node_for_circular_uses(
        &self,
        data_node: &DataNode,
        all_groupings: &[Grouping],
        visited: &mut Vec<String>,
    ) -> Result<(), ParseError> {
        match data_node {
            DataNode::Uses(uses) => {
                // Find the referenced grouping
                if let Some(grouping) = all_groupings.iter().find(|g| g.name == uses.name) {
                    self.validate_grouping_no_cycles(
                        &grouping.name,
                        &grouping.data_nodes,
                        all_groupings,
                        visited,
                    )?;
                }
            }
            DataNode::Container(container) => {
                for child in &container.children {
                    self.check_data_node_for_circular_uses(child, all_groupings, visited)?;
                }
            }
            DataNode::List(list) => {
                for child in &list.children {
                    self.check_data_node_for_circular_uses(child, all_groupings, visited)?;
                }
            }
            DataNode::Choice(choice) => {
                for case in &choice.cases {
                    for child in &case.data_nodes {
                        self.check_data_node_for_circular_uses(child, all_groupings, visited)?;
                    }
                }
            }
            DataNode::Case(case) => {
                for child in &case.data_nodes {
                    self.check_data_node_for_circular_uses(child, all_groupings, visited)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate that all typedef references are defined.
    fn validate_typedef_references(&self, module: &YangModule) -> Result<(), ParseError> {
        // Check typedefs in the module's own typedefs
        for typedef in &module.typedefs {
            Self::validate_typespec_references(&typedef.type_spec, &module.typedefs)?;
        }

        // Check typedefs in data nodes using visitor pattern
        let mut validator = TypedefRefValidator {
            typedefs: &module.typedefs,
        };
        walk_data_nodes(&module.data_nodes, &mut validator)?;

        Ok(())
    }

    /// Validate typedef references in a TypeSpec.
    fn validate_typespec_references(
        type_spec: &TypeSpec,
        typedefs: &[TypeDef],
    ) -> Result<(), ParseError> {
        match type_spec {
            TypeSpec::TypedefRef { name } => {
                // Check if the typedef is defined
                if !typedefs.iter().any(|t| t.name == *name) {
                    return Err(ParseError::SemanticError {
                        message: format!("Undefined typedef reference: {}", name),
                    });
                }
            }
            TypeSpec::Union { types } => {
                // Validate each type in the union
                for t in types {
                    Self::validate_typespec_references(t, typedefs)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate that all grouping references are defined.
    fn validate_grouping_references(&self, module: &YangModule) -> Result<(), ParseError> {
        let mut validator = GroupingRefValidator {
            groupings: &module.groupings,
        };

        // Check groupings in module data nodes
        walk_data_nodes(&module.data_nodes, &mut validator)?;

        // Also check groupings themselves for nested uses
        for grouping in &module.groupings {
            walk_data_nodes(&grouping.data_nodes, &mut validator)?;
        }

        Ok(())
    }

    /// Validate that all leafref paths are valid.
    /// Note: This is a simplified validation that just checks the path is not empty.
    /// Full path validation would require resolving the path against the data tree.
    fn validate_leafref_paths(&self, module: &YangModule) -> Result<(), ParseError> {
        let mut validator = LeafrefPathValidator;
        walk_data_nodes(&module.data_nodes, &mut validator)?;
        Ok(())
    }

    /// Validate leafref paths in a TypeSpec.
    fn validate_typespec_leafref_paths(type_spec: &TypeSpec) -> Result<(), ParseError> {
        match type_spec {
            TypeSpec::LeafRef { path } => {
                if path.is_empty() {
                    return Err(ParseError::SemanticError {
                        message: "Leafref path cannot be empty".to_string(),
                    });
                }
            }
            TypeSpec::Union { types } => {
                for t in types {
                    Self::validate_typespec_leafref_paths(t)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate that type constraints are well-formed.
    fn validate_type_constraints(&self, module: &YangModule) -> Result<(), ParseError> {
        // Validate constraints in typedefs
        for typedef in &module.typedefs {
            self.validate_typespec_constraints(&typedef.type_spec)?;
        }

        // Validate constraints in data nodes using visitor
        let mut validator = ConstraintValidator { parser: self };
        walk_data_nodes(&module.data_nodes, &mut validator)?;

        Ok(())
    }

    /// Validate constraints in a TypeSpec.
    fn validate_typespec_constraints(&self, type_spec: &TypeSpec) -> Result<(), ParseError> {
        match type_spec {
            TypeSpec::Int8 { range }
            | TypeSpec::Int16 { range }
            | TypeSpec::Int32 { range }
            | TypeSpec::Int64 { range }
            | TypeSpec::Uint8 { range }
            | TypeSpec::Uint16 { range }
            | TypeSpec::Uint32 { range }
            | TypeSpec::Uint64 { range } => {
                if let Some(range_constraint) = range {
                    self.validate_range_constraint(range_constraint)?;
                }
            }
            TypeSpec::String { length, .. } | TypeSpec::Binary { length } => {
                if let Some(length_constraint) = length {
                    self.validate_length_constraint(length_constraint)?;
                }
            }
            TypeSpec::Union { types } => {
                for t in types {
                    self.validate_typespec_constraints(t)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate that a range constraint is well-formed (min <= max).
    fn validate_range_constraint(&self, constraint: &RangeConstraint) -> Result<(), ParseError> {
        for range in &constraint.ranges {
            if range.min > range.max {
                return Err(ParseError::SemanticError {
                    message: format!(
                        "Invalid range constraint: min ({}) is greater than max ({})",
                        range.min, range.max
                    ),
                });
            }
        }
        Ok(())
    }

    /// Validate that a length constraint is well-formed (min <= max).
    fn validate_length_constraint(&self, constraint: &LengthConstraint) -> Result<(), ParseError> {
        for length_range in &constraint.lengths {
            if length_range.min > length_range.max {
                return Err(ParseError::SemanticError {
                    message: format!(
                        "Invalid length constraint: min ({}) is greater than max ({})",
                        length_range.min, length_range.max
                    ),
                });
            }
        }
        Ok(())
    }

    /// Expand typedef references and grouping uses in a module.
    /// This resolves all TypedefRef types to their concrete types and
    /// expands all Uses nodes to their grouping definitions.
    pub fn expand_module(&self, module: &mut YangModule) -> Result<(), ParseError> {
        // First, expand typedefs in the module's own typedefs (for nested typedefs)
        // We need to collect typedef information first to avoid borrow issues
        let typedefs = module.typedefs.clone();

        for typedef in &mut module.typedefs {
            Self::expand_typedef_in_typespec_with_defs(&mut typedef.type_spec, &typedefs)?;
        }

        // Expand data nodes
        let typedefs = module.typedefs.clone();
        let groupings = module.groupings.clone();

        for data_node in &mut module.data_nodes {
            Self::expand_data_node_with_defs(data_node, &typedefs, &groupings)?;
        }

        Ok(())
    }

    /// Expand typedef references in a TypeSpec using a list of typedef definitions.
    fn expand_typedef_in_typespec_with_defs(
        type_spec: &mut TypeSpec,
        typedefs: &[TypeDef],
    ) -> Result<(), ParseError> {
        match type_spec {
            TypeSpec::TypedefRef { name } => {
                // Find the typedef definition
                let typedef = typedefs.iter().find(|t| t.name == *name).ok_or_else(|| {
                    ParseError::SemanticError {
                        message: format!("Undefined typedef: {}", name),
                    }
                })?;

                // Replace with the typedef's type_spec
                *type_spec = typedef.type_spec.clone();

                // Recursively expand in case the typedef itself references another typedef
                Self::expand_typedef_in_typespec_with_defs(type_spec, typedefs)?;
            }
            TypeSpec::Union { types } => {
                // Expand each type in the union
                for t in types {
                    Self::expand_typedef_in_typespec_with_defs(t, typedefs)?;
                }
            }
            _ => {
                // Other types don't need expansion
            }
        }
        Ok(())
    }

    /// Expand typedef references and grouping uses in a data node.
    fn expand_data_node_with_defs(
        data_node: &mut DataNode,
        typedefs: &[TypeDef],
        groupings: &[Grouping],
    ) -> Result<(), ParseError> {
        match data_node {
            DataNode::Container(container) => {
                Self::expand_children_with_defs(&mut container.children, typedefs, groupings)?;
            }
            DataNode::List(list) => {
                Self::expand_children_with_defs(&mut list.children, typedefs, groupings)?;
            }
            DataNode::Leaf(leaf) => {
                Self::expand_typedef_in_typespec_with_defs(&mut leaf.type_spec, typedefs)?;
            }
            DataNode::LeafList(leaf_list) => {
                Self::expand_typedef_in_typespec_with_defs(&mut leaf_list.type_spec, typedefs)?;
            }
            DataNode::Choice(choice) => {
                for case in &mut choice.cases {
                    Self::expand_children_with_defs(&mut case.data_nodes, typedefs, groupings)?;
                }
            }
            DataNode::Case(case) => {
                Self::expand_children_with_defs(&mut case.data_nodes, typedefs, groupings)?;
            }
            DataNode::Uses(_) => {
                // Uses nodes will be expanded by expand_children_with_defs
            }
        }
        Ok(())
    }

    /// Expand children nodes, replacing Uses nodes with their grouping definitions.
    fn expand_children_with_defs(
        children: &mut Vec<DataNode>,
        typedefs: &[TypeDef],
        groupings: &[Grouping],
    ) -> Result<(), ParseError> {
        let mut expanded = Vec::new();

        for child in children.iter() {
            match child {
                DataNode::Uses(uses) => {
                    // Find the grouping definition
                    let grouping =
                        groupings
                            .iter()
                            .find(|g| g.name == uses.name)
                            .ok_or_else(|| ParseError::SemanticError {
                                message: format!("Undefined grouping: {}", uses.name),
                            })?;

                    // Clone the grouping's data nodes and expand them recursively
                    let mut cloned_nodes = grouping.data_nodes.clone();

                    // Recursively expand the cloned nodes (this handles nested uses)
                    Self::expand_children_with_defs(&mut cloned_nodes, typedefs, groupings)?;

                    // Add the expanded nodes to the result
                    expanded.extend(cloned_nodes);
                }
                _ => {
                    // Clone the child and recursively expand it
                    let mut cloned_child = child.clone();
                    Self::expand_data_node_with_defs(&mut cloned_child, typedefs, groupings)?;
                    expanded.push(cloned_child);
                }
            }
        }

        *children = expanded;
        Ok(())
    }
}

impl Default for YangParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor trait for traversing data nodes in a YANG module.
///
/// This trait provides a way to traverse the data node tree without duplicating
/// the traversal logic. Implement this trait to perform operations on data nodes
/// such as validation, transformation, or collection.
///
/// # Example
///
/// ```ignore
/// struct MyVisitor;
///
/// impl DataNodeVisitor for MyVisitor {
///     type Error = String;
///
///     fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
///         println!("Visiting leaf: {}", leaf.name);
///         Ok(())
///     }
/// }
///
/// let mut visitor = MyVisitor;
/// walk_data_nodes(&module.data_nodes, &mut visitor)?;
/// ```
pub trait DataNodeVisitor: Sized {
    /// The error type returned by visitor methods.
    type Error;

    /// Visit a leaf node.
    ///
    /// Default implementation does nothing.
    fn visit_leaf(&mut self, _leaf: &Leaf) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visit a leaf-list node.
    ///
    /// Default implementation does nothing.
    fn visit_leaf_list(&mut self, _leaf_list: &LeafList) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Visit a container node.
    ///
    /// Default implementation recursively visits all children.
    fn visit_container(&mut self, container: &Container) -> Result<(), Self::Error> {
        for child in &container.children {
            walk_data_node(child, self)?;
        }
        Ok(())
    }

    /// Visit a list node.
    ///
    /// Default implementation recursively visits all children.
    fn visit_list(&mut self, list: &List) -> Result<(), Self::Error> {
        for child in &list.children {
            walk_data_node(child, self)?;
        }
        Ok(())
    }

    /// Visit a choice node.
    ///
    /// Default implementation recursively visits all cases.
    fn visit_choice(&mut self, choice: &Choice) -> Result<(), Self::Error> {
        for case in &choice.cases {
            self.visit_case(case)?;
        }
        Ok(())
    }

    /// Visit a case node.
    ///
    /// Default implementation recursively visits all data nodes in the case.
    fn visit_case(&mut self, case: &Case) -> Result<(), Self::Error> {
        for child in &case.data_nodes {
            walk_data_node(child, self)?;
        }
        Ok(())
    }

    /// Visit a uses node.
    ///
    /// Default implementation does nothing.
    /// Note: Uses nodes are typically expanded before visiting, so this may not be called
    /// in many scenarios.
    fn visit_uses(&mut self, _uses: &Uses) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Walk a single data node, dispatching to the appropriate visitor method.
///
/// This function handles the dispatch logic for visiting different types of data nodes.
/// It calls the appropriate visitor method based on the node type.
///
/// # Arguments
///
/// * `node` - The data node to visit
/// * `visitor` - The visitor implementation
///
/// # Errors
///
/// Returns any error returned by the visitor methods.
pub fn walk_data_node<V: DataNodeVisitor>(
    node: &DataNode,
    visitor: &mut V,
) -> Result<(), V::Error> {
    match node {
        DataNode::Leaf(leaf) => visitor.visit_leaf(leaf),
        DataNode::LeafList(leaf_list) => visitor.visit_leaf_list(leaf_list),
        DataNode::Container(container) => visitor.visit_container(container),
        DataNode::List(list) => visitor.visit_list(list),
        DataNode::Choice(choice) => visitor.visit_choice(choice),
        DataNode::Case(case) => visitor.visit_case(case),
        DataNode::Uses(uses) => visitor.visit_uses(uses),
    }
}

/// Walk a collection of data nodes, visiting each one in order.
///
/// This is a convenience function for visiting multiple data nodes, such as
/// the top-level data nodes in a module or the children of a container.
///
/// # Arguments
///
/// * `nodes` - The collection of data nodes to visit
/// * `visitor` - The visitor implementation
///
/// # Errors
///
/// Returns any error returned by the visitor methods. Stops at the first error.
pub fn walk_data_nodes<V: DataNodeVisitor>(
    nodes: &[DataNode],
    visitor: &mut V,
) -> Result<(), V::Error> {
    for node in nodes {
        walk_data_node(node, visitor)?;
    }
    Ok(())
}

/// Internal parser for processing tokens into AST.
struct ModuleParser {
    tokens: Vec<Token>,
    position: usize,
    _filename: String,
}

impl ModuleParser {
    fn new(tokens: Vec<Token>, filename: &str) -> Self {
        Self {
            tokens,
            position: 0,
            _filename: filename.to_string(),
        }
    }

    /// Get the current token without consuming it.
    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    /// Expect a specific token and consume it, or return an error.
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let token = self.peek();
        if std::mem::discriminant(token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("Expected {:?}, found {:?}", expected, token)))
        }
    }

    /// Create a syntax error at the current position.
    fn error(&self, message: String) -> ParseError {
        ParseError::SyntaxError {
            line: 1, // TODO: Track line numbers in lexer
            column: self.position,
            message,
        }
    }

    /// Parse an identifier, allowing YANG keywords to be used as identifiers.
    /// This is necessary because YANG allows keywords like "description", "type", etc.
    /// to be used as identifiers in certain contexts.
    fn parse_identifier_or_keyword(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(id) => Ok(id),
            // Allow YANG keywords to be used as identifiers
            Token::Description => Ok("description".to_string()),
            Token::Type => Ok("type".to_string()),
            Token::Config => Ok("config".to_string()),
            Token::Status => Ok("status".to_string()),
            Token::Reference => Ok("reference".to_string()),
            Token::Default => Ok("default".to_string()),
            Token::Input => Ok("input".to_string()),
            Token::Output => Ok("output".to_string()),
            Token::Action => Ok("action".to_string()),
            Token::Notification => Ok("notification".to_string()),
            token => Err(self.error(format!("Expected identifier, found {:?}", token))),
        }
    }

    /// Parse a string that may be concatenated with + operator (YANG 1.1 feature).
    /// Handles: "string1" + "string2" + "string3" -> "string1string2string3"
    fn parse_concatenated_string(&mut self) -> Result<String, ParseError> {
        let mut result = String::new();

        // Parse the first string
        match self.advance() {
            Token::StringLiteral(s) => result.push_str(&s),
            token => return Err(self.error(format!("Expected string literal, found {:?}", token))),
        }

        // Check for concatenation with + operator
        while self.peek() == &Token::Plus {
            self.advance(); // consume the +

            // Parse the next string
            match self.advance() {
                Token::StringLiteral(s) => result.push_str(&s),
                token => {
                    return Err(self.error(format!(
                        "Expected string literal after +, found {:?}",
                        token
                    )))
                }
            }
        }

        Ok(result)
    }

    /// Parse a description statement: description <string> ;
    /// Supports string concatenation with + operator.
    fn parse_description_statement(&mut self) -> Result<String, ParseError> {
        self.expect(Token::Description)?;
        let description = self.parse_concatenated_string()?;
        self.expect(Token::Semicolon)?;
        Ok(description)
    }

    /// Parse a complete YANG module.
    fn parse_module(&mut self) -> Result<YangModule, ParseError> {
        // Expect: module <identifier> { <statements> }
        self.expect(Token::Module)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected module name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        // Parse module statements
        let mut yang_version = None;
        let mut namespace = None;
        let mut prefix = None;
        let mut imports = Vec::new();
        let mut typedefs = Vec::new();
        let mut groupings = Vec::new();
        let mut data_nodes = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::YangVersion => {
                    yang_version = Some(self.parse_yang_version()?);
                }
                Token::Namespace => {
                    namespace = Some(self.parse_namespace()?);
                }
                Token::Prefix => {
                    prefix = Some(self.parse_prefix()?);
                }
                Token::Import => {
                    imports.push(self.parse_import()?);
                }
                Token::Organization
                | Token::Contact
                | Token::Description
                | Token::Reference
                | Token::Revision => {
                    // Skip module metadata statements for now
                    self.skip_statement()?;
                }
                Token::Typedef => {
                    typedefs.push(self.parse_typedef()?);
                }
                Token::Grouping => {
                    groupings.push(self.parse_grouping()?);
                }
                Token::Container => {
                    data_nodes.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    data_nodes.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    data_nodes.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    data_nodes.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    data_nodes.push(DataNode::Choice(self.parse_choice()?));
                }
                Token::Uses => {
                    data_nodes.push(DataNode::Uses(self.parse_uses()?));
                }
                Token::Rpc | Token::Notification | Token::Action => {
                    // Skip RPC, notification, and action statements for now
                    self.skip_statement()?;
                }
                _ => {
                    // Skip unknown statements for now
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        // Validate required fields
        let namespace = namespace.ok_or_else(|| {
            self.error("Missing required 'namespace' statement in module".to_string())
        })?;
        let prefix = prefix.ok_or_else(|| {
            self.error("Missing required 'prefix' statement in module".to_string())
        })?;

        Ok(YangModule {
            name,
            namespace,
            prefix,
            yang_version,
            imports,
            typedefs,
            groupings,
            data_nodes,
            rpcs: Vec::new(),
            notifications: Vec::new(),
        })
    }

    /// Parse yang-version statement: yang-version "1.0" | "1.1" | 1.0 | 1.1 ;
    fn parse_yang_version(&mut self) -> Result<YangVersion, ParseError> {
        self.expect(Token::YangVersion)?;

        let version = match self.peek() {
            Token::StringLiteral(s) if s == "1.0" => {
                self.advance();
                YangVersion::V1_0
            }
            Token::StringLiteral(s) if s == "1.1" => {
                self.advance();
                YangVersion::V1_1
            }
            Token::Number(1) => {
                self.advance();
                // Check if followed by .0 or .1
                if self.peek() == &Token::Dot {
                    self.advance(); // consume the dot
                    match self.advance() {
                        Token::Number(0) => YangVersion::V1_0,
                        Token::Number(1) => YangVersion::V1_1,
                        token => {
                            return Err(
                                self.error(format!("Invalid yang-version value: 1.{:?}", token))
                            )
                        }
                    }
                } else {
                    YangVersion::V1_0
                }
            }
            Token::Identifier(s) if s == "1" => {
                self.advance();
                YangVersion::V1_0
            }
            token => return Err(self.error(format!("Invalid yang-version value: {:?}", token))),
        };

        self.expect(Token::Semicolon)?;
        Ok(version)
    }

    /// Parse namespace statement: namespace <string> ;
    fn parse_namespace(&mut self) -> Result<String, ParseError> {
        self.expect(Token::Namespace)?;

        let namespace = self.parse_concatenated_string()?;

        self.expect(Token::Semicolon)?;
        Ok(namespace)
    }

    /// Parse prefix statement: prefix <identifier> | <string> ;
    fn parse_prefix(&mut self) -> Result<String, ParseError> {
        self.expect(Token::Prefix)?;

        let prefix = match self.advance() {
            Token::Identifier(id) => id,
            Token::StringLiteral(s) => s,
            token => {
                return Err(self.error(format!(
                    "Expected prefix identifier or string, found {:?}",
                    token
                )))
            }
        };

        self.expect(Token::Semicolon)?;
        Ok(prefix)
    }

    /// Parse import statement: import <identifier> { prefix <identifier>; [revision-date <string>;] }
    fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.expect(Token::Import)?;

        let module = match self.advance() {
            Token::Identifier(id) => id,
            token => {
                return Err(self.error(format!("Expected import module name, found {:?}", token)))
            }
        };

        self.expect(Token::LeftBrace)?;

        let mut prefix = None;
        let revision = None;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Prefix => {
                    self.advance();
                    prefix = Some(match self.advance() {
                        Token::Identifier(id) => id,
                        token => {
                            return Err(self
                                .error(format!("Expected prefix identifier, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Revision => {
                    self.advance();
                    // Skip revision-date for now
                    self.skip_until_semicolon()?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let prefix = prefix.ok_or_else(|| {
            self.error("Missing required 'prefix' statement in import".to_string())
        })?;

        Ok(Import {
            module,
            prefix,
            revision,
        })
    }

    /// Parse typedef statement: typedef <identifier> { type <type-spec>; [units <string>;] [default <string>;] [description <string>;] }
    fn parse_typedef(&mut self) -> Result<TypeDef, ParseError> {
        self.expect(Token::Typedef)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected typedef name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut type_spec = None;
        let mut units = None;
        let mut default = None;
        let mut description = None;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Type => {
                    type_spec = Some(self.parse_type_spec()?);
                }
                Token::Units => {
                    self.advance();
                    units = Some(match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        token => {
                            return Err(
                                self.error(format!("Expected units value, found {:?}", token))
                            )
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Default => {
                    self.advance();
                    default = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        Token::Identifier(s) => s,
                        Token::Number(n) => n.to_string(),
                        token => {
                            return Err(
                                self.error(format!("Expected default value, found {:?}", token))
                            )
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let type_spec = type_spec.ok_or_else(|| {
            self.error("Missing required 'type' statement in typedef".to_string())
        })?;

        Ok(TypeDef {
            name,
            type_spec,
            units,
            default,
            description,
        })
    }

    /// Parse type specification: type <type-name> [{ <type-body> }]
    fn parse_type_spec(&mut self) -> Result<TypeSpec, ParseError> {
        self.expect(Token::Type)?;

        let base_type = self.peek().clone();

        let mut type_spec = match base_type {
            Token::Int8 => {
                self.advance();
                TypeSpec::Int8 { range: None }
            }
            Token::Int16 => {
                self.advance();
                TypeSpec::Int16 { range: None }
            }
            Token::Int32 => {
                self.advance();
                TypeSpec::Int32 { range: None }
            }
            Token::Int64 => {
                self.advance();
                TypeSpec::Int64 { range: None }
            }
            Token::Uint8 => {
                self.advance();
                TypeSpec::Uint8 { range: None }
            }
            Token::Uint16 => {
                self.advance();
                TypeSpec::Uint16 { range: None }
            }
            Token::Uint32 => {
                self.advance();
                TypeSpec::Uint32 { range: None }
            }
            Token::Uint64 => {
                self.advance();
                TypeSpec::Uint64 { range: None }
            }
            Token::String => {
                self.advance();
                TypeSpec::String {
                    length: None,
                    pattern: None,
                }
            }
            Token::Boolean => {
                self.advance();
                TypeSpec::Boolean
            }
            Token::Empty => {
                self.advance();
                TypeSpec::Empty
            }
            Token::Binary => {
                self.advance();
                TypeSpec::Binary { length: None }
            }
            Token::Enumeration => {
                self.advance();
                TypeSpec::Enumeration { values: Vec::new() }
            }
            Token::Union => {
                self.advance();
                TypeSpec::Union { types: Vec::new() }
            }
            Token::LeafRef => {
                self.advance();
                TypeSpec::LeafRef {
                    path: String::new(),
                }
            }
            Token::Identifier(_) => {
                // Could be a typedef reference or other type
                // Store as TypedefRef for later resolution
                let name = match self.advance() {
                    Token::Identifier(id) => id,
                    _ => unreachable!(),
                };
                TypeSpec::TypedefRef { name }
            }
            _ => return Err(self.error(format!("Expected type name, found {:?}", base_type))),
        };

        // Check for type body (constraints, enum values, union types, etc.)
        if self.peek() == &Token::LeftBrace {
            self.advance();
            type_spec = self.parse_type_body(type_spec)?;
            self.expect(Token::RightBrace)?;
            // No semicolon after type body - the type statement ends with the closing brace
        } else {
            // Expect semicolon after simple type (no body)
            self.expect(Token::Semicolon)?;
        }

        Ok(type_spec)
    }

    /// Parse type body (constraints, enum values, union types)
    fn parse_type_body(&mut self, mut type_spec: TypeSpec) -> Result<TypeSpec, ParseError> {
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Range => {
                    let range = self.parse_range_constraint()?;
                    type_spec = match type_spec {
                        TypeSpec::Int8 { .. } => TypeSpec::Int8 { range: Some(range) },
                        TypeSpec::Int16 { .. } => TypeSpec::Int16 { range: Some(range) },
                        TypeSpec::Int32 { .. } => TypeSpec::Int32 { range: Some(range) },
                        TypeSpec::Int64 { .. } => TypeSpec::Int64 { range: Some(range) },
                        TypeSpec::Uint8 { .. } => TypeSpec::Uint8 { range: Some(range) },
                        TypeSpec::Uint16 { .. } => TypeSpec::Uint16 { range: Some(range) },
                        TypeSpec::Uint32 { .. } => TypeSpec::Uint32 { range: Some(range) },
                        TypeSpec::Uint64 { .. } => TypeSpec::Uint64 { range: Some(range) },
                        _ => type_spec,
                    };
                }
                Token::Length => {
                    let length = self.parse_length_constraint()?;
                    type_spec = match type_spec {
                        TypeSpec::String { pattern, .. } => TypeSpec::String {
                            length: Some(length),
                            pattern,
                        },
                        TypeSpec::Binary { .. } => TypeSpec::Binary {
                            length: Some(length),
                        },
                        _ => type_spec,
                    };
                }
                Token::Pattern => {
                    let pattern = self.parse_pattern_constraint()?;
                    type_spec = match type_spec {
                        TypeSpec::String { length, .. } => TypeSpec::String {
                            length,
                            pattern: Some(pattern),
                        },
                        _ => type_spec,
                    };
                }
                Token::Enum => {
                    let enum_value = self.parse_enum_value()?;
                    if let TypeSpec::Enumeration { ref mut values } = type_spec {
                        values.push(enum_value);
                    }
                }
                Token::Type => {
                    // Union type member
                    let member_type = self.parse_type_spec()?;
                    if let TypeSpec::Union { ref mut types } = type_spec {
                        types.push(member_type);
                    }
                }
                Token::Identifier(ref id) if id == "path" => {
                    // leafref path
                    self.advance();
                    let path = match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        token => {
                            return Err(
                                self.error(format!("Expected path value, found {:?}", token))
                            )
                        }
                    };
                    self.expect(Token::Semicolon)?;
                    if let TypeSpec::LeafRef { .. } = type_spec {
                        type_spec = TypeSpec::LeafRef { path };
                    }
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }
        Ok(type_spec)
    }

    /// Parse range constraint: range "min..max | min..max"
    fn parse_range_constraint(&mut self) -> Result<RangeConstraint, ParseError> {
        self.expect(Token::Range)?;

        let range_str = match self.advance() {
            Token::StringLiteral(s) | Token::Identifier(s) => s,
            token => return Err(self.error(format!("Expected range value, found {:?}", token))),
        };

        self.expect(Token::Semicolon)?;

        // Parse range string: "1..10 | 20..30"
        let ranges = self.parse_range_string(&range_str)?;
        Ok(RangeConstraint::new(ranges))
    }

    /// Parse range string into Range objects
    fn parse_range_string(&self, range_str: &str) -> Result<Vec<Range>, ParseError> {
        let mut ranges = Vec::new();

        for part in range_str.split('|') {
            let part = part.trim();
            if part.contains("..") {
                let parts: Vec<&str> = part.split("..").collect();
                if parts.len() == 2 {
                    let min = parts[0].trim().parse::<i64>().map_err(|_| {
                        self.error(format!("Invalid range min value: {}", parts[0]))
                    })?;
                    let max = parts[1].trim().parse::<i64>().map_err(|_| {
                        self.error(format!("Invalid range max value: {}", parts[1]))
                    })?;
                    ranges.push(Range::new(min, max));
                }
            } else {
                // Single value range
                let value = part
                    .parse::<i64>()
                    .map_err(|_| self.error(format!("Invalid range value: {}", part)))?;
                ranges.push(Range::new(value, value));
            }
        }

        Ok(ranges)
    }

    /// Parse length constraint: length "min..max | min..max"
    fn parse_length_constraint(&mut self) -> Result<LengthConstraint, ParseError> {
        self.expect(Token::Length)?;

        let length_str = match self.advance() {
            Token::StringLiteral(s) | Token::Identifier(s) => s,
            token => return Err(self.error(format!("Expected length value, found {:?}", token))),
        };

        self.expect(Token::Semicolon)?;

        // Parse length string: "1..10 | 20..30"
        let lengths = self.parse_length_string(&length_str)?;
        Ok(LengthConstraint::new(lengths))
    }

    /// Parse length string into LengthRange objects
    fn parse_length_string(&self, length_str: &str) -> Result<Vec<LengthRange>, ParseError> {
        let mut lengths = Vec::new();

        for part in length_str.split('|') {
            let part = part.trim();
            if part.contains("..") {
                let parts: Vec<&str> = part.split("..").collect();
                if parts.len() == 2 {
                    let min = parts[0].trim().parse::<u64>().map_err(|_| {
                        self.error(format!("Invalid length min value: {}", parts[0]))
                    })?;
                    let max = parts[1].trim().parse::<u64>().map_err(|_| {
                        self.error(format!("Invalid length max value: {}", parts[1]))
                    })?;
                    lengths.push(LengthRange::new(min, max));
                }
            } else {
                // Single value length
                let value = part
                    .parse::<u64>()
                    .map_err(|_| self.error(format!("Invalid length value: {}", part)))?;
                lengths.push(LengthRange::new(value, value));
            }
        }

        Ok(lengths)
    }

    /// Parse pattern constraint: pattern "regex"
    fn parse_pattern_constraint(&mut self) -> Result<PatternConstraint, ParseError> {
        self.expect(Token::Pattern)?;

        let pattern = self.parse_concatenated_string()?;

        self.expect(Token::Semicolon)?;

        Ok(PatternConstraint::new(pattern))
    }

    /// Parse enum value: enum <identifier> [{ value <number>; description <string>; }]
    fn parse_enum_value(&mut self) -> Result<EnumValue, ParseError> {
        self.expect(Token::Enum)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected enum name, found {:?}", token))),
        };

        let mut value = None;
        let mut description = None;

        if self.peek() == &Token::LeftBrace {
            self.advance();

            while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
                match self.peek() {
                    Token::Identifier(ref id) if id == "value" => {
                        self.advance();
                        value = Some(match self.advance() {
                            Token::Number(n) => n as i32,
                            token => {
                                return Err(self.error(format!(
                                    "Expected enum value number, found {:?}",
                                    token
                                )))
                            }
                        });
                        self.expect(Token::Semicolon)?;
                    }
                    Token::Description => {
                        self.advance();
                        description = Some(match self.advance() {
                            Token::StringLiteral(s) => s,
                            token => {
                                return Err(self.error(format!(
                                    "Expected description string, found {:?}",
                                    token
                                )))
                            }
                        });
                        self.expect(Token::Semicolon)?;
                    }
                    _ => {
                        self.skip_statement()?;
                    }
                }
            }

            self.expect(Token::RightBrace)?;
        } else {
            self.expect(Token::Semicolon)?;
        }

        Ok(EnumValue {
            name,
            value,
            description,
        })
    }

    /// Parse grouping statement: grouping <identifier> { <data-definition-statements> }
    fn parse_grouping(&mut self) -> Result<Grouping, ParseError> {
        self.expect(Token::Grouping)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected grouping name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut data_nodes = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Container => {
                    data_nodes.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    data_nodes.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    data_nodes.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    data_nodes.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Uses => {
                    data_nodes.push(DataNode::Uses(self.parse_uses()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Grouping {
            name,
            description,
            data_nodes,
        })
    }

    /// Parse container statement: container <identifier> { <statements> }
    fn parse_container(&mut self) -> Result<Container, ParseError> {
        self.expect(Token::Container)?;

        let name = self.parse_identifier_or_keyword()?;

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut config = true;
        let mut mandatory = false;
        let mut children = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Mandatory => {
                    self.advance();
                    mandatory = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Container => {
                    children.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    children.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    children.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    children.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    children.push(DataNode::Choice(self.parse_choice()?));
                }
                Token::Uses => {
                    children.push(DataNode::Uses(self.parse_uses()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Container {
            name,
            description,
            config,
            mandatory,
            children,
        })
    }

    /// Parse list statement: list <identifier> { <statements> }
    fn parse_list(&mut self) -> Result<List, ParseError> {
        self.expect(Token::List)?;

        let name = self.parse_identifier_or_keyword()?;

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut config = true;
        let mut keys = Vec::new();
        let mut children = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Key => {
                    self.advance();
                    let key_str = match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        token => {
                            return Err(self.error(format!("Expected key value, found {:?}", token)))
                        }
                    };
                    // Split space-separated keys
                    keys.extend(key_str.split_whitespace().map(String::from));
                    self.expect(Token::Semicolon)?;
                }
                Token::Container => {
                    children.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    children.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    children.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    children.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    children.push(DataNode::Choice(self.parse_choice()?));
                }
                Token::Uses => {
                    children.push(DataNode::Uses(self.parse_uses()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(List {
            name,
            description,
            config,
            keys,
            children,
        })
    }

    /// Parse leaf statement: leaf <identifier> { type <type-spec>; <statements> }
    fn parse_leaf(&mut self) -> Result<Leaf, ParseError> {
        self.expect(Token::Leaf)?;

        let name = self.parse_identifier_or_keyword()?;

        self.expect(Token::LeftBrace)?;

        let mut type_spec = None;
        let mut description = None;
        let mut mandatory = false;
        let mut default = None;
        let mut config = true;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Type => {
                    type_spec = Some(self.parse_type_spec()?);
                }
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Mandatory => {
                    self.advance();
                    mandatory = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Default => {
                    self.advance();
                    default = Some(match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        Token::Number(n) => n.to_string(),
                        token => {
                            return Err(
                                self.error(format!("Expected default value, found {:?}", token))
                            )
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let type_spec = type_spec
            .ok_or_else(|| self.error("Missing required 'type' statement in leaf".to_string()))?;

        Ok(Leaf {
            name,
            description,
            type_spec,
            mandatory,
            default,
            config,
        })
    }

    /// Parse leaf-list statement: leaf-list <identifier> { type <type-spec>; <statements> }
    fn parse_leaf_list(&mut self) -> Result<LeafList, ParseError> {
        self.expect(Token::LeafList)?;

        let name = self.parse_identifier_or_keyword()?;

        self.expect(Token::LeftBrace)?;

        let mut type_spec = None;
        let mut description = None;
        let mut config = true;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Type => {
                    type_spec = Some(self.parse_type_spec()?);
                }
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let type_spec = type_spec.ok_or_else(|| {
            self.error("Missing required 'type' statement in leaf-list".to_string())
        })?;

        Ok(LeafList {
            name,
            description,
            type_spec,
            config,
        })
    }

    /// Parse choice statement: choice <identifier> { <case-statements> }
    fn parse_choice(&mut self) -> Result<Choice, ParseError> {
        self.expect(Token::Choice)?;

        let name = self.parse_identifier_or_keyword()?;

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut mandatory = false;
        let mut cases = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Mandatory => {
                    self.advance();
                    mandatory = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Case => {
                    cases.push(self.parse_case()?);
                }
                // Shorthand: data nodes directly in choice are implicitly wrapped in a case
                Token::Container | Token::List | Token::Leaf | Token::LeafList => {
                    let data_node = match self.peek() {
                        Token::Container => DataNode::Container(self.parse_container()?),
                        Token::List => DataNode::List(self.parse_list()?),
                        Token::Leaf => DataNode::Leaf(self.parse_leaf()?),
                        Token::LeafList => DataNode::LeafList(self.parse_leaf_list()?),
                        _ => unreachable!(),
                    };
                    // Create an implicit case with the same name as the data node
                    let case_name = match &data_node {
                        DataNode::Container(c) => c.name.clone(),
                        DataNode::List(l) => l.name.clone(),
                        DataNode::Leaf(l) => l.name.clone(),
                        DataNode::LeafList(l) => l.name.clone(),
                        _ => unreachable!(),
                    };
                    cases.push(Case {
                        name: case_name,
                        description: None,
                        data_nodes: vec![data_node],
                    });
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Choice {
            name,
            description,
            mandatory,
            cases,
        })
    }

    /// Parse case statement: case <identifier> { <data-definition-statements> }
    fn parse_case(&mut self) -> Result<Case, ParseError> {
        self.expect(Token::Case)?;

        let name = self.parse_identifier_or_keyword()?;

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut data_nodes = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    description = Some(self.parse_description_statement()?);
                }
                Token::Container => {
                    data_nodes.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    data_nodes.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    data_nodes.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    data_nodes.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    data_nodes.push(DataNode::Choice(self.parse_choice()?));
                }
                Token::Uses => {
                    data_nodes.push(DataNode::Uses(self.parse_uses()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Case {
            name,
            description,
            data_nodes,
        })
    }

    /// Parse uses statement: uses <identifier> [{ <statements> }]
    fn parse_uses(&mut self) -> Result<Uses, ParseError> {
        self.expect(Token::Uses)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected grouping name, found {:?}", token))),
        };

        let mut description = None;

        // Check for optional body
        if self.peek() == &Token::LeftBrace {
            self.advance();

            while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
                match self.peek() {
                    Token::Description => {
                        self.advance();
                        description = Some(match self.advance() {
                            Token::StringLiteral(s) => s,
                            token => {
                                return Err(self.error(format!(
                                    "Expected description string, found {:?}",
                                    token
                                )))
                            }
                        });
                        self.expect(Token::Semicolon)?;
                    }
                    _ => {
                        self.skip_statement()?;
                    }
                }
            }

            self.expect(Token::RightBrace)?;
        } else {
            self.expect(Token::Semicolon)?;
        }

        Ok(Uses { name, description })
    }

    /// Skip an unknown statement (consume until semicolon or closing brace).
    fn skip_statement(&mut self) -> Result<(), ParseError> {
        // Skip the statement keyword
        self.advance();

        // Skip any arguments (identifiers, strings, numbers, etc.) until we find a semicolon or block
        loop {
            match self.peek() {
                Token::Semicolon => {
                    self.advance();
                    return Ok(());
                }
                Token::LeftBrace => {
                    self.advance();
                    self.skip_block()?;
                    return Ok(());
                }
                Token::RightBrace | Token::Eof => {
                    // End of current block or file
                    return Ok(());
                }
                _ => {
                    // Skip any other token (identifier, string, number, etc.)
                    self.advance();
                }
            }
        }
    }

    /// Skip a block (everything between { and }).
    fn skip_block(&mut self) -> Result<(), ParseError> {
        let mut depth = 1;
        while depth > 0 && self.peek() != &Token::Eof {
            match self.advance() {
                Token::LeftBrace => depth += 1,
                Token::RightBrace => depth -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    /// Skip until semicolon.
    fn skip_until_semicolon(&mut self) -> Result<(), ParseError> {
        while self.peek() != &Token::Semicolon && self.peek() != &Token::Eof {
            self.advance();
        }
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
