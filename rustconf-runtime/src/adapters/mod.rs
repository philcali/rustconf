//! HTTP transport adapter implementations.
//!
//! This module provides concrete implementations of the `HttpTransport` trait
//! for popular HTTP client libraries. Each adapter is feature-gated to allow
//! users to choose their preferred transport without pulling in unnecessary
//! dependencies.

#[cfg(feature = "reqwest")]
pub mod reqwest_adapter;

#[cfg(feature = "hyper")]
pub mod hyper_adapter;
