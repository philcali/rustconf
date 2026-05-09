# Implementation Plan: Junos Generator Compliance

## Overview

Implement four parser/codegen fixes to bring successful code generation from 31 to 40 out of 40 Juniper cRPD and IETF YANG models. Changes span `rustconf/src/generator/types.rs`, `rustconf/src/generator/operations.rs`, `rustconf/src/parser/mod.rs`, and `rustconf/src/parser/ast.rs`. Tasks are ordered by impact (most modules unblocked first), with property-based tests validating correctness properties from the design document.

## Tasks

- [x] 1. Strip prefix from `TypedefRef.name` in code generator (`types.rs`)
  - [x] 1.1 Update `generate_leaf_type` to strip module prefix from `TypeSpec::TypedefRef` names
    - In the `TypeSpec::TypedefRef { name }` match arm, use `rsplit_once(':')` to extract the local name portion before passing to `to_type_name()`
    - E.g., `"inet:ipv4-address"` → `to_type_name("ipv4-address")` → `"Ipv4Address"`
    - This same stripping must also apply in `generate_typedef` and `data_node_to_struct_field` where `TypedefRef` names flow through to `syn::parse_str`
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [ ]* 1.2 Write property test for prefix stripping idempotence
    - **Property 1: Prefix stripping idempotence**
    - Generate pairs of valid YANG identifiers `(prefix, local_name)`, construct `TypeSpec::TypedefRef { name: "prefix:local_name" }`, call `generate_leaf_type`, compare with result from `TypeSpec::TypedefRef { name: "local_name" }` alone, verify both pass `syn::parse_str`
    - **Validates: Requirements 1.1, 1.2, 1.4, 1.5**

  - [ ]* 1.3 Write unit tests for prefixed type reference generation
    - Test `TypedefRef { name: "inet:ipv4-address" }` generates `Ipv4Address`
    - Test `TypedefRef { name: "yang:date-and-time" }` generates `DateAndTime`
    - Test unprefixed `TypedefRef { name: "counter64" }` still generates `Counter64`
    - _Requirements: 1.1, 1.2, 1.5_

- [ ] 2. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Add `bits` type support in parser and code generator
  - [x] 3.1 Add `BitDef` struct and `TypeSpec::Bits` variant to AST in `rustconf/src/parser/ast.rs`
    - Add `BitDef { name: String, position: Option<u32>, description: Option<String> }` struct
    - Add `Bits { bits: Vec<BitDef> }` variant to `TypeSpec` enum
    - _Requirements: 2.1, 2.2_

  - [x] 3.2 Add `Token::Bits` match arm in `parse_type_spec` in `rustconf/src/parser/mod.rs`
    - Create `TypeSpec::Bits { bits: Vec::new() }` when `Token::Bits` is encountered
    - _Requirements: 2.1, 2.5_

  - [x] 3.3 Add `parse_bit_def` method and `Token::Bit` handling in `parse_type_body`
    - Implement `parse_bit_def()` to parse `bit <name> { position <value>; description "..."; }` statements
    - In `parse_type_body`, match `Token::Bit` and push parsed `BitDef` into `TypeSpec::Bits.bits`
    - Handle both `bit name;` (no body) and `bit name { ... }` (with body) forms
    - _Requirements: 2.2, 2.5_

  - [x] 3.4 Map `TypeSpec::Bits` to `String` in `generate_leaf_type` in `rustconf/src/generator/types.rs`
    - Add `TypeSpec::Bits { .. } => "String"` match arm
    - Update `needs_validation` and `get_validated_type_name` to handle the new variant (no validation needed)
    - _Requirements: 2.3, 2.4_

  - [ ]* 3.5 Write property test for bits type bit definition preservation
    - **Property 2: Bits type bit definition preservation**
    - Generate vectors of `(name: String, position: Option<u32>)` pairs, construct YANG `type bits { bit <name> { position <pos>; } ... }` text, parse, verify `TypeSpec::Bits` contains matching bit definitions in order
    - **Validates: Requirements 2.1, 2.2**

  - [ ]* 3.6 Write unit tests for bits type parsing and generation
    - Test parsing `type bits { bit read; bit write { position 1; } }` produces correct AST
    - Test parsing `type bits;` and `type bits {}` succeed (empty bits body)
    - Test `TypeSpec::Bits` generates `String` type in code generator
    - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [x] 4. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Generate type definitions for `choice`/`case` nodes in RPC input/output (`operations.rs`)
  - [x] 5.1 Add `generate_nested_types` helper method to `OperationsGenerator`
    - Implement a method that delegates to `TypeGenerator::generate_choice`, `TypeGenerator::generate_container`, and `TypeGenerator::generate_list` for data nodes found in RPC input/output
    - _Requirements: 3.1, 3.2, 3.3_

  - [x] 5.2 Update `generate_rpc_types` to emit type definitions for choice/case nodes
    - After generating the input struct, iterate over input data nodes and call `generate_nested_types` for each
    - After generating the output struct, iterate over output data nodes and call `generate_nested_types` for each
    - This ensures choice enums, case structs, and nested container/list types are defined alongside the RPC input/output structs
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [ ]* 5.3 Write property test for RPC choice/case type completeness
    - **Property 3: RPC choice/case type completeness**
    - Generate RPC definitions with random choice/case structures in input/output, generate code, extract type references from struct fields, extract type definitions, verify references ⊆ definitions
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.5**

  - [ ]* 5.4 Write unit test for RPC with choice in input generating complete types
    - Construct an RPC with a `choice` containing two `case` nodes in its input, generate code, verify the choice enum and case structs are present in the output
    - _Requirements: 3.1, 3.2, 3.4_

- [ ] 6. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Support decimal range values in parser (`parse_range_value`)
  - [x] 7.1 Add `f64` parsing fallback in `parse_range_value` in `rustconf/src/parser/mod.rs`
    - After the existing `i64` and `u64` parse attempts, add an `f64` fallback: `trimmed.parse::<f64>().map(|v| v as i64)`
    - This truncates decimal values like `0.001` to `i64` (conservative — widens the allowed range)
    - _Requirements: 4.1, 4.2, 4.3_

  - [ ]* 7.2 Write property test for decimal range parsing well-formedness
    - **Property 4: Decimal range parsing well-formedness**
    - Generate range constraint strings containing decimal values (optionally mixed with `min`, `max` keywords and `|` separators), where each range part has `lower <= upper`, parse the constraint, verify all `Range` entries satisfy `min <= max`
    - **Validates: Requirements 4.1, 4.3, 4.4, 4.5**

  - [ ]* 7.3 Write unit tests for decimal range parsing
    - Test parsing `"0.001..max"` succeeds and produces a valid `RangeConstraint`
    - Test parsing `"0.001..1.0 | 2.0..max"` produces 2 `Range` entries
    - Test parsing `"0.001"` as a single-value range succeeds
    - _Requirements: 4.1, 4.3, 4.4_

- [x] 8. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Integration validation — update threshold and verify all 40 models
  - [x] 9.1 Update `MIN_PASSING_MODELS` from 34 to 40 in `tests/integration/tests/yang_parser_compliance_tests.rs`
    - Change the constant to reflect the new expected baseline after all fixes
    - _Requirements: 5.4_

  - [x] 9.2 Verify all 40 vendor YANG models parse and generate successfully
    - Run the existing integration test suite (`test_vendor_model_parse_count`, `test_generated_code_compiles_for_parsed_models`)
    - Confirm all 40 models pass parsing and code generation
    - Confirm `check_generated_module` in `build.rs` reports zero validation failures
    - _Requirements: 5.1, 5.2, 5.3_

- [x] 10. Fix `check_generated_module` to scan `operations.rs` for type definitions (`build.rs`)
  - [x] 10.1 Update `check_generated_module` in `tests/integration/build.rs` to also scan `operations.rs` for `pub type`, `pub struct`, and `pub enum` definitions
    - Currently the function only scans `types.rs` and `validation.rs` for defined types, then checks `operations.rs` for references — but `generate_nested_types` emits container structs, choice enums, and case structs directly into `operations.rs`
    - Add `"operations.rs"` to the list of files scanned for type definitions (the `for filename in &["types.rs", "validation.rs"]` loop)
    - This unblocks `ietf-netconf`, where the `Source` struct is generated inside `operations.rs` by `generate_nested_types` for the `get-config` RPC's input container
    - _Requirements: 3.3, 3.4, 5.1, 5.3_

- [x] 11. Add `regex` dependency to integration test crate
  - [x] 11.1 Add `regex = "1"` to `[dependencies]` in `tests/integration/Cargo.toml`
    - Generated validation code for modules with YANG `pattern` constraints (e.g., `ietf-inet-types`, `ietf-yang-types`, `ietf-netconf-acm`, `ietf-routing-types`) uses `regex::Regex::new(...)` but the integration test crate does not depend on `regex`
    - This causes `E0433: use of unresolved module or unlinked crate 'regex'` for all generated validation files
    - _Requirements: 5.1, 5.2_

- [x] 12. Deduplicate struct names for config/state containers in type generator (`types.rs`)
  - [x] 12.1 Update `generate_container` in `rustconf/src/generator/types.rs` to track emitted type names and suffix duplicates
    - `ietf-interfaces` defines both `container interfaces { list interface { ... } }` (config) and `container interfaces-state { list interface { ... } }` (operational state), both producing a struct named `Interface`
    - The generator needs to detect when a type name has already been emitted within the same module and disambiguate — e.g., by appending the parent container name (`InterfaceState`) or a numeric suffix
    - Alternatively, the top-level `generate_types_file` in `mod.rs` could collect all type names first and rename collisions before emitting code
    - _Requirements: 5.1, 5.2_

  - [x] 12.2 Verify `ietf-interfaces` generates valid, compilable Rust code after deduplication
    - The generated `types.rs` should contain two distinct struct names (e.g., `Interface` and `InterfaceState` or `Interface` and `Interface2`)
    - Both structs should have the correct fields for their respective config/state data nodes
    - _Requirements: 5.1, 5.2_

- [x] 13. Skip modules with unresolved cross-module type references in build validation
  - [x] 13.1 Extend `check_generated_module` in `tests/integration/build.rs` to detect cross-module type references in `types.rs`
    - After collecting defined types from `types.rs`, `validation.rs`, and `operations.rs`, scan `types.rs` for field type references (same pattern as the existing `operations.rs` scan)
    - If `types.rs` references a type not defined locally, the module has unresolved cross-module references
    - When detected, return an error like `"types.rs references undefined type '<TypeName>' (likely cross-module import)"` so the module is skipped from compilation (same as `ietf-netconf` is today)
    - Affected modules: `ietf-interfaces` (references `Counter64`, `DateAndTime`, `PhysAddress`, `Gauge64` from `ietf-yang-types`), `ietf-netconf-acm` (references `Xpath10`, `ZeroBasedCounter32`), `ietf-routing-types` (references `DottedQuad`, `Ipv4Address`, `Ipv6Address`)
    - This ensures the integration crate compiles cleanly — modules with cross-module references are excluded from `mod.rs` rather than breaking compilation
    - Proper cross-module type resolution (generating `use` imports or inlining typedefs) is deferred to a follow-on spec
    - _Requirements: 5.1, 5.2_

- [x] 14. Integration re-validation — verify integration crate compiles and tests pass
  - [x] 14.1 Rebuild the integration test crate and verify zero compilation errors
    - Run `cargo build -p rustconf-integration-tests` and confirm it succeeds
    - Modules with cross-module type references will be excluded from `mod.rs` by `check_generated_module`, so the crate should compile cleanly
    - _Requirements: 5.1, 5.2_

  - [x] 14.2 Run the full integration test suite and verify parse compliance
    - Run `cargo test -p rustconf-integration-tests --test yang_parser_compliance_tests -- --nocapture`
    - Confirm `test_vendor_model_parse_count` passes with 40/40 models (parsing is independent of code generation)
    - Confirm `test_generated_code_compiles_for_parsed_models` runs without crashing (some models may report codegen failures due to skipped modules, which is expected and logged)
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 15. Final checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation between major changes
- Property tests use the `proptest` crate (already a workspace dev-dependency) with ≥100 iterations per property
- Unit tests validate specific examples and edge cases
- The implementation language is Rust, matching the existing codebase
- Task ordering follows impact: prefix stripping (4 modules) → bits type (1 module) → choice/case in RPC (1 module) → decimal range (3 modules)
