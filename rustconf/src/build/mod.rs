//! Build system integration module.

pub mod builder;
pub mod error;

pub use builder::RustconfBuilder;
pub use error::BuildError;
