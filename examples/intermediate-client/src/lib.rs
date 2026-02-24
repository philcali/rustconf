//! Device Client
//!
//! Type-safe Rust bindings for the Device Management RESTCONF API.
//!
//! This is an example intermediate client crate that demonstrates how to:
//! - Generate RESTCONF bindings at build time
//! - Commit generated code to version control
//! - Publish as a reusable library
//! - Eliminate build-time dependencies for end users
//!
//! ## Usage
//!
//! ```no_run
//! use device_client::{RestconfClient, reqwest_adapter::ReqwestTransport};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let transport = ReqwestTransport::new();
//!     let client = RestconfClient::new("https://device.example.com", transport)?;
//!     
//!     // Use generated operations
//!     let info = device_client::get_system_info(&client).await?;
//!     println!("Hostname: {}", info.hostname.unwrap_or_default());
//!     
//!     Ok(())
//! }
//! ```

// Re-export generated code
pub mod generated;

// Re-export for convenience
pub use generated::*;
