//! Integration test harness for validating rustconf-generated code against live RESTCONF emulators.
//!
//! This crate provides:
//! - Emulator lifecycle management via `testcontainers`
//! - Configuration loading from environment variables and TOML files
//! - Test fixture management for applying and restoring device state
//! - Conformance reporting for structured test result output
//!
//! # Usage
//!
//! Integration tests are gated behind `RUSTCONF_INTEGRATION_TEST=1`. When this
//! environment variable is not set, emulator-dependent tests are skipped gracefully.

pub mod config;
pub mod emulators;
pub mod error;
pub mod fixture;
pub mod generated;
pub mod harness;
pub mod multi_emulator;
pub mod reporter;

pub use config::HarnessConfig;
pub use emulators::{EmulatorConfig, JunosCrpdConfig, NetopeerConfig};
pub use error::HarnessError;
pub use fixture::{FixtureDefinition, FixtureManager};
pub use harness::TestHarness;
pub use multi_emulator::{create_harness, CombinedConformanceReport, MultiEmulatorRunner};
pub use reporter::{ConformanceReporter, TestDetails, TestResult, TestStatus};
