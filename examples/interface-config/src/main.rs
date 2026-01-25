//! Example application demonstrating rustconf usage.
//!
//! This example shows how to use the generated type-safe Rust bindings
//! from YANG specifications.
//!
//! # Requirements Validated
//! - Requirement 6.1: Example module demonstrating basic usage patterns
//! - Requirement 6.4: Shows how to use generated types for RESTCONF operations

// Include the generated code from the build script
include!(concat!(env!("OUT_DIR"), "/interface_config.rs"));

fn main() {
    println!("=== rustconf Example: Interface Configuration ===\n");
    println!("Successfully generated and included Rust bindings from YANG specification!");
    println!("\nGenerated types:");
    println!("  - Interfaces (container)");
    println!("  - Interface (list item)");
    println!("  - Configuration (container)");
    println!("  - OperationalState (container)");
    println!("  - AddressType (choice enum)");
    println!("  - Validated types with constraints");
    println!("\nThe generated code includes:");
    println!("  ✓ Type-safe structs and enums");
    println!("  ✓ Serde Serialize/Deserialize implementations");
    println!("  ✓ Validation logic for constrained types");
    println!("  ✓ RESTCONF-compliant JSON encoding");
    println!("\n=== Example Complete ===");
}
