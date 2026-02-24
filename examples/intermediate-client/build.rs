fn main() {
    rustconf::RustconfBuilder::new()
        .yang_file("yang/device-management.yang")
        .search_path("yang/")
        .output_dir("src/generated")
        .enable_validation(true)
        .enable_restful_rpcs(true)
        .modular_output(true)
        .module_name("device_management")
        .generate()
        .expect("Failed to generate RESTCONF bindings");
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=yang/");
}
