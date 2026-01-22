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

Add rustconf to your `Cargo.toml`:

```toml
[build-dependencies]
rustconf = "0.1"
```

Create a `build.rs` file:

```rust
fn main() {
    rustconf::RustconfBuilder::new()
        .yang_file("specs/example.yang")
        .search_path("specs/")
        .output_dir(std::env::var("OUT_DIR").unwrap())
        .enable_validation(true)
        .generate()
        .expect("Failed to generate RESTCONF bindings");
}
```

## Development Status

This project is currently under active development. See the implementation plan in `.kiro/specs/rustconf/tasks.md` for progress.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
