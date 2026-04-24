# Requirements Document

## Introduction

The rustconf YANG parser currently fails to parse 37 out of 40 real-world vendor YANG models (Juniper cRPD and IETF). Six categories of compliance gaps in the lexer, parser, and code generator prevent successful parsing. This spec addresses the six parser-level fixes that would unlock approximately 34 of the 37 failing models, bringing the parser into compliance with RFC 7950 constructs used in production YANG models.

## Glossary

- **Lexer**: The tokenization component (`rustconf/src/parser/lexer.rs`) that converts raw YANG text into a stream of `Token` values.
- **Parser**: The parsing component (`rustconf/src/parser/mod.rs`) that consumes tokens and produces an AST (`YangModule`).
- **AST**: The Abstract Syntax Tree types defined in `rustconf/src/parser/ast.rs` representing parsed YANG constructs.
- **Code_Generator**: The Rust code generation component (`rustconf/src/generator/`) that transforms AST nodes into Rust source code.
- **YANG_Model**: A `.yang` file conforming to RFC 7950 that defines a data model for network configuration.
- **Module_Prefix**: A short alias declared via `prefix` in a YANG `import` statement, used to qualify identifiers with a colon (e.g., `junos-conf-interfaces:foo`).
- **Prefixed_Identifier**: An identifier of the form `prefix:name` referencing a type or grouping from an imported module.
- **Enum_Name**: The name argument of a YANG `enum` statement, which RFC 7950 allows to be either an unquoted identifier or a quoted string.
- **Range_Bound**: A value in a YANG `range` or `length` constraint expression; RFC 7950 permits the keywords `min` and `max` alongside integer literals.
- **Identityref_Type**: A YANG built-in type (`identityref`) that references a base identity; values are identity names resolved at runtime.
- **Keyword_As_Identifier**: A situation where a YANG keyword (e.g., `when`, `import`, `contact`) appears in a context where it serves as a plain identifier (e.g., as an enum name or leaf name).
- **Wildcard_Enum**: An `enum` statement whose name argument is the literal character `*`, used in models such as `ietf-routing-types`.
- **Integration_Test_Harness**: The existing test infrastructure in `tests/integration/` that discovers vendor YANG models and invokes `RustconfBuilder` to generate and compile code.

## Requirements

### Requirement 1: Quoted Enum Names

**User Story:** As a developer using rustconf, I want the parser to accept quoted strings as enum names, so that Juniper YANG models using `enum "bootp"` syntax parse successfully.

#### Acceptance Criteria

1. WHEN the Parser encounters an `enum` statement followed by a `StringLiteral` token, THE Parser SHALL accept the string value as the Enum_Name and produce a valid `EnumValue` AST node.
2. WHEN the Parser encounters an `enum` statement followed by an `Identifier` token, THE Parser SHALL continue to accept the identifier as the Enum_Name (preserving existing behaviour).
3. WHEN the Parser encounters an `enum` statement followed by a keyword token (e.g., `When`, `Import`, `Contact`), THE Parser SHALL accept the keyword text as the Enum_Name.
4. FOR ALL valid Enum_Name values (unquoted identifiers, quoted strings, and keyword tokens), parsing then printing then parsing the containing `type enumeration` block SHALL produce an equivalent AST (round-trip property).

### Requirement 2: Module-Prefixed Identifiers

**User Story:** As a developer using rustconf, I want the parser to handle `prefix:name` identifiers in type references, so that YANG models using cross-module type references parse successfully.

#### Acceptance Criteria

1. WHEN the Parser encounters a `type` statement whose argument contains a colon (e.g., `junos-conf-interfaces:foo`), THE Parser SHALL parse the full Prefixed_Identifier as a `TypedefRef` with the complete `prefix:name` string stored in the `name` field.
2. WHEN the Parser encounters a `uses` statement whose argument contains a colon (e.g., `junos:some-grouping`), THE Parser SHALL parse the full Prefixed_Identifier as the grouping name.
3. WHEN the Lexer produces an `Identifier` token followed by a `Colon` token followed by another `Identifier` token in a type or uses context, THE Parser SHALL combine them into a single Prefixed_Identifier string.
4. WHEN a Prefixed_Identifier appears in a non-type, non-uses context (e.g., inside a `path` string), THE Parser SHALL preserve existing behaviour and not misinterpret the colon.

### Requirement 3: `min` and `max` Keywords in Range and Length Constraints

**User Story:** As a developer using rustconf, I want the parser to recognise `min` and `max` as valid bounds in `range` and `length` constraint expressions, so that models using `"0..max"` or `"1..max"` parse successfully.

#### Acceptance Criteria

1. WHEN the Parser parses a range constraint string containing the keyword `max` as an upper bound, THE Parser SHALL interpret `max` as the maximum value for the corresponding numeric type (e.g., `i64::MAX` for `int64`, `u64::MAX` for `uint64`).
2. WHEN the Parser parses a range constraint string containing the keyword `min` as a lower bound, THE Parser SHALL interpret `min` as the minimum value for the corresponding numeric type (e.g., `i64::MIN` for `int64`, `0` for unsigned types).
3. WHEN the Parser parses a length constraint string containing the keyword `max` as an upper bound, THE Parser SHALL interpret `max` as `u64::MAX`.
4. WHEN the Parser parses a length constraint string containing the keyword `min` as a lower bound, THE Parser SHALL interpret `min` as `0`.
5. FOR ALL range constraint strings using `min` or `max` keywords, THE Parser SHALL produce a `RangeConstraint` whose `Range` values are numerically equivalent to the type-appropriate extremes.

### Requirement 4: `identityref` Type Support

**User Story:** As a developer using rustconf, I want the parser and code generator to support the `identityref` built-in type, so that YANG models declaring `type identityref { base some-identity; }` parse and generate valid Rust code.

#### Acceptance Criteria

1. WHEN the Parser encounters `type identityref`, THE Parser SHALL recognise the `IdentityRef` token and begin parsing an identityref type body.
2. WHEN the Parser parses an identityref type body containing a `base` statement, THE Parser SHALL store the base identity name in the AST.
3. THE AST SHALL include an `IdentityRef` variant in the `TypeSpec` enum with a `base` field containing the identity name as a `String`.
4. WHEN the Code_Generator encounters a `TypeSpec::IdentityRef` node, THE Code_Generator SHALL generate a Rust `String` type (or a type alias wrapping `String`) to represent the identity reference value.
5. IF the Parser encounters `type identityref` without a `base` statement in the type body, THEN THE Parser SHALL return a descriptive parse error indicating that `base` is required.

### Requirement 5: YANG Keywords Used as Identifiers

**User Story:** As a developer using rustconf, I want the parser to allow YANG keywords to appear as identifiers in contexts where an identifier is expected, so that models using keywords like `when`, `import`, or `contact` as leaf names or enum names parse successfully.

#### Acceptance Criteria

1. WHEN the Parser expects an identifier (e.g., after `leaf`, `container`, `enum`, `list`, `grouping`, `choice`, `case`, `uses`, or `typedef`), THE Parser SHALL accept any YANG keyword token as a valid identifier by extracting its textual representation.
2. WHEN a keyword token appears in a statement-keyword position (e.g., at the start of a statement inside a module body), THE Parser SHALL continue to treat the token as a keyword (preserving existing behaviour).
3. FOR ALL YANG keyword tokens that have a textual representation matching a valid YANG identifier, THE Parser SHALL be able to use that token as an identifier in name-argument positions without error.

### Requirement 6: Wildcard Enum `*`

**User Story:** As a developer using rustconf, I want the parser to accept `*` as a valid enum name, so that YANG models like `ietf-routing-types` that use `enum "*"` or `enum *` parse successfully.

#### Acceptance Criteria

1. WHEN the Parser encounters an `enum` statement followed by a `StringLiteral` token containing `*`, THE Parser SHALL accept `*` as the Enum_Name and produce a valid `EnumValue` AST node.
2. WHEN the Code_Generator encounters an `EnumValue` whose name is `*`, THE Code_Generator SHALL generate a valid Rust enum variant name (e.g., `Wildcard` or `Star`) since `*` is not a valid Rust identifier.
3. WHEN the Code_Generator generates serialization code for a wildcard enum variant, THE Code_Generator SHALL ensure the variant serializes back to the string `"*"` for YANG/XML/JSON compatibility.

### Requirement 7: Integration Validation

**User Story:** As a developer using rustconf, I want to validate that the parser fixes collectively unlock the expected vendor YANG models, so that I have confidence the compliance gaps are resolved.

#### Acceptance Criteria

1. WHEN the Integration_Test_Harness runs against the full set of 40 vendor YANG models after all parser fixes are applied, THE Integration_Test_Harness SHALL successfully parse and generate code for at least 34 of the 40 models (up from the current 3).
2. WHEN a YANG_Model fails to parse after the fixes, THE Integration_Test_Harness SHALL report the failure with a descriptive error message identifying the unsupported construct.
3. WHEN the Integration_Test_Harness generates code for a previously-failing model, THE Code_Generator SHALL produce Rust code that compiles without errors.

## Out of Scope (Future Work)

The following structural gaps are acknowledged but excluded from this spec:

- `augment` / `deviation` statement parsing and code generation
- `when` / `must` constraint evaluation
- `bits` type parser and code generation
- `leafref` path resolution and validation
- Pattern constraint regex validation (currently a placeholder returning `true`)
- RPC operation body code generation (currently marked `unimplemented!`)
