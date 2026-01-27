# Generator Module Refactoring - Tasks

## Phase 1: Module Splitting

### 1. Create Module Structure (Refs: Requirements 1.1-1.7)
- [x] 1.1 Create `rustconf/src/generator/types.rs` with `TypeGenerator` struct
- [x] 1.2 Create `rustconf/src/generator/operations.rs` with `OperationsGenerator` struct
- [x] 1.3 Create `rustconf/src/generator/notifications.rs` with `NotificationGenerator` struct
- [x] 1.4 Create `rustconf/src/generator/paths.rs` with `PathGenerator` struct
- [x] 1.5 Add module declarations to `rustconf/src/generator/mod.rs`
- [x] 1.6 Run `cargo build` to verify structure compiles

### 2. Extract Type Generation Logic (Refs: Requirements 1.2)
- [x] 2.1 Move `generate_typedef()` to `TypeGenerator` (keep public temporarily)
- [x] 2.2 Move `generate_data_node()` to `TypeGenerator`
- [x] 2.3 Move `generate_container()` to `TypeGenerator`
- [x] 2.4 Move `generate_list()` to `TypeGenerator`
- [x] 2.5 Move `generate_choice()` to `TypeGenerator`
- [x] 2.6 Move `generate_case_struct()` to `TypeGenerator`
- [x] 2.7 Move `generate_field()` to `TypeGenerator`
- [x] 2.8 Move `generate_leaf_type()` to `TypeGenerator`
- [x] 2.9 Move `needs_validation()` to `TypeGenerator`
- [x] 2.10 Move `get_validated_type_name()` to `TypeGenerator`
- [x] 2.11 Move `constraint_hash()` to `TypeGenerator`
- [x] 2.12 Update `CodeGenerator::generate_module_content()` to use `TypeGenerator`
- [x] 2.13 Run all tests to verify functionality

### 3. Extract Path Generation Logic (Refs: Requirements 1.5)
- [x] 3.1 Move `generate_container_path_helper()` to `PathGenerator`
- [x] 3.2 Move `generate_list_path_helpers()` to `PathGenerator`
- [x] 3.3 Move `generate_list_key_params()` to `PathGenerator`
- [x] 3.4 Move `find_key_type()` to `PathGenerator`
- [x] 3.5 Move `generate_key_param_names()` to `PathGenerator`
- [x] 3.6 Move `generate_percent_encode_helper()` to `PathGenerator`
- [x] 3.7 Update operations generation to use `PathGenerator`
- [x] 3.8 Run all tests to verify functionality

### 4. Extract Operations Generation Logic (Refs: Requirements 1.3)
- [x] 4.1 Create `CrudOperation` enum in `operations.rs`
- [x] 4.2 Create `ResourceType` enum in `operations.rs`
- [x] 4.3 Move `generate_operations_module()` to `OperationsGenerator`
- [x] 4.4 Move `generate_crud_operations()` to `OperationsGenerator`
- [x] 4.5 Move `generate_node_crud_operations()` to `OperationsGenerator`
- [x] 4.6 Move `generate_container_crud_operations()` to `OperationsGenerator`
- [x] 4.7 Move `generate_list_crud_operations()` to `OperationsGenerator`
- [x] 4.8 Move `generate_rpc_types()` to `OperationsGenerator`
- [x] 4.9 Move `generate_rpc_function()` to `OperationsGenerator`
- [x] 4.10 Move `generate_rpc_error()` to `OperationsGenerator`
- [x] 4.11 Update `CodeGenerator::generate_module_content()` to use `OperationsGenerator`
- [x] 4.12 Run all tests to verify functionality

### 5. Extract Notification Generation Logic (Refs: Requirements 1.4)
- [x] 5.1 Move `generate_notifications()` to `NotificationGenerator`
- [x] 5.2 Move `generate_notification_type()` to `NotificationGenerator`
- [x] 5.3 Update `CodeGenerator::generate_module_content()` to use `NotificationGenerator`
- [x] 5.4 Run all tests to verify functionality

### 6. Clean Up Main Module (Refs: Requirements 1.1, 1.6, 1.7)
- [x] 6.1 Remove moved methods from `generator/mod.rs`
- [x] 6.2 Make sub-generator methods private where appropriate
- [x] 6.3 Verify `generator/mod.rs` is ~200 lines (down from 1,651)
- [x] 6.4 Add module-level documentation
- [x] 6.5 Run all tests to verify functionality
- [x] 6.6 Run `cargo clippy` to check for issues

## Phase 2: Test Consolidation

### 7. Create Consolidated Test Structure (Refs: Requirements 2.1-2.5)
- [x] 7.1 Update `rustconf/src/generator/tests.rs` with submodule structure
- [x] 7.2 Add `mod type_generation` submodule
- [x] 7.3 Add `mod crud_operations` submodule
- [x] 7.4 Add `mod rpc_operations` submodule
- [x] 7.5 Add `mod notifications` submodule
- [x] 7.6 Add `mod integration` submodule

### 8. Migrate Type Generation Tests (Refs: Requirements 2.1, 2.4)
- [x] 8.1 Copy relevant tests from existing `tests.rs` to `tests::type_generation`
- [x] 8.2 Update imports and paths as needed
- [x] 8.3 Run tests to verify they pass

### 9. Migrate CRUD Tests (Refs: Requirements 2.1, 2.4)
- [x] 9.1 Copy all tests from `crud_tests.rs` to `tests::crud_operations`
- [x] 9.2 Update imports and paths as needed
- [x] 9.3 Run tests to verify they pass
- [x] 9.4 Delete `crud_tests.rs`

### 10. Migrate RPC Tests (Refs: Requirements 2.1, 2.4)
- [x] 10.1 Copy all tests from `rpc_tests.rs` to `tests::rpc_operations`
- [x] 10.2 Update imports and paths as needed
- [x] 10.3 Run tests to verify they pass
- [x] 10.4 Delete `rpc_tests.rs`

### 11. Migrate Notification Tests (Refs: Requirements 2.1, 2.4)
- [x] 11.1 Copy all tests from `notification_tests.rs` to `tests::notifications`
- [x] 11.2 Update imports and paths as needed
- [x] 11.3 Run tests to verify they pass
- [x] 11.4 Delete `notification_tests.rs`

### 12. Migrate Integration Tests (Refs: Requirements 2.2, 2.4)
- [x] 12.1 Copy tests from `integration_test.rs` to `tests::integration`
- [x] 12.2 Copy tests from `rpc_integration_test.rs` to `tests::integration`
- [x] 12.3 Copy tests from `notification_integration_test.rs` to `tests::integration`
- [x] 12.4 Update imports and paths as needed
- [x] 12.5 Run tests to verify they pass
- [x] 12.6 Delete `integration_test.rs`, `rpc_integration_test.rs`, `notification_integration_test.rs`

### 13. Clean Up Old Test Files (Refs: Requirements 2.3, 2.4)
- [x] 13.1 Remove old test module declarations from `mod.rs`
- [x] 13.2 Add new consolidated test module declaration
- [x] 13.3 Verify all tests still pass
- [x] 13.4 Update test count documentation if needed

## Phase 3: Formatting Integration

### 14. Integrate Formatting for Type Aliases (Refs: Requirements 3.1, 3.4)
- [x] 14.1 Update `TypeGenerator::generate_typedef()` to use `formatting::generate_type_alias()`
- [x] 14.2 Handle `syn::Type` conversion for base types
- [x] 14.3 Run tests to verify output is identical
- [x] 14.4 Compare generated code before/after

### 15. Extend Formatting Module for Serde (Refs: Requirements 3.2, 3.3)
- [x] 15.1 Add `StructField` type to `formatting.rs` with serde attributes
- [x] 15.2 Add `generate_struct_with_serde()` function
- [x] 15.3 Add tests for new formatting functions
- [x] 15.4 Document the new API

### 16. Integrate Formatting for Structs (Refs: Requirements 3.1, 3.4, 3.5)
- [ ] 16.1 Update `TypeGenerator::generate_container()` to use formatting module
- [ ] 16.2 Update `TypeGenerator::generate_list()` to use formatting module
- [ ] 16.3 Update `TypeGenerator::generate_case_struct()` to use formatting module
- [ ] 16.4 Run tests to verify output is identical
- [ ] 16.5 Compare generated code before/after

### 17. Integrate Formatting for Enums (Refs: Requirements 3.1, 3.4, 3.5)
- [ ] 17.1 Update `TypeGenerator::generate_choice()` to use `formatting::generate_enum()`
- [ ] 17.2 Handle serde attributes for enums
- [ ] 17.3 Run tests to verify output is identical
- [ ] 17.4 Compare generated code before/after

## Phase 4: CRUD Pattern Extraction

### 18. Implement CRUD Abstractions (Refs: Requirements 4.1, 4.2)
- [ ] 18.1 Implement `CrudOperation` enum with all variants (GET, PUT, PATCH, DELETE, POST)
- [ ] 18.2 Implement `ResourceType` enum with all variants (Container, Collection, Item)
- [ ] 18.3 Add helper methods to `CrudOperation` (http_method, function_prefix, etc.)
- [ ] 18.4 Add tests for enum behavior

### 19. Extract Common CRUD Generation (Refs: Requirements 4.3)
- [ ] 19.1 Implement `generate_crud_operation()` generic function
- [ ] 19.2 Implement `operation_function_name()` helper
- [ ] 19.3 Implement `operation_parameters()` helper
- [ ] 19.4 Implement `operation_return_type()` helper
- [ ] 19.5 Implement `generate_operation_body()` helper
- [ ] 19.6 Implement `generate_operation_doc()` helper

### 20. Refactor Container CRUD Operations (Refs: Requirements 4.4, 4.5, 4.6)
- [ ] 20.1 Update `generate_container_crud_operations()` to use generic function
- [ ] 20.2 Remove duplicated code
- [ ] 20.3 Run tests to verify output is identical
- [ ] 20.4 Measure code reduction

### 21. Refactor List CRUD Operations (Refs: Requirements 4.4, 4.5, 4.6)
- [ ] 21.1 Update `generate_list_crud_operations()` to use generic function
- [ ] 21.2 Remove duplicated code
- [ ] 21.3 Run tests to verify output is identical
- [ ] 21.4 Measure code reduction

### 22. Verify CRUD Improvements (Refs: Requirements 4.6)
- [ ] 22.1 Run all CRUD tests
- [ ] 22.2 Compare generated code before/after
- [ ] 22.3 Verify at least 40% code reduction in operations.rs
- [ ] 22.4 Document the new pattern

## Phase 5: Parser Visitor Pattern

### 23. Implement Visitor Trait (Refs: Requirements 5.1, 5.2)
- [ ] 23.1 Create `DataNodeVisitor` trait in `parser/mod.rs`
- [ ] 23.2 Implement default methods for each node type (leaf, leaf_list, container, list, choice, case, uses)
- [ ] 23.3 Implement `walk_data_node()` function
- [ ] 23.4 Implement `walk_data_nodes()` function
- [ ] 23.5 Add tests for visitor pattern

### 24. Refactor Typedef Validation (Refs: Requirements 5.3, 5.4, 5.5)
- [ ] 24.1 Create `TypedefRefValidator` struct implementing `DataNodeVisitor`
- [ ] 24.2 Update `validate_typedef_references()` to use visitor
- [ ] 24.3 Remove old `validate_data_node_typedef_references()` method
- [ ] 24.4 Run validation tests to verify behavior

### 25. Refactor Grouping Validation (Refs: Requirements 5.3, 5.4, 5.5)
- [ ] 25.1 Create `GroupingRefValidator` struct implementing `DataNodeVisitor`
- [ ] 25.2 Update `validate_grouping_references()` to use visitor
- [ ] 25.3 Remove old `validate_data_node_grouping_references()` method
- [ ] 25.4 Run validation tests to verify behavior

### 26. Refactor Leafref Validation (Refs: Requirements 5.3, 5.4, 5.5)
- [ ] 26.1 Create `LeafrefPathValidator` struct implementing `DataNodeVisitor`
- [ ] 26.2 Update `validate_leafref_paths()` to use visitor
- [ ] 26.3 Remove old `validate_data_node_leafref_paths()` method
- [ ] 26.4 Run validation tests to verify behavior

### 27. Refactor Constraint Validation (Refs: Requirements 5.3, 5.4, 5.5, 5.6)
- [ ] 27.1 Create `ConstraintValidator` struct implementing `DataNodeVisitor`
- [ ] 27.2 Update `validate_data_node_constraints()` to use visitor
- [ ] 27.3 Remove old recursive constraint validation methods
- [ ] 27.4 Run validation tests to verify behavior

### 28. Verify Parser Improvements (Refs: Requirements 5.6)
- [ ] 28.1 Run all parser tests
- [ ] 28.2 Verify at least 30% code reduction in parser/mod.rs
- [ ] 28.3 Document the visitor pattern usage

## Phase 6: Validation Type Collection

### 29. Implement Validation Collector Visitor (Refs: Requirements 6.1, 6.2)
- [ ] 29.1 Create `ValidationTypeCollector` struct implementing `DataNodeVisitor`
- [ ] 29.2 Implement `visit_leaf()` to collect validated types
- [ ] 29.3 Implement `visit_leaf_list()` to collect validated types
- [ ] 29.4 Add helper methods for type collection

### 30. Refactor Validation Type Collection (Refs: Requirements 6.3, 6.4, 6.5)
- [ ] 30.1 Update `collect_validated_types()` to use visitor
- [ ] 30.2 Remove old `collect_validated_types_from_node()` method
- [ ] 30.3 Run tests to verify collected types are identical
- [ ] 30.4 Verify generated validation code is unchanged

## Final Verification

### 31. Comprehensive Testing
- [ ] 31.1 Run full test suite: `cargo test`
- [ ] 31.2 Run clippy: `cargo clippy --all-targets`
- [ ] 31.3 Run format check: `cargo fmt --check`
- [ ] 31.4 Build examples: `cargo build --examples`
- [ ] 31.5 Run example integration tests

### 32. Code Metrics Verification
- [ ] 32.1 Verify `generator/mod.rs` is ~200 lines (down from 1,651)
- [ ] 32.2 Verify test files reduced from 9 to 3 (tests.rs, url_path_tests.rs, url_path_example_test.rs)
- [ ] 32.3 Measure code duplication reduction
- [ ] 32.4 Document metrics in completion report

### 33. Documentation Updates
- [ ] 33.1 Update module-level documentation in all new modules
- [ ] 33.2 Add examples to public API documentation
- [ ] 33.3 Update README if needed
- [ ] 33.4 Add migration guide for contributors

### 34. Final Review
- [ ] 34.1 Review all changes for consistency
- [ ] 34.2 Verify no breaking changes to public API
- [ ] 34.3 Verify generated code is identical (or intentionally improved)
- [ ] 34.4 Create summary of improvements
- [ ] 34.5 Mark spec as complete

## Notes

- Each phase can be committed independently
- Run tests after each major change
- Compare generated code output before/after each phase
- Document any unexpected issues or improvements
- Keep track of code metrics throughout the process
- Current state: `generator/mod.rs` is 1,651 lines with 9 test files
- Target state: `generator/mod.rs` ~200 lines with 3 test files
