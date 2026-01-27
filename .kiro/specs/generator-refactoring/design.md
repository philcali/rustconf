# Generator Module Refactoring - Design Document

## Architecture Overview

This refactoring reorganizes the generator module from a monolithic structure into a modular, maintainable architecture while preserving all existing functionality and public APIs.

### Current Structure

```
generator/
├── mod.rs (1,650 lines - everything mixed together)
├── config.rs
├── error.rs
├── formatting.rs (underutilized)
├── naming.rs
├── validation.rs
└── tests/ (6+ separate test files)
```

### Target Structure

```
generator/
├── mod.rs (~200 lines - orchestration & public API)
├── config.rs (unchanged)
├── error.rs (unchanged)
├── formatting.rs (unchanged)
├── naming.rs (unchanged)
├── validation.rs (unchanged)
├── types.rs (struct/enum/choice generation)
├── operations.rs (CRUD + RPC generation)
├── notifications.rs (notification generation)
├── paths.rs (path helper generation)
└── tests.rs (consolidated tests with submodules)
```

## Detailed Design

### 1. Module Splitting Strategy

#### 1.1 New Module: `generator/types.rs`

**Responsibility**: Generate Rust type definitions (structs, enums, type aliases) from YANG data nodes.

**Extracted Methods**:
- `generate_typedef()` - Generate type aliases
- `generate_data_node()` - Dispatch to specific generators
- `generate_container()` - Generate struct from container
- `generate_list()` - Generate struct from list
- `generate_choice()` - Generate enum from choice
- `generate_case_struct()` - Generate struct for case data
- `generate_field()` - Generate struct field
- `generate_leaf_type()` - Generate Rust type from YANG type

**New Structure**:
```rust
pub struct TypeGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> TypeGenerator<'a> {
    pub fn new(config: &'a GeneratorConfig) -> Self { ... }
    
    pub fn generate_typedef(&self, typedef: &TypeDef) -> Result<String, GeneratorError> { ... }
    pub fn generate_data_node(&self, node: &DataNode, module: &YangModule) -> Result<String, GeneratorError> { ... }
    // ... other methods
}
```

#### 1.2 New Module: `generator/operations.rs`

**Responsibility**: Generate CRUD operations and RPC functions.

**Extracted Methods**:
- `generate_operations_module()` - Main operations module
- `generate_crud_operations()` - CRUD operations wrapper
- `generate_node_crud_operations()` - Dispatch CRUD generation
- `generate_container_crud_operations()` - Container CRUD
- `generate_list_crud_operations()` - List CRUD
- `generate_rpc_types()` - RPC input/output types
- `generate_rpc_function()` - RPC function
- `generate_rpc_error()` - RPC error type
- `generate_percent_encode_helper()` - URL encoding helper

**New Abstractions**:
```rust
#[derive(Debug, Clone, Copy)]
pub enum CrudOperation {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Container,
    Collection,  // List collection
    Item,        // List item
}

pub struct OperationsGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> OperationsGenerator<'a> {
    pub fn new(config: &'a GeneratorConfig) -> Self { ... }
    
    pub fn generate_operations_module(&self, module: &YangModule) -> Result<String, GeneratorError> { ... }
    
    fn generate_crud_operation(
        &self,
        operation: CrudOperation,
        resource_type: ResourceType,
        resource_name: &str,
        type_name: &str,
        path_helper: &str,
        // ...
    ) -> String { ... }
}
```

#### 1.3 New Module: `generator/notifications.rs`

**Responsibility**: Generate notification types and related code.

**Extracted Methods**:
- `generate_notifications()` - Notifications module wrapper
- `generate_notification_type()` - Individual notification struct

**New Structure**:
```rust
pub struct NotificationGenerator<'a> {
    config: &'a GeneratorConfig,
    type_generator: TypeGenerator<'a>,
}

impl<'a> NotificationGenerator<'a> {
    pub fn new(config: &'a GeneratorConfig) -> Self { ... }
    
    pub fn generate_notifications(&self, module: &YangModule) -> Result<String, GeneratorError> { ... }
    fn generate_notification_type(&self, notification: &Notification, module: &YangModule) -> Result<String, GeneratorError> { ... }
}
```

#### 1.4 New Module: `generator/paths.rs`

**Responsibility**: Generate RESTCONF URL path helpers.

**Extracted Methods**:
- `generate_container_path_helper()` - Container path
- `generate_list_path_helpers()` - List collection and item paths
- `generate_list_key_params()` - Key parameter list
- `find_key_type()` - Find key field type
- `generate_key_param_names()` - Key parameter names

**New Structure**:
```rust
pub struct PathGenerator<'a> {
    config: &'a GeneratorConfig,
}

impl<'a> PathGenerator<'a> {
    pub fn new(config: &'a GeneratorConfig) -> Self { ... }
    
    pub fn generate_container_path(&self, container: &Container, module: &YangModule) -> Result<String, GeneratorError> { ... }
    pub fn generate_list_paths(&self, list: &List, module: &YangModule) -> Result<String, GeneratorError> { ... }
    
    fn build_restconf_path(&self, module: &YangModule, resource_name: &str) -> String { ... }
}
```

#### 1.5 Updated Module: `generator/mod.rs`

**Responsibility**: Orchestration, public API, and high-level generation flow.

**Retained Methods**:
- `CodeGenerator::new()` - Constructor
- `CodeGenerator::generate()` - Main entry point
- `CodeGenerator::write_files()` - File writing
- `generate_module_content()` - Orchestrates generation
- Helper methods: `generate_rustdoc()`, `generate_derive_attributes()`, etc.

**New Structure**:
```rust
pub struct CodeGenerator {
    config: GeneratorConfig,
}

impl CodeGenerator {
    pub fn new(config: GeneratorConfig) -> Self { ... }
    
    pub fn generate(&self, module: &YangModule) -> Result<GeneratedCode, GeneratorError> {
        let module_content = self.generate_module_content(module)?;
        // ... create GeneratedCode
    }
    
    fn generate_module_content(&self, module: &YangModule) -> Result<String, GeneratorError> {
        let mut content = String::new();
        
        // Use sub-generators
        let type_gen = TypeGenerator::new(&self.config);
        let ops_gen = OperationsGenerator::new(&self.config);
        let notif_gen = NotificationGenerator::new(&self.config);
        
        // Orchestrate generation
        content.push_str(&self.generate_file_header(module));
        content.push_str(&self.generate_use_statements());
        
        // Generate types
        for typedef in &module.typedefs {
            content.push_str(&type_gen.generate_typedef(typedef)?);
        }
        
        for data_node in &module.data_nodes {
            content.push_str(&type_gen.generate_data_node(data_node, module)?);
        }
        
        // Generate operations
        if !module.rpcs.is_empty() || !module.data_nodes.is_empty() {
            content.push_str(&ops_gen.generate_operations_module(module)?);
        }
        
        // Generate notifications
        if !module.notifications.is_empty() {
            content.push_str(&notif_gen.generate_notifications(module)?);
        }
        
        Ok(content)
    }
    
    // Helper methods remain here
    fn generate_rustdoc(&self, description: &str) -> String { ... }
    fn generate_derive_attributes(&self) -> String { ... }
    fn generate_file_header(&self, module: &YangModule) -> String { ... }
    fn generate_use_statements(&self) -> String { ... }
    fn get_json_field_name(&self, yang_name: &str, module: &YangModule) -> String { ... }
}
```

### 2. Test Consolidation Strategy

#### 2.1 Consolidated Test Structure

```rust
// generator/tests.rs

#[cfg(test)]
mod type_generation {
    // Tests from main tests.rs related to type generation
    #[test]
    fn test_generate_simple_container() { ... }
    
    #[test]
    fn test_generate_list() { ... }
    
    // ... more type tests
}

#[cfg(test)]
mod crud_operations {
    // Tests from crud_tests.rs
    #[test]
    fn test_generate_crud_for_config_container() { ... }
    
    // ... more CRUD tests
}

#[cfg(test)]
mod rpc_operations {
    // Tests from rpc_tests.rs
    #[test]
    fn test_generate_rpc_with_input_and_output() { ... }
    
    // ... more RPC tests
}

#[cfg(test)]
mod notifications {
    // Tests from notification_tests.rs
    #[test]
    fn test_generate_notification_with_data_nodes() { ... }
    
    // ... more notification tests
}

#[cfg(test)]
mod integration {
    // Tests from integration_test.rs, rpc_integration_test.rs, notification_integration_test.rs
    #[test]
    fn test_full_module_generation() { ... }
    
    // ... more integration tests
}
```

#### 2.2 Files to Keep Separate

- `url_path_tests.rs` - Focused enough to remain separate
- `url_path_example_test.rs` - Example/documentation test

### 3. Formatting Module Integration

#### 3.1 Current State

The `formatting.rs` module provides:
- `generate_struct()` - Uses `quote!` and `prettyplease`
- `generate_enum()` - Uses `quote!` and `prettyplease`
- `generate_impl_block()` - Uses `quote!` and `prettyplease`
- `generate_type_alias()` - Uses `quote!` and `prettyplease`

But the main generator uses manual string building.

#### 3.2 Integration Strategy

**Phase 1**: Use formatting module for simple cases
```rust
// Before (manual string building)
fn generate_typedef(&self, typedef: &TypeDef) -> Result<String, GeneratorError> {
    let mut output = String::new();
    output.push_str(&format!("pub type {} = {};\n", type_name, base_type));
    Ok(output)
}

// After (using formatting module)
fn generate_typedef(&self, typedef: &TypeDef) -> Result<String, GeneratorError> {
    let type_name = naming::to_type_name(&typedef.name);
    let base_type: Type = parse_quote!(#base_type_ident);
    
    formatting::generate_type_alias(
        &type_name,
        base_type,
        typedef.description.as_deref(),
    ).map_err(|e| GeneratorError::CodeGeneration(e.to_string()))
}
```

**Phase 2**: Extend formatting module for complex cases
```rust
// Add to formatting.rs
pub fn generate_struct_with_serde(
    name: &str,
    fields: Vec<StructField>,
    derives: Vec<&str>,
    doc_comment: Option<&str>,
) -> Result<String, syn::Error> {
    // Generate struct with serde attributes
}

pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub serde_attrs: Vec<String>,
    pub doc_comment: Option<String>,
}
```

### 4. CRUD Pattern Extraction

#### 4.1 Common CRUD Generation Pattern

```rust
impl<'a> OperationsGenerator<'a> {
    fn generate_crud_operation(
        &self,
        operation: CrudOperation,
        resource_type: ResourceType,
        resource_name: &str,
        type_name: &str,
        path_helper: &str,
        key_params: Option<&str>,
    ) -> String {
        let mut output = String::new();
        
        // Generate documentation
        output.push_str(&self.generate_operation_doc(operation, resource_name));
        
        // Generate function signature
        let fn_name = self.operation_function_name(operation, resource_name, resource_type);
        let params = self.operation_parameters(operation, type_name, key_params);
        let return_type = self.operation_return_type(operation, type_name);
        
        output.push_str(&format!(
            "        pub async fn {}({}) -> {} {{\n",
            fn_name, params, return_type
        ));
        
        // Generate function body
        output.push_str(&self.generate_operation_body(operation, path_helper, key_params));
        
        output.push_str("        }\n\n");
        output
    }
    
    fn operation_function_name(
        &self,
        operation: CrudOperation,
        resource_name: &str,
        resource_type: ResourceType,
    ) -> String {
        let prefix = match operation {
            CrudOperation::Get => "get",
            CrudOperation::Post => "create",
            CrudOperation::Put => "put",
            CrudOperation::Patch => "patch",
            CrudOperation::Delete => "delete",
        };
        
        let suffix = match resource_type {
            ResourceType::Item => "_by_key",
            _ => "",
        };
        
        format!("{}_{}{}", prefix, naming::to_field_name(resource_name), suffix)
    }
    
    // ... other helper methods
}
```

### 5. Parser Visitor Pattern

#### 5.1 Visitor Trait Definition

```rust
// In parser/mod.rs or new parser/visitor.rs

pub trait DataNodeVisitor {
    type Error;
    
    fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn visit_leaf_list(&mut self, leaf_list: &LeafList) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn visit_container(&mut self, container: &Container) -> Result<(), Self::Error> {
        // Default: visit children
        for child in &container.children {
            walk_data_node(child, self)?;
        }
        Ok(())
    }
    
    fn visit_list(&mut self, list: &List) -> Result<(), Self::Error> {
        // Default: visit children
        for child in &list.children {
            walk_data_node(child, self)?;
        }
        Ok(())
    }
    
    fn visit_choice(&mut self, choice: &Choice) -> Result<(), Self::Error> {
        // Default: visit cases
        for case in &choice.cases {
            self.visit_case(case)?;
        }
        Ok(())
    }
    
    fn visit_case(&mut self, case: &Case) -> Result<(), Self::Error> {
        // Default: visit children
        for child in &case.data_nodes {
            walk_data_node(child, self)?;
        }
        Ok(())
    }
    
    fn visit_uses(&mut self, uses: &Uses) -> Result<(), Self::Error> {
        Ok(())
    }
}

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

pub fn walk_data_nodes<V: DataNodeVisitor>(
    nodes: &[DataNode],
    visitor: &mut V,
) -> Result<(), V::Error> {
    for node in nodes {
        walk_data_node(node, visitor)?;
    }
    Ok(())
}
```

#### 5.2 Example Visitor Implementation

```rust
// Typedef reference validator
struct TypedefRefValidator<'a> {
    typedefs: &'a [TypeDef],
}

impl<'a> DataNodeVisitor for TypedefRefValidator<'a> {
    type Error = ParseError;
    
    fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
        Self::validate_typespec_references(&leaf.type_spec, self.typedefs)
    }
    
    fn visit_leaf_list(&mut self, leaf_list: &LeafList) -> Result<(), Self::Error> {
        Self::validate_typespec_references(&leaf_list.type_spec, self.typedefs)
    }
}

// Usage
impl YangParser {
    fn validate_typedef_references(&self, module: &YangModule) -> Result<(), ParseError> {
        let mut validator = TypedefRefValidator {
            typedefs: &module.typedefs,
        };
        
        walk_data_nodes(&module.data_nodes, &mut validator)?;
        Ok(())
    }
}
```

### 6. Validation Type Collection with Visitor

```rust
// In generator/mod.rs or generator/types.rs

struct ValidationTypeCollector<'a> {
    config: &'a GeneratorConfig,
    types: HashMap<String, TypeSpec>,
}

impl<'a> ValidationTypeCollector<'a> {
    fn new(config: &'a GeneratorConfig) -> Self {
        Self {
            config,
            types: HashMap::new(),
        }
    }
    
    fn needs_validation(&self, type_spec: &TypeSpec) -> bool {
        // ... existing logic
    }
    
    fn collect_from_typespec(&mut self, type_spec: &TypeSpec) {
        if self.needs_validation(type_spec) {
            let type_name = self.get_validated_type_name(type_spec);
            self.types.insert(type_name, type_spec.clone());
        }
    }
}

impl<'a> DataNodeVisitor for ValidationTypeCollector<'a> {
    type Error = std::convert::Infallible;
    
    fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
        self.collect_from_typespec(&leaf.type_spec);
        Ok(())
    }
    
    fn visit_leaf_list(&mut self, leaf_list: &LeafList) -> Result<(), Self::Error> {
        self.collect_from_typespec(&leaf_list.type_spec);
        Ok(())
    }
}

// Usage
impl CodeGenerator {
    fn collect_validated_types(&self, module: &YangModule) -> Vec<(String, TypeSpec)> {
        if !self.config.enable_validation {
            return Vec::new();
        }
        
        let mut collector = ValidationTypeCollector::new(&self.config);
        
        // Collect from typedefs
        for typedef in &module.typedefs {
            collector.collect_from_typespec(&typedef.type_spec);
        }
        
        // Collect from data nodes using visitor
        let _ = walk_data_nodes(&module.data_nodes, &mut collector);
        
        collector.types.into_iter().collect()
    }
}
```

## Implementation Plan

### Phase 1: Module Splitting (No Behavior Changes)
1. Create new module files with empty structures
2. Move methods one-by-one, keeping them public temporarily
3. Update `mod.rs` to use new modules
4. Run tests after each move
5. Make methods private once confirmed working

### Phase 2: Test Consolidation
1. Create new `tests.rs` with submodules
2. Copy tests into appropriate submodules
3. Verify all tests pass
4. Delete old test files

### Phase 3: Formatting Integration
1. Start with simple cases (type aliases)
2. Extend formatting module as needed
3. Gradually replace string building

### Phase 4: Pattern Extraction
1. Implement CRUD operation abstraction
2. Refactor container operations
3. Refactor list operations
4. Verify generated code is identical

### Phase 5: Visitor Pattern
1. Implement visitor trait and walk functions
2. Refactor one validation method
3. Refactor remaining validation methods
4. Apply to validation type collection

## Testing Strategy

### Regression Testing
- All existing tests must pass at each phase
- No changes to test assertions (only organization)

### Output Comparison
- Generate code before and after refactoring
- Compare outputs byte-for-byte
- Any differences must be intentional improvements

### Snapshot Testing (Optional)
- Add snapshot tests for generated code
- Helps catch unintended changes

## Rollback Strategy

Each phase is independently valuable and can be committed separately. If issues arise:
1. Revert the specific phase
2. Fix the issue
3. Re-apply the phase

## Success Criteria

- ✅ All existing tests pass
- ✅ Generated code is identical (or intentionally improved)
- ✅ Public API unchanged
- ✅ Code metrics improved (lines, duplication)
- ✅ Developer feedback positive

## Future Enhancements

After this refactoring:
1. Add new features more easily
2. Improve error messages with better context
3. Add more sophisticated code generation patterns
4. Consider template-based generation for complex cases
