//! Server-Side Generation Basic Example
//!
//! This example demonstrates how to use rustconf's server-side code generation
//! to create RESTCONF server handlers from YANG schemas.
//!
//! It shows two usage patterns:
//! 1. Using the generated stub handler as-is for testing
//! 2. Creating a custom handler that overrides specific methods

mod generated;

use generated::operations::operations::*;
use generated::server::handlers::DeviceManagementHandler;
use generated::server::router::RestconfRouter;
use generated::server::stubs::StubDeviceManagementHandler;
use rustconf_runtime::{HttpMethod, ServerError, ServerRequest};

// ---------------------------------------------------------------------------
// Part 1: Using the stub handler for testing
// ---------------------------------------------------------------------------

/// Demonstrates using the generated stub handler as-is.
///
/// The stub handler returns sensible defaults and logs every call,
/// making it useful for testing client code without real hardware.
async fn demo_stub_handler() {
    println!("=== Part 1: Stub Handler for Testing ===\n");

    // Create a stub handler — no setup needed
    let stub = StubDeviceManagementHandler::new();

    // Call some RPC operations directly on the handler
    let input = RestartDeviceInput {
        delay_seconds: Some(5),
    };
    let result = stub.restart_device(input).await;
    println!("restart_device result: {:?}", result);

    let result = stub.get_system_info().await;
    println!("get_system_info result: {:?}", result);

    let input = ConfigureInterfaceInput {
        interface_name: "eth0".to_string(),
        ip_address: Some("192.168.1.100".to_string()),
        enabled: Some(true),
    };
    let result = stub.configure_interface(input).await;
    println!("configure_interface result: {:?}", result);

    // Inspect the call log — useful for verifying client behavior in tests
    println!("\nCall log:");
    for entry in stub.get_call_log() {
        println!("  • {}", entry);
    }
}

// ---------------------------------------------------------------------------
// Part 2: Routing requests through the stub handler
// ---------------------------------------------------------------------------

/// Demonstrates routing RESTCONF requests through the generated router.
///
/// The router parses URL paths, deserializes request bodies, dispatches
/// to the appropriate handler method, and serializes responses.
async fn demo_request_routing() {
    println!("\n=== Part 2: Request Routing ===\n");

    let stub = StubDeviceManagementHandler::new();
    let router = RestconfRouter::new(stub, "/restconf");

    // Route a restart-device RPC request
    let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/restart-device")
        .with_header("Content-Type", "application/json")
        .with_body(br#"{"delay-seconds": 10}"#.to_vec());

    let response = router.route(request).await;
    println!(
        "restart-device → {} {}",
        response.status_code,
        String::from_utf8_lossy(&response.body)
    );

    // Route a get-system-info RPC request (no body needed)
    let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/get-system-info")
        .with_header("Content-Type", "application/json")
        .with_body(b"{}".to_vec());

    let response = router.route(request).await;
    println!(
        "get-system-info → {} {}",
        response.status_code,
        String::from_utf8_lossy(&response.body)
    );

    // Route an unknown path — returns 404
    let request = ServerRequest::new(HttpMethod::GET, "/restconf/data/unknown");
    let response = router.route(request).await;
    println!(
        "unknown path   → {} {}",
        response.status_code,
        String::from_utf8_lossy(&response.body)
    );
}

// ---------------------------------------------------------------------------
// Part 3: Custom handler with selective overrides
// ---------------------------------------------------------------------------

/// A production handler that overrides specific methods while delegating
/// the rest to the stub. This pattern lets you incrementally implement
/// real logic without touching every operation at once.
struct ProductionHandler {
    stub: StubDeviceManagementHandler,
    hostname: String,
    version: String,
}

impl ProductionHandler {
    fn new(hostname: &str, version: &str) -> Self {
        Self {
            stub: StubDeviceManagementHandler::new(),
            hostname: hostname.to_string(),
            version: version.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DeviceManagementHandler for ProductionHandler {
    // Override get_system_info with real data
    async fn get_system_info(&self) -> Result<GetSystemInfoOutput, ServerError> {
        Ok(GetSystemInfoOutput {
            hostname: Some(self.hostname.clone()),
            uptime_seconds: Some(86400),
            version: Some(self.version.clone()),
        })
    }

    // Override configure_interface with validation logic
    async fn configure_interface(
        &self,
        input: ConfigureInterfaceInput,
    ) -> Result<ConfigureInterfaceOutput, ServerError> {
        // Custom validation
        if input.interface_name.is_empty() {
            return Err(ServerError::ValidationError(
                "interface-name cannot be empty".to_string(),
            ));
        }

        println!(
            "  [ProductionHandler] Configuring {} with IP {:?}",
            input.interface_name, input.ip_address
        );

        Ok(ConfigureInterfaceOutput {
            success: Some(true),
            message: Some(format!("Configured {}", input.interface_name)),
        })
    }

    // Delegate restart_device to the stub (not yet implemented)
    async fn restart_device(
        &self,
        input: RestartDeviceInput,
    ) -> Result<RestartDeviceOutput, ServerError> {
        self.stub.restart_device(input).await
    }
}

/// Demonstrates mixing custom and stub handler implementations.
async fn demo_custom_handler() {
    println!("\n=== Part 3: Custom Handler ===\n");

    let handler = ProductionHandler::new("router-01.lab", "2.4.1");
    let router = RestconfRouter::new(handler, "/restconf");

    // get-system-info returns real data from our custom handler
    let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/get-system-info")
        .with_header("Content-Type", "application/json")
        .with_body(b"{}".to_vec());

    let response = router.route(request).await;
    println!(
        "get-system-info → {} {}",
        response.status_code,
        String::from_utf8_lossy(&response.body)
    );

    // configure-interface uses our custom validation and logic
    let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/configure-interface")
        .with_header("Content-Type", "application/json")
        .with_body(
            br#"{"interface-name": "eth0", "ip-address": "10.0.0.1", "enabled": true}"#.to_vec(),
        );

    let response = router.route(request).await;
    println!(
        "configure-interface → {} {}",
        response.status_code,
        String::from_utf8_lossy(&response.body)
    );

    // restart-device falls through to the stub
    let request = ServerRequest::new(HttpMethod::POST, "/restconf/operations/restart-device")
        .with_header("Content-Type", "application/json")
        .with_body(br#"{"delay-seconds": 0}"#.to_vec());

    let response = router.route(request).await;
    println!(
        "restart-device (stub) → {} {}",
        response.status_code,
        String::from_utf8_lossy(&response.body)
    );
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Server-Side Generation Example ===\n");
    println!("This example demonstrates server-side code generation from YANG schemas.");
    println!("The build.rs uses enable_server_generation(true) to produce:");
    println!("  • DeviceManagementHandler trait  (server/handlers.rs)");
    println!("  • StubDeviceManagementHandler    (server/stubs.rs)");
    println!("  • RestconfRouter                 (server/router.rs)");
    println!("  • HandlerRegistry                (server/registry.rs)");
    println!();

    // We use a simple single-threaded runtime since this is just a demo
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        demo_stub_handler().await;
        demo_request_routing().await;
        demo_custom_handler().await;
    });

    println!("\n=== Example Complete ===");
}
