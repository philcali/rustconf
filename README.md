# rustconf

A Rust build library that generates type-safe bindings from RESTCONF/YANG specifications at compile time.

## Overview

rustconf enables Rust developers to work with RESTCONF APIs in a type-safe, idiomatic manner by generating Rust code from YANG specifications during the build process.

### Features

- **Type-Safe Bindings**: YANG schema constraints are enforced at compile time through Rust's type system
- **Build Integration**: Seamless integration with cargo build process via build.rs
- **YANG 1.0/1.1 Support**: Full support for both YANG versions
- **Serialization**: Automatic serde implementations for JSON and XML
- **Validation**: Runtime validation of constraints defined in YANG schemas
- **Idiomatic Rust**: Generated code follows Rust conventions and best practices

## Components

1. **YANG Parser**: Parses YANG specification files into an abstract syntax tree
2. **Code Generator**: Transforms YANG AST into idiomatic Rust code
3. **Build Integration**: Provides build.rs API for seamless cargo workflow

## Usage

rustconf supports two usage patterns:

### Pattern 1: Direct Build-Time Generation (Simple Projects)

For simple projects that directly consume YANG specifications:

Add rustconf to your `Cargo.toml`:

```toml
[build-dependencies]
rustconf = "0.1"

[dependencies]
rustconf-runtime = { version = "0.1", features = ["reqwest"] }
```

Create a `build.rs` file:

```rust
fn main() {
    rustconf::RustconfBuilder::new()
        .yang_file("specs/example.yang")
        .search_path("specs/")
        .output_dir(std::env::var("OUT_DIR").unwrap())
        .enable_validation(true)
        .enable_restful_rpcs(true)
        .generate()
        .expect("Failed to generate RESTCONF bindings");
}
```

### Pattern 2: Intermediate Client Crate (Recommended for Libraries)

For creating reusable client libraries that can be published and shared:

#### Benefits

- **No build-time generation for end users**: Generated code is committed and published as source
- **Simplified dependencies**: End users only need your client crate, not rustconf
- **Faster compilation**: No YANG parsing or code generation during builds
- **Version control**: Generated code changes are visible in diffs
- **Better IDE support**: Generated code is indexed like normal source files

#### Creating an Intermediate Client Crate

1. Create a new library crate:

```bash
cargo new --lib my-device-client
cd my-device-client
```

2. Set up `Cargo.toml`:

```toml
[package]
name = "my-device-client"
version = "0.1.0"
edition = "2021"

[features]
default = ["rustconf-runtime/reqwest"]

[dependencies]
rustconf-runtime = { version = "0.1" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
rustconf = "0.1"
```

3. Create `build.rs` (template for intermediate crates):

```rust
fn main() {
    rustconf::RustconfBuilder::new()
        .yang_file("yang/device-management.yang")
        .search_path("yang/")
        .output_dir("src/generated")  // Generate to src/ instead of OUT_DIR
        .enable_validation(true)
        .enable_restful_rpcs(true)
        .modular_output(true)  // Generate multiple files for better organization
        .module_name("device_management")
        .generate()
        .expect("Failed to generate RESTCONF bindings");
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=yang/");
}
```

4. Create `src/lib.rs`:

```rust
//! My Device Client
//!
//! Type-safe Rust bindings for the Device Management RESTCONF API.

// Re-export generated code
pub mod generated;

// Re-export for convenience
pub use generated::*;
```

5. Add your YANG files to `yang/` directory

6. Generate code and commit it:

```bash
cargo build  # Generates code to src/generated/
git add src/generated/
git commit -m "Add generated RESTCONF bindings"
```

7. Publish your crate:

```bash
cargo publish
```

#### Using an Intermediate Client Crate

End users simply add your crate as a dependency:

```toml
[dependencies]
my-device-client = "0.1"
```

No build.rs, no rustconf dependency, no YANG files needed:

```rust
use my_device_client::{RestconfClient, reqwest_adapter::ReqwestTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = ReqwestTransport::new();
    let client = RestconfClient::new("https://device.example.com", transport)?;
    
    // Use generated operations...
    Ok(())
}
```

See `examples/intermediate-client/` for a complete working example.

## Development Status

This project is currently under active development. See the implementation plan in `.kiro/specs/rustconf/tasks.md` for progress.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
