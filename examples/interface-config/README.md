# Interface Configuration Example

This example demonstrates how to use rustconf to generate type-safe Rust bindings from YANG specifications.

## Overview

The example includes:
- A YANG specification file (`yang/interface-config.yang`) defining network interface configuration
- A build script (`build.rs`) that generates Rust code from the YANG specification
- An application (`src/main.rs`) that demonstrates using the generated types

## What This Example Demonstrates

### 1. Generated Type Usage
- Creating instances of generated structs (`Interface`, `InterfaceConfig`, `InterfaceState`)
- Working with validated types (`MtuType`, `InterfaceName`)
- Using generated enums (`InterfaceType`, `Speed`, `Duplex`, `AdminStatus`, `OperStatus`)
- Handling choice types (`AddressType` with IPv4/IPv6 variants)

### 2. Serialization and Deserialization
- Serializing Rust types to JSON using `serde_json`
- Deserializing JSON to Rust types with automatic validation
- RESTCONF-compliant JSON encoding (kebab-case field names)
- Handling optional fields with `skip_serializing_if`

### 3. Validation Error Handling
- Range constraint validation (MTU must be 68-9000)
- Pattern constraint validation (interface names must match `[a-zA-Z0-9_-]+`)
- Length constraint validation (interface names must be 1-255 characters)
- Validation during both construction and deserialization

### 4. Optional Fields
- Creating minimal instances with only required fields
- Working with `Option<T>` for optional YANG nodes
- Proper JSON serialization that omits optional fields

### 5. Enums and Choices
- Using generated enums for YANG enumeration types
- Working with choice types that represent mutually exclusive options
- Serialization of enum variants

## Running the Example

```bash
# From the repository root
cargo run -p interface-config

# Or from this directory
cargo run
```

## Expected Output

The example will print demonstrations of:
1. Creating generated types with various configurations
2. Serializing types to JSON
3. Deserializing JSON to types
4. Validation errors for invalid values
5. Working with optional fields
6. Using enums and choice types

All demonstrations include assertions to verify correct behavior.

## Requirements Validated

This example validates the following requirements from the rustconf specification:
- **Requirement 6.1**: Example module demonstrating basic usage patterns
- **Requirement 6.4**: Shows how to use generated types for RESTCONF operations

## YANG Specification

The `interface-config.yang` file demonstrates common YANG constructs:
- **Typedefs**: Custom types with constraints (`interface-name`, `mtu-type`)
- **Groupings**: Reusable data structures (`interface-statistics`)
- **Containers**: Structured data (`config`, `state`)
- **Lists**: Collections with keys (`interface`)
- **Leaves**: Individual data fields with various types
- **Choices**: Mutually exclusive options (`address-type`)
- **Enumerations**: Fixed sets of values (`type`, `speed`, `duplex`)
- **RPCs**: Remote procedure calls (`reset-interface`, `get-statistics`)
- **Notifications**: Event notifications (`interface-state-change`)

## Build Process

The build script (`build.rs`) uses rustconf's `RustconfBuilder` API:

```rust
rustconf::RustconfBuilder::new()
    .yang_file("yang/interface-config.yang")
    .search_path("yang/")
    .output_dir(out_dir)
    .enable_validation(true)
    .module_name("interface_config")
    .generate()
```

This generates Rust code at compile time, making it available to the application via `include!`.

## Notes

- The generated code enforces YANG constraints at runtime through validation
- All generated types implement `Debug`, `Clone`, `Serialize`, and `Deserialize`
- Validated types use newtype wrappers with constructor methods
- The example assumes the YANG parser successfully generates the bindings
