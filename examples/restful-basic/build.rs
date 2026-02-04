//! Build script for generating YANG bindings with RESTful RPC support.

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    // Generate bindings with RESTful RPC support enabled
    let result = rustconf::RustconfBuilder::new()
        .yang_file("yang/device-management.yang")
        .search_path("yang/")
        .output_dir(&out_dir)
        .module_name("device_management")
        // Enable RESTful RPC generation
        .enable_restful_rpcs(true)
        .generate();

    if let Err(e) = result {
        eprintln!("Failed to generate bindings: {:?}", e);
        std::process::exit(1);
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=yang/device-management.yang");
}
