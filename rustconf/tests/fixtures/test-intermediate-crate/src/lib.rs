//! Test Intermediate Crate
//!
//! Type-safe Rust bindings for the Test Device RESTCONF API.
//! This is a test crate for validating the intermediate crate pattern.

// Re-export generated code
pub mod generated;

// Re-export for convenience
pub use generated::*;

// Re-export rustconf-runtime items
pub use rustconf_runtime::{RestconfClient, HttpTransport, RpcError};

// Re-export transport adapters based on rustconf-runtime features
#[cfg(feature = "rustconf-runtime/reqwest")]
pub use rustconf_runtime::reqwest_adapter;

#[cfg(feature = "rustconf-runtime/hyper")]
pub use rustconf_runtime::hyper_adapter;
