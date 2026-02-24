# Test Intermediate Crate

This is a test intermediate crate used for integration testing of the intermediate crate pattern.

## Purpose

This crate demonstrates how to create an intermediate client crate that:
- Uses rustconf as a build-dependency
- Generates code to `src/generated/` during build
- Depends on rustconf-runtime for runtime components
- Can be used by end-user projects without requiring rustconf

## Structure

- `yang/` - YANG specification files
- `build.rs` - Build script that runs rustconf code generation
- `src/lib.rs` - Library entry point that re-exports generated code
- `src/generated/` - Generated code (created during build)

## Usage

This crate is built automatically as part of the integration test suite.
