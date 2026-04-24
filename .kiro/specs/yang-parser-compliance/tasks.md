# Implementation Plan: YANG Parser Compliance

## Overview

Implement six parser/codegen fixes to bring the rustconf YANG parser into compliance with RFC 7950 constructs used in real-world vendor models. Changes span `rustconf/src/parser/mod.rs`, `rustconf/src/parser/ast.rs`, `rustconf/src/generator/types.rs`, and `rustconf/src/generator/naming.rs`. Each task builds incrementally, with property-based tests validating correctness properties from the design document.

## Tasks

- [x] 1. Extend `parse_identifier_or_keyword` to cover all keyword tokens
  - [x] 1.1 Add match arms for every keyword token in `Token::is_keyword()` to `parse_identifier_or_keyword` in `rustconf/src/parser/mod.rs`
    - Map each keyword variant (Module, Submodule, Namespace, Prefix, Import, Include, Revision, YangVersion, Organization, Contact, Description, Reference, Container, List, Leaf, LeafList, Choice, Case, Grouping, Uses, Typedef, Type, Int8–Uint64, String, Boolean, Enumeration, Enum, Binary, Bits, Bit, Union, LeafRef, IdentityRef, Empty, InstanceIdentifier, Config, Mandatory, Default, Status, Units, Range, Length, Pattern, Key, Unique, MinElements, MaxElements, OrderedBy, Must, When, Presence, IfFeature, Feature, Rpc, Input, Output, Notification, Action, Extension, Argument, Augment, Deviation, Deviate, Identity, Base) to its lowercase/kebab-case string
    - _Requirements: 5.1, 5.2, 5.3_

  - [ ]* 1.2 Write property test for keywords as identifiers
    - **Property 5: Keywords accepted as identifiers in name positions**
    - For each YANG keyword token, generate a `leaf <keyword> { type string; }` statement, parse it, and verify the leaf name matches the keyword text
    - **Validates: Requirements 5.1, 5.2, 5.3**

- [x] 2. Update `parse_enum_value` to accept StringLiteral and keyword tokens
  - [x] 2.1 Modify `parse_enum_value` in `rustconf/src/parser/mod.rs` to accept `Token::StringLiteral`, `Token::Identifier`, and keyword tokens as enum names
    - When `peek()` is `StringLiteral`, advance and extract the string value
    - When `peek()` is a keyword token, delegate to `parse_identifier_or_keyword()`
    - Preserve existing `Identifier` handling
    - _Requirements: 1.1, 1.2, 1.3, 6.1_

  - [ ]* 2.2 Write property test for enum name round-trip
    - **Property 1: Enum name round-trip**
    - Generate random valid YANG enum names (alphanumeric identifiers, quoted strings, keyword names), wrap in `type enumeration { enum <name>; }`, parse, verify `EnumValue` AST node name matches
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.4**

  - [ ]* 2.3 Write unit test for wildcard enum `*` parsing
    - Parse `enum "*"` and verify `EnumValue.name` is `"*"`
    - _Requirements: 6.1_

- [ ] 3. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Update `parse_type_spec` and `parse_uses` for prefixed identifiers
  - [x] 4.1 Update `parse_type_spec` in `rustconf/src/parser/mod.rs` to handle `prefix:name` pattern
    - After consuming an `Identifier` token, check if next token is `Colon` followed by another identifier/keyword
    - If so, combine into `"prefix:name"` and store in `TypedefRef { name }`
    - _Requirements: 2.1, 2.3_

  - [x] 4.2 Update `parse_uses` in `rustconf/src/parser/mod.rs` to handle `prefix:name` pattern
    - After consuming the grouping name identifier, check for `Colon` + identifier pattern
    - Combine into `"prefix:name"` for the `Uses.name` field
    - _Requirements: 2.2, 2.3_

  - [x] 4.3 Update `validate_typespec_references` to skip prefixed typedef refs
    - When a `TypedefRef.name` contains `:`, skip local typedef validation since it refers to an imported module
    - _Requirements: 2.4_

  - [ ]* 4.4 Write property test for prefixed identifier preservation
    - **Property 2: Prefixed identifier preservation**
    - Generate pairs of valid YANG identifiers, construct `type prefix:name;` and `uses prefix:name;`, parse, verify the name field contains the exact `"prefix:name"` string
    - **Validates: Requirements 2.1, 2.2, 2.3**

- [x] 5. Add `min`/`max` keyword handling in range and length parsing
  - [x] 5.1 Add `parse_range_value` and `parse_length_value` helper functions in `rustconf/src/parser/mod.rs`
    - `parse_range_value`: map `"min"` → `i64::MIN`, `"max"` → `i64::MAX`, otherwise parse as `i64`
    - `parse_length_value`: map `"min"` → `0`, `"max"` → `u64::MAX`, otherwise parse as `u64`
    - Update `parse_range_string` and `parse_length_string` to use these helpers
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ]* 5.2 Write property test for min/max keyword resolution
    - **Property 3: min/max keyword resolution in constraints**
    - Generate constraint strings mixing `min`, `max`, and integer literals with `..` separator, parse as range and length constraints, verify numeric bounds match type-appropriate extremes
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**

- [x] 6. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Add `TypeSpec::IdentityRef` to AST and parser
  - [x] 7.1 Add `IdentityRef { base: String }` variant to `TypeSpec` enum in `rustconf/src/parser/ast.rs`
    - _Requirements: 4.3_

  - [x] 7.2 Add `identityref` parsing in `parse_type_spec` in `rustconf/src/parser/mod.rs`
    - Add match arm for `Token::IdentityRef` that creates `TypeSpec::IdentityRef { base: String::new() }`
    - In `parse_type_body`, handle `Token::Base` inside an `IdentityRef` type to extract and store the base identity name (including prefixed names)
    - Return a descriptive error if `base` is missing
    - _Requirements: 4.1, 4.2, 4.5_

  - [x] 7.3 Update `validate_typespec_constraints` and other validators to handle `IdentityRef`
    - Ensure the catch-all `_ => {}` arms cover the new variant (verify no exhaustive match breaks)
    - _Requirements: 4.3_

  - [ ]* 7.4 Write property test for identityref base name preservation
    - **Property 4: identityref base name preservation**
    - Generate valid identity names (plain and prefixed), construct `type identityref { base <name>; }`, parse, verify `TypeSpec::IdentityRef.base` contains the exact name
    - **Validates: Requirements 4.1, 4.2, 4.3**

  - [ ]* 7.5 Write unit test for identityref missing base error
    - Parse `type identityref {}` and verify a descriptive error is returned
    - _Requirements: 4.5_

- [x] 8. Update code generator for `IdentityRef` and wildcard enum `*`
  - [x] 8.1 Add `TypeSpec::IdentityRef` match arm in `generate_leaf_type` in `rustconf/src/generator/types.rs`
    - Map `IdentityRef` to `"String"` base type
    - Also update `needs_validation` and `get_validated_type_name` to handle the new variant (no validation needed)
    - _Requirements: 4.4_

  - [x] 8.2 Add wildcard enum `*` → `Star` variant mapping in code generator
    - In enum variant name generation (in `rustconf/src/generator/naming.rs` or `types.rs`), map `"*"` to `"Star"`
    - Emit `#[serde(rename = "*")]` attribute on the generated variant for round-trip serialization
    - _Requirements: 6.2, 6.3_

  - [x]* 8.3 Write unit tests for codegen changes
    - Test that `IdentityRef` generates `String` type
    - Test that `*` enum generates `Star` variant with `#[serde(rename = "*")]`
    - _Requirements: 4.4, 6.2, 6.3_

- [x] 9. Checkpoint
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Integration validation
  - [x] 10.1 Write integration test asserting ≥34 of 40 vendor models parse successfully
    - Add a test in `tests/integration/tests/` that runs the parser against all 40 vendor YANG models and asserts at least 34 succeed
    - Verify failed models produce descriptive error messages
    - Verify generated code compiles (handled by existing `build.rs`)
    - _Requirements: 7.1, 7.2, 7.3_

- [ ] 11. Final checkpoint
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests use the `proptest` crate (already a workspace dev-dependency) with ≥100 iterations per property
- Unit tests validate specific examples and edge cases
- The implementation language is Rust, matching the existing codebase
