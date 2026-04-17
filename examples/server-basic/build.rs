//! Build script for generating server-side YANG bindings.
//!
//! This demonstrates how to enable server-side code generation
//! alongside the standard client code generation.

fn main() {
    rustconf::RustconfBuilder::new()
        .yang_file("yang/device-management.yang")
        .search_path("yang/")
        .output_dir("src/generated")
        .module_name("device_management")
        .enable_restful_rpcs(true)
        .enable_validation(true)
        .modular_output(true)
        // Enable server-side code generation
        .enable_server_generation(true)
        .generate()
        .expect("Failed to generate RESTCONF bindings");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=yang/device-management.yang");
}
