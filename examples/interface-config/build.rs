//! Build script for generating YANG bindings.
//!
//! This example demonstrates how to use rustconf's RustconfBuilder API
//! to generate type-safe Rust bindings from YANG specifications during
//! the build process.

fn main() {
    // Get the output directory from cargo
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set by cargo");

    // Configure and run rustconf code generation
    let result = rustconf::RustconfBuilder::new()
        // Specify the YANG file to process
        .yang_file("yang/interface-config.yang")
        // Add search path for resolving YANG imports
        .search_path("yang/")
        // Set output directory for generated code
        .output_dir(out_dir)
        // Enable validation in generated types
        .enable_validation(true)
        // Optionally customize the generated module name
        .module_name("interface_config")
        // Generate the Rust bindings
        .enable_restful_rpcs(true)
        .generate();

    // Handle generation errors with proper error reporting
    if let Err(e) = result {
        // Error details are already reported to cargo by the builder
        // Exit with error code to fail the build
        eprintln!("Failed to generate RESTCONF bindings: {:?}", e);
        std::process::exit(1);
    }

    // Emit cargo directives for build dependencies
    // The builder automatically emits cargo:rerun-if-changed for YANG files,
    // but we can add additional directives here if needed
    println!("cargo:rerun-if-changed=build.rs");

    // Success message (optional, for debugging)
    println!("cargo:warning=Successfully generated RESTCONF bindings from interface-config.yang");
}
