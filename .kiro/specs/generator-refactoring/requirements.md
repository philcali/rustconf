# Generator Module Refactoring

## Overview

Refactor the rustconf generator module to improve maintainability, reduce code duplication, and establish cleaner separation of concerns before adding new features. The generator module has grown to 1,650 lines with significant duplication and mixed responsibilities.

## Goals

1. **Improve Maintainability**: Split large modules into focused, single-responsibility components
2. **Reduce Duplication**: Extract common patterns and create reusable abstractions
3. **Simplify Testing**: Consolidate test files to reduce navigation overhead
4. **Enable Future Growth**: Create a structure that makes adding new features easier

## User Stories

### 1. Module Organization

**As a developer**, I want the generator module split into focused submodules, so that I can easily find and modify specific code generation logic without navigating a 1,650-line file.

**Acceptance Criteria:**
- 1.1. The main `generator/mod.rs` file is reduced to ~200 lines (orchestration only)
- 1.2. Type generation logic (structs, enums, choices) is extracted to `generator/types.rs`
- 1.3. CRUD and RPC operations generation is extracted to `generator/operations.rs`
- 1.4. Notification generation is extracted to `generator/notifications.rs`
- 1.5. Path helper generation is extracted to `generator/paths.rs`
- 1.6. All existing tests pass after the refactoring
- 1.7. Public API remains unchanged (no breaking changes)

### 2. Test Consolidation

**As a developer**, I want related test files consolidated, so that I can write and find tests more easily without switching between multiple files.

**Acceptance Criteria:**
- 2.1. `crud_tests.rs`, `rpc_tests.rs`, and `notification_tests.rs` are merged into `tests.rs` as submodules
- 2.2. Integration test files are consolidated into a single `integration_tests.rs` file
- 2.3. URL path tests remain in `url_path_tests.rs` (focused enough to stay separate)
- 2.4. All existing tests continue to pass
- 2.5. Test organization uses `mod` blocks for logical grouping

### 3. Code Generation Abstraction

**As a developer**, I want to use the existing `formatting.rs` module abstractions instead of manual string building, so that generated code is cleaner and more maintainable.

**Acceptance Criteria:**
- 3.1. Type generation (structs, enums) uses `formatting::generate_struct()` and `formatting::generate_enum()`
- 3.2. Manual string concatenation is replaced with `quote!` macros where appropriate
- 3.3. Generated code formatting is consistent and uses `prettyplease`
- 3.4. All existing tests pass with identical or improved output
- 3.5. Code generation logic is more readable and less error-prone

### 4. CRUD Operation Pattern Extraction

**As a developer**, I want CRUD operation generation to use shared patterns, so that I can avoid duplication and make changes in one place.

**Acceptance Criteria:**
- 4.1. A generic `CrudOperation` enum is created (GET, PUT, PATCH, DELETE, POST)
- 4.2. A generic `ResourceType` enum is created (Container, List, ListItem)
- 4.3. Common CRUD generation logic is extracted to a shared function
- 4.4. Container and List CRUD generation reuse the common logic
- 4.5. Generated CRUD operations remain functionally identical
- 4.6. Code duplication in operations generation is reduced by at least 40%

### 5. Parser Visitor Pattern

**As a developer**, I want a visitor pattern for traversing data nodes, so that validation and transformation logic doesn't need to be duplicated across multiple methods.

**Acceptance Criteria:**
- 5.1. A `DataNodeVisitor` trait is created with visit methods for each node type
- 5.2. A `walk_data_nodes()` function implements the traversal logic
- 5.3. Existing validation methods are refactored to use the visitor pattern
- 5.4. At least 3 validation methods use the new visitor pattern
- 5.5. All parser tests continue to pass
- 5.6. Code duplication in parser validation is reduced by at least 30%

### 6. Validation Type Collection Simplification

**As a developer**, I want validation type collection to use a cleaner pattern, so that the recursive traversal logic is easier to understand and maintain.

**Acceptance Criteria:**
- 6.1. Validation type collection uses the visitor pattern
- 6.2. The `collect_validated_types_from_node()` method is simplified
- 6.3. Recursive traversal logic is centralized
- 6.4. All validation tests continue to pass
- 6.5. Generated validation types remain identical

## Non-Goals

- Changing the public API of the generator module
- Modifying the generated code output (should remain identical)
- Adding new features (this is purely refactoring)
- Changing the parser's AST structure
- Modifying the build module (already well-structured)

## Success Metrics

- **Code Reduction**: Generator module reduced from 1,650 lines to ~200 lines in main file
- **Test Consolidation**: 6+ test files reduced to 3 focused test files
- **Duplication Reduction**: At least 30% reduction in duplicated code patterns
- **Test Coverage**: 100% of existing tests pass after refactoring
- **No Breaking Changes**: Public API remains completely unchanged

## Technical Constraints

- Must maintain backward compatibility with existing code
- All existing tests must pass without modification
- Generated code output must remain identical (byte-for-byte where possible)
- Refactoring must be done incrementally to allow for testing at each step
- Must not introduce new dependencies

## Dependencies

- Existing `formatting.rs` module
- Existing `quote` and `syn` dependencies
- Existing test suite

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing functionality | High | Run full test suite after each change; use feature flags if needed |
| Introducing subtle bugs in code generation | High | Compare generated output before/after; add snapshot tests |
| Making code harder to understand | Medium | Add comprehensive documentation; use clear naming |
| Incomplete refactoring | Low | Break into small, independently valuable tasks |

## Timeline Estimate

- Module splitting: 4-6 hours
- Test consolidation: 2-3 hours
- Formatting abstraction: 3-4 hours
- CRUD pattern extraction: 3-4 hours
- Parser visitor pattern: 4-5 hours
- Validation simplification: 2-3 hours

**Total: 18-25 hours** (can be done incrementally)

## Open Questions

1. Should we use feature flags to enable the refactored code gradually?
2. Should we add snapshot testing to ensure generated code remains identical?
3. Should we refactor the parser module in the same effort or as a separate spec?
4. Do we want to maintain the old code temporarily for comparison?

## References

- Current generator module: `rustconf/src/generator/mod.rs` (1,650 lines)
- Current test files: `rustconf/src/generator/*_tests.rs` (~2,500 lines total)
- Formatting module: `rustconf/src/generator/formatting.rs` (362 lines)
- Parser module: `rustconf/src/parser/mod.rs` (2,044 lines)
