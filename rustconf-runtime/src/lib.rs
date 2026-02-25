//! Runtime components for rustconf-generated RESTCONF clients.
//!
//! This crate provides the core runtime types and traits needed by code generated
//! from rustconf. It includes:
//!
//! - HTTP transport abstraction (`HttpTransport` trait)
//! - RESTCONF client implementation (`RestconfClient`)
//! - Error types (`RpcError`)
//! - Optional transport adapters for reqwest and hyper (feature-gated)
//!
//! # Features
//!
//! - `reqwest`: Enable the reqwest-based HTTP transport adapter
//! - `hyper`: Enable the hyper-based HTTP transport adapter
//!
//! # Example
//!
//! ```rust,ignore
//! use rustconf_runtime::{RestconfClient, HttpTransport, RpcError};
//!
//! #[cfg(feature = "reqwest")]
//! use rustconf_runtime::reqwest_adapter::ReqwestTransport;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), RpcError> {
//!     let transport = ReqwestTransport::new();
//!     let client = RestconfClient::new("https://device.example.com", transport)?;
//!     
//!     // Use the client with generated operations...
//!     Ok(())
//! }
//! ```

pub mod adapters;
pub mod error;
pub mod transport;

// Re-export commonly used types
pub use error::{DefaultErrorMapper, ErrorMapper, RpcError, ServerError};
pub use transport::{
    HttpMethod, HttpRequest, HttpResponse, HttpTransport, RequestInterceptor, RestconfClient,
    ServerRequest, ServerResponse, ServerTransport,
};

// Re-export adapter modules when features are enabled
#[cfg(feature = "reqwest")]
pub use adapters::reqwest_adapter;

#[cfg(feature = "hyper")]
pub use adapters::hyper_adapter;
