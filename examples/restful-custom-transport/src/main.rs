//! Custom Transport Implementation Example
//!
//! This example demonstrates how to implement a custom HTTP transport
//! with retry logic, custom headers, and timeout configuration.

use async_trait::async_trait;
use std::time::Duration;

// Include the generated code
include!(concat!(env!("OUT_DIR"), "/device_management.rs"));

/// Custom HTTP transport with retry logic and custom headers
struct CustomTransport {
    client: reqwest::Client,
    max_retries: u32,
    custom_header_value: String,
}

impl CustomTransport {
    /// Create a new custom transport with specific configuration
    fn new(custom_header_value: String, max_retries: u32, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            max_retries,
            custom_header_value,
        }
    }
}

#[async_trait]
impl HttpTransport for CustomTransport {
    async fn execute(&self, mut request: HttpRequest) -> Result<HttpResponse, RpcError> {
        // Add custom header to all requests
        request.headers.push((
            "X-Custom-App-Version".to_string(),
            self.custom_header_value.clone(),
        ));
        println!(
            "   [CustomTransport] Added custom header: X-Custom-App-Version: {}",
            self.custom_header_value
        );

        // Retry logic with exponential backoff
        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let backoff_ms = 100 * 2_u64.pow(attempt);
                println!(
                    "   [CustomTransport] Retry attempt {} after {}ms",
                    attempt, backoff_ms
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }

            // Convert HttpMethod to reqwest::Method
            let method = match request.method {
                HttpMethod::GET => reqwest::Method::GET,
                HttpMethod::POST => reqwest::Method::POST,
                HttpMethod::PUT => reqwest::Method::PUT,
                HttpMethod::DELETE => reqwest::Method::DELETE,
                HttpMethod::PATCH => reqwest::Method::PATCH,
            };

            // Build and execute request
            let mut req_builder = self.client.request(method, &request.url);
            for (key, value) in &request.headers {
                req_builder = req_builder.header(key, value);
            }
            if let Some(ref body) = request.body {
                req_builder = req_builder.body(body.clone());
            }

            match req_builder.send().await {
                Ok(response) => {
                    println!(
                        "   [CustomTransport] Request succeeded on attempt {}",
                        attempt + 1
                    );

                    let status_code = response.status().as_u16();
                    let headers = response
                        .headers()
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                        .collect();
                    let body = response
                        .bytes()
                        .await
                        .map_err(|e| {
                            RpcError::TransportError(format!("Failed to read response body: {}", e))
                        })?
                        .to_vec();

                    return Ok(HttpResponse {
                        status_code,
                        headers,
                        body,
                    });
                }
                Err(e) => {
                    println!(
                        "   [CustomTransport] Request failed on attempt {}: {}",
                        attempt + 1,
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(RpcError::TransportError(format!(
            "HTTP request failed after {} retries: {}",
            self.max_retries,
            last_error.unwrap()
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Transport Implementation Example ===\n");

    // Create a custom transport with specific configuration
    println!("1. Creating custom transport...");
    println!("   - Custom header: X-Custom-App-Version: v1.2.3");
    println!("   - Max retries: 3");
    println!("   - Timeout: 30 seconds");
    let transport = CustomTransport::new("v1.2.3".to_string(), 3, 30);

    // Create a RESTCONF client with the custom transport
    println!("\n2. Creating RestconfClient with custom transport...");
    let client = RestconfClient::new("https://device.example.com", transport)?;
    println!("   Base URL: {}", client.base_url());

    // Make a request to demonstrate the custom transport
    println!("\n3. Making a request (will demonstrate retry logic)...");
    match operations::get_system_info(&client).await {
        Ok(output) => {
            println!("\n   ✓ Success!");
            println!("   Hostname: {}", output.hostname.unwrap_or_default());
            println!("   Version: {}", output.version.unwrap_or_default());
        }
        Err(e) => {
            println!("\n   ✗ Error (expected): {}", e);
            println!("\n   This is expected since we don't have a real RESTCONF server.");
            println!("   Notice how the custom transport:");
            println!("   • Added the custom header automatically");
            println!("   • Attempted retries with exponential backoff");
            println!("   • Applied the configured timeout");
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nWhat this example demonstrated:");
    println!("✓ Implementing the HttpTransport trait");
    println!("✓ Adding retry logic with exponential backoff");
    println!("✓ Injecting custom headers");
    println!("✓ Configuring timeouts");
    println!("✓ Using the custom transport with RestconfClient");

    println!("\nWhen to use custom transports:");
    println!("• Integration with existing HTTP infrastructure");
    println!("• Adding application-specific behavior");
    println!("• Mock transports for testing");
    println!("• Special authentication or signing requirements");
    println!("• Custom retry or circuit breaker logic");

    Ok(())
}
