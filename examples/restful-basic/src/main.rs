//! Basic RESTful RPC Usage Example
//!
//! This example demonstrates how to use the generated RESTful RPC client.
//!
//! The example shows:
//! - The generated types and functions from YANG
//! - How RestconfClient would be used (if we had a real server)
//! - The structure of input/output types
//!
//! Note: This example demonstrates the API but won't actually connect
//! since there's no real RESTCONF server.

// Include the generated code
include!(concat!(env!("OUT_DIR"), "/device_management.rs"));

fn main() {
    println!("=== RESTful RPC Basic Usage Example ===\n");

    println!("This example demonstrates the generated RESTful RPC client API.\n");

    println!("Generated Types:");
    println!("----------------");
    println!("✓ HttpMethod enum (GET, POST, PUT, DELETE, PATCH)");
    println!("✓ HttpRequest struct (method, url, headers, body)");
    println!("✓ HttpResponse struct (status_code, headers, body)");
    println!("✓ HttpTransport trait (for pluggable HTTP clients)");
    println!("✓ RequestInterceptor trait (for auth, logging, etc.)");
    println!("✓ RestconfClient<T> struct (manages base URL and transport)");
    println!("✓ RpcError enum (all error types)");

    println!("\nGenerated RPC Functions:");
    println!("------------------------");
    println!("✓ operations::get_system_info(&client) -> Result<GetSystemInfoOutput, RpcError>");
    println!(
        "✓ operations::restart_device(&client, input) -> Result<RestartDeviceOutput, RpcError>"
    );
    println!("✓ operations::configure_interface(&client, input) -> Result<ConfigureInterfaceOutput, RpcError>");

    println!("\nGenerated Input/Output Types:");
    println!("-----------------------------");
    println!("✓ operations::RestartDeviceInput {{ delay_seconds: Option<u32> }}");
    println!(
        "✓ operations::RestartDeviceOutput {{ success: Option<bool>, message: Option<String> }}"
    );
    println!("✓ operations::GetSystemInfoOutput {{ hostname: Option<String>, uptime_seconds: Option<u64>, version: Option<String> }}");
    println!("✓ operations::ConfigureInterfaceInput {{ interface_name: String, ip_address: Option<String>, enabled: Option<bool> }}");
    println!("✓ operations::ConfigureInterfaceOutput {{ success: Option<bool>, message: Option<String> }}");

    println!("\nHow to Use (with a real server):");
    println!("---------------------------------");
    println!("1. Create a transport adapter:");
    println!("   let transport = reqwest_adapter::ReqwestTransport::new();");
    println!();
    println!("2. Create a RestconfClient:");
    println!("   let client = RestconfClient::new(\"https://device.example.com\", transport)?;");
    println!();
    println!("3. Call RPC operations:");
    println!("   let output = operations::get_system_info(&client).await?;");
    println!("   println!(\"Hostname: {{}}\", output.hostname.unwrap_or_default());");
    println!();
    println!("4. With input parameters:");
    println!("   let input = operations::RestartDeviceInput {{ delay_seconds: Some(10) }};");
    println!("   let output = operations::restart_device(&client, input).await?;");

    println!("\nTransport Adapters:");
    println!("-------------------");
    println!("The generated code includes transport adapters (when features are enabled):");
    println!("• reqwest_adapter::ReqwestTransport (feature: reqwest-client)");
    println!("• hyper_adapter::HyperTransport (feature: hyper-client)");
    println!("• Or implement your own HttpTransport trait");

    println!("\nInterceptors:");
    println!("-------------");
    println!("Add authentication, logging, or custom behavior:");
    println!("  let client = RestconfClient::new(url, transport)?");
    println!("      .with_interceptor(MyAuthInterceptor::new(token));");

    println!("\n=== Example Complete ===");
    println!("\nTo see working examples with actual HTTP calls:");
    println!("• Run: cargo run --example restful-interceptor");
    println!("• Run: cargo run --example restful-custom-transport");
    println!("• Run: cargo run --example restful-error-handling");
}
