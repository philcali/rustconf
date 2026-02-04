//! Request Interceptor Example
//!
//! This example demonstrates how to implement and use request interceptors
//! for authentication, logging, and custom request/response handling.

use async_trait::async_trait;

// Include the generated code
include!(concat!(env!("OUT_DIR"), "/device_management.rs"));

/// Authentication interceptor that adds Bearer token to all requests
struct AuthInterceptor {
    token: String,
}

impl AuthInterceptor {
    fn new(token: String) -> Self {
        Self { token }
    }
}

#[async_trait]
impl RequestInterceptor for AuthInterceptor {
    async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
        println!("   [AuthInterceptor] Adding Authorization header");
        request.headers.push((
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        ));
        Ok(())
    }

    async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError> {
        if response.status_code == 401 {
            println!("   [AuthInterceptor] Detected 401 Unauthorized - token may be expired");
            return Err(RpcError::Unauthorized(
                "Token expired or invalid".to_string(),
            ));
        }
        Ok(())
    }
}

/// Logging interceptor that records request and response details
struct LoggingInterceptor;

#[async_trait]
impl RequestInterceptor for LoggingInterceptor {
    async fn before_request(&self, request: &mut HttpRequest) -> Result<(), RpcError> {
        println!("   [LoggingInterceptor] Sending request:");
        println!("     Method: {:?}", request.method);
        println!("     URL: {}", request.url);
        println!("     Headers: {} headers", request.headers.len());
        if let Some(ref body) = request.body {
            println!("     Body size: {} bytes", body.len());
        }
        Ok(())
    }

    async fn after_response(&self, response: &HttpResponse) -> Result<(), RpcError> {
        println!("   [LoggingInterceptor] Received response:");
        println!("     Status: {}", response.status_code);
        println!("     Headers: {} headers", response.headers.len());
        println!("     Body size: {} bytes", response.body.len());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Request Interceptor Example ===\n");

    #[cfg(not(feature = "reqwest-client"))]
    {
        println!("ERROR: reqwest-client feature not enabled!");
        println!("Run with: cargo run --example restful-interceptor --features reqwest-client");
        return Ok(());
    }

    #[cfg(feature = "reqwest-client")]
    {
        // Example 1: Using AuthInterceptor
        println!("Example 1: Authentication Interceptor");
        println!("--------------------------------------");
        let transport = reqwest_adapter::ReqwestTransport::new();
        let client = RestconfClient::new("https://device.example.com", transport)?
            .with_interceptor(AuthInterceptor::new("my-secret-token-12345".to_string()));

        println!("Calling get_system_info with authentication...");
        match operations::get_system_info(&client).await {
            Ok(output) => {
                println!("   ✓ Success!");
                println!("   Hostname: {}", output.hostname.unwrap_or_default());
            }
            Err(e) => {
                println!("   ✗ Error (expected): {}", e);
                println!("   Notice how the Authorization header was added automatically!");
            }
        }

        // Example 2: Using LoggingInterceptor
        println!("\nExample 2: Logging Interceptor");
        println!("-------------------------------");
        let transport = reqwest_adapter::ReqwestTransport::new();
        let client = RestconfClient::new("https://device.example.com", transport)?
            .with_interceptor(LoggingInterceptor);

        println!("Calling get_system_info with logging...");
        match operations::get_system_info(&client).await {
            Ok(output) => {
                println!("   ✓ Success!");
                println!("   Hostname: {}", output.hostname.unwrap_or_default());
            }
            Err(e) => {
                println!("   ✗ Error (expected): {}", e);
                println!("   Notice how all request/response details were logged!");
            }
        }

        println!("\n=== Example Complete ===");
        println!("\nWhat this example demonstrated:");
        println!("✓ Implementing the RequestInterceptor trait");
        println!("✓ Adding authentication headers with before_request");
        println!("✓ Validating responses with after_response");
        println!("✓ Logging request/response details");
        println!("✓ Aborting requests on authentication errors");

        println!("\nCommon interceptor use cases:");
        println!("• Authentication (Bearer tokens, API keys, OAuth)");
        println!("• Logging and monitoring");
        println!("• Request signing");
        println!("• Custom header injection");
        println!("• Response validation");
        println!("• Rate limiting");
    }

    Ok(())
}
