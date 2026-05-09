# Requirements Document: Junos Generator Compliance

## Introduction

The rustconf YANG parser compliance spec (`yang-parser-compliance`) brought successful code generation from 4 to 31 out of 40 Juniper cRPD and IETF YANG models. Nine models still fail, grouped into four categories of parser and code generator gaps. This spec addresses those four categories to bring all 40 models to successful generation.

The failures are:

1. **Prefixed type references not resolved in generator** (4 modules: `ietf-inet-types`, `ietf-interfaces`, `ietf-routing-types`, `ietf-yang-types`) — The parser correctly stores `prefix:name` type references, but the code generator's `to_type_name()` call on the full `prefix:name` string produces invalid Rust type names that fail `syn::parse_str`.
2. **`bits` type not supported in parser** (1 module: `ietf-netconf-acm`) — The parser's `parse_type_spec` has no match arm for `Token::Bits`, causing a `"Expected type name, found Bits"` error.
3. **`choice`/`case` nodes in RPC input/output produce undefined type references** (1 module: `ietf-netconf`) — The generator emits struct fields referencing types from `choice`/`case` constructs in RPC input/output, but never generates the corresponding type definitions. The `build.rs` validation catches this.
4. **Decimal range values not supported in parser** (3 modules: `junos-conf-logical-systems`, `junos-conf-protocols`, `junos-conf-routing-instances`) — The parser's `parse_range_value` only handles integers and the `min`/`max` keywords, failing on decimal values like `0.001` used in `decimal64` range constraints.

## Glossary

- **Parser**: The parsing component (`rustconf/src/parser/mod.rs`) that consumes tokens and produces an AST (`YangModule`).
- **Code_Generator**: The Rust code generation component (`rustconf/src/generator/`) that transforms AST nodes into Rust source code.
- **Type_Generator**: The type generation sub-component (`rustconf/src/generator/types.rs`) responsible for mapping YANG types to Rust types.
- **Operations_Generator**: The operations generation sub-component (`rustconf/src/generator/operations.rs`) responsible for generating RPC input/output types and CRUD operations.
- **Prefixed_Type_Reference**: A `TypeSpec::TypedefRef` whose `name` field contains a colon-separated `prefix:name` string referencing a type from an imported module.
- **Bits_Type**: A YANG built-in type (`bits`) that represents a set of named bit positions, each of which can be set or cleared independently (RFC 7950 §9.7).
- **Choice_Node**: A YANG `choice` statement defining mutually exclusive branches (`case` nodes), each containing data nodes.
- **Decimal_Range**: A `range` constraint value containing a decimal point (e.g., `0.001`), used with `decimal64` types per RFC 7950 §9.3.
- **Integration_Test_Harness**: The existing test infrastructure in `tests/integration/` that discovers vendor YANG models and invokes `RustconfBuilder` to generate and compile code.
- **Build_Validation**: The `check_generated_module` function in `tests/integration/build.rs` that verifies generated code references only defined types.

## Requirements

### Requirement 1: Resolve Prefixed Type References in Code Generator

**User Story:** As a developer using rustconf, I want the code generator to handle `prefix:name` type references, so that IETF standard typedef modules using cross-module type references generate valid Rust code.

#### Acceptance Criteria

1. WHEN the Type_Generator encounters a `TypeSpec::TypedefRef` whose `name` field contains a colon (a Prefixed_Type_Reference), THE Type_Generator SHALL extract the local name portion after the colon and use it for Rust type name generation, discarding the prefix.
2. WHEN the Type_Generator generates a Rust type name from a Prefixed_Type_Reference, THE Type_Generator SHALL apply the same `to_type_name` conversion to the local name as it does for non-prefixed typedef references.
3. WHEN the Type_Generator encounters a Prefixed_Type_Reference in a typedef definition, THE Type_Generator SHALL produce a valid Rust type alias that compiles without errors.
4. WHEN the Type_Generator encounters a Prefixed_Type_Reference in a leaf or leaf-list field, THE Type_Generator SHALL produce a struct field whose type string passes `syn::parse_str` without error.
5. FOR ALL Prefixed_Type_Reference values of the form `prefix:name`, generating the Rust type name from the full string and from only the local name portion SHALL produce the same result (idempotence after prefix stripping).

### Requirement 2: Support `bits` Built-in Type in Parser and Code Generator

**User Story:** As a developer using rustconf, I want the parser and code generator to support the `bits` built-in type, so that YANG models using `type bits` (such as `ietf-netconf-acm`) parse and generate valid Rust code.

#### Acceptance Criteria

1. WHEN the Parser encounters `type bits` in a YANG module, THE Parser SHALL recognise the `Bits` token and produce a valid `TypeSpec` AST node representing the bits type.
2. WHEN the Parser parses a `bits` type body containing `bit` statements with name and optional `position` values, THE Parser SHALL store each bit definition in the AST.
3. WHEN the Code_Generator encounters a `bits` type AST node, THE Code_Generator SHALL generate a Rust type that can represent a set of named bit flags (e.g., a `String` type for initial support, or a dedicated bitflags struct).
4. WHEN the Code_Generator generates serialization code for a `bits` type, THE Code_Generator SHALL produce code that serializes bit values as a space-separated string of set bit names per RFC 7950 §9.7.4.
5. IF the Parser encounters `type bits` without any `bit` statements in the type body, THEN THE Parser SHALL accept the empty bits type without error (bits may be defined in a base typedef).

### Requirement 3: Generate Type Definitions for `choice`/`case` Nodes in RPC Input/Output

**User Story:** As a developer using rustconf, I want the code generator to produce complete type definitions for `choice`/`case` constructs in RPC input and output, so that generated code does not reference undefined types.

#### Acceptance Criteria

1. WHEN the Operations_Generator generates input or output types for an RPC that contains Choice_Node data nodes, THE Operations_Generator SHALL generate the corresponding enum type definition for each choice.
2. WHEN the Operations_Generator generates input or output types for an RPC that contains `case` nodes with complex data (containers, lists, or multiple leaves), THE Operations_Generator SHALL generate the corresponding struct type definitions for each case.
3. WHEN the Operations_Generator generates a struct field referencing a choice type, THE Operations_Generator SHALL ensure the referenced type is defined in the same generated output file or in the types module.
4. WHEN the Build_Validation scans generated code for a module containing RPC choice nodes, THE Build_Validation SHALL find all referenced types defined and SHALL NOT skip the module.
5. FOR ALL RPC definitions containing Choice_Node data nodes, the set of type names referenced in generated struct fields SHALL be a subset of the type names defined in the generated code (no undefined references).

### Requirement 4: Support Decimal Range Values in Parser

**User Story:** As a developer using rustconf, I want the parser to accept decimal values in `range` constraints, so that YANG models using `decimal64` types with range constraints like `"0.001..max"` parse successfully.

#### Acceptance Criteria

1. WHEN the Parser parses a range constraint string containing a decimal value (e.g., `0.001`), THE Parser SHALL accept the decimal value without returning an `"Invalid range value"` error.
2. WHEN the Parser parses a decimal range value, THE Parser SHALL convert the decimal to an `i64` representation compatible with the existing `RangeConstraint` structure (e.g., by scaling to the `decimal64` fraction-digits precision or by truncating to the nearest integer).
3. WHEN the Parser parses a range constraint string mixing decimal values with `min` and `max` keywords (e.g., `"0.001..max"`), THE Parser SHALL handle both decimal values and keywords correctly.
4. WHEN the Parser parses a range constraint string with multiple decimal range parts separated by `|` (e.g., `"0.001..1.0 | 2.0..max"`), THE Parser SHALL parse each part independently and produce a valid `RangeConstraint` with multiple `Range` entries.
5. FOR ALL range constraint strings containing decimal values, parsing the constraint SHALL produce a `RangeConstraint` whose `Range` entries have `min <= max` (well-formedness invariant).

### Requirement 5: Integration Validation — All 40 Models Generate Successfully

**User Story:** As a developer using rustconf, I want to validate that all parser and generator fixes collectively enable all 40 vendor YANG models to generate successfully, so that I have confidence the compliance gaps are fully resolved.

#### Acceptance Criteria

1. WHEN the Integration_Test_Harness runs against the full set of 40 vendor YANG models after all fixes are applied, THE Integration_Test_Harness SHALL successfully parse and generate code for all 40 of the 40 models (up from the current 31).
2. WHEN the Integration_Test_Harness generates code for a previously-failing model, THE Code_Generator SHALL produce Rust code that compiles without errors.
3. WHEN the Build_Validation runs against all 40 generated modules, THE Build_Validation SHALL report zero validation failures (no undefined type references).
4. THE existing integration test threshold (`MIN_PASSING_MODELS`) SHALL be updated from 34 to 40 to reflect the new expected baseline.

## Out of Scope (Future Work)

The following items are acknowledged but excluded from this spec:

- `augment` / `deviation` statement parsing and code generation
- `when` / `must` constraint evaluation
- `leafref` path resolution and validation
- Pattern constraint regex validation (currently a placeholder returning `true`)
- RFC 7951 JSON namespace wrapping for RPC payloads
- RESTCONF media type headers (`application/yang-data+json`)
- RESTCONF error response parsing (RFC 8040 §7.1)
- RESTCONF query parameter support (`depth`, `fields`, `content`, `filter`)
- Enumeration type generation as proper Rust enums (currently mapped to `String`)
- `leaf-list` code generation (currently skipped)
- List key encoding in RESTCONF URLs
