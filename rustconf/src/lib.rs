//! # rustconf
//!
//! A Rust build library that generates type-safe bindings from RESTCONF/YANG specifications
//! at compile time.
//!
//! ## Overview
//!
//! rustconf consists of three main components:
//! - **YANG Parser**: Parses YANG 1.0/1.1 specification files into an AST
//! - **Code Generator**: Transforms the YANG AST into idiomatic Rust code
//! - **Build Integration**: Provides build.rs integration for seamless cargo workflow
//!
//! ## Example
//!
//! ```rust,no_run
//! rustconf::RustconfBuilder::new()
//!     .yang_file("specs/example.yang")
//!     .search_path("specs/")
//!     .output_dir(std::env::var("OUT_DIR").unwrap())
//!     .enable_validation(true)
//!     .generate()
//!     .expect("Failed to generate RESTCONF bindings");
//! ```

pub mod build;
pub mod generator;
pub mod parser;

// Re-export main API types
pub use build::{BuildError, RustconfBuilder};
pub use generator::{CodeGenerator, GeneratorConfig, GeneratorError};
pub use parser::{ParseError, YangParser};
