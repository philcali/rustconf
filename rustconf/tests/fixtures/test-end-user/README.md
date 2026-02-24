# Test End-User Project

This is a test end-user project that demonstrates using the test-intermediate-crate.

## Purpose

This project validates that end users can:
- Depend only on the intermediate client crate
- Use generated types and operations without rustconf
- Build without requiring build.rs
- Use the RESTCONF client API

## Key Points

- **No rustconf dependency**: This project does not depend on rustconf
- **No build.rs**: No build-time code generation required
- **Simple usage**: Just add the intermediate crate as a dependency

## Structure

- `src/main.rs` - Example application using the intermediate crate API
- `Cargo.toml` - Only depends on test-intermediate-crate

## Running

This project is built and tested as part of the integration test suite.
