//! Test End-User Application
//!
//! This application demonstrates using the test-intermediate-crate
//! without requiring rustconf as a dependency or build.rs.

use test_intermediate_crate::{Device, RestconfClient, RpcError};

// Mock transport for testing without actual HTTP calls
struct MockTransport;

#[async_trait::async_trait]
impl test_intermediate_crate::HttpTransport for MockTransport {
    async fn execute(
        &self,
        _request: test_intermediate_crate::HttpRequest,
    ) -> Result<test_intermediate_crate::HttpResponse, RpcError> {
        // Return a mock successful response
        Ok(test_intermediate_crate::HttpResponse {
            status_code: 200,
            headers: vec![],
            body: b"{}".to_vec(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test End-User Application");
    println!("=========================");
    
    // Create a device instance (demonstrates using generated types)
    let device = Device {
        name: Some("test-device-1".to_string()),
        enabled: Some(true),
        port: Some(8080),
    };
    
    println!("Created device: {:?}", device);
    
    // Create a RestconfClient with mock transport
    let client = RestconfClient::new("http://localhost:8080", MockTransport)?;
    println!("Created RestconfClient successfully");
    
    // Demonstrate that we can use the client type
    println!("Client base URL: http://localhost:8080");
    
    println!("\nSuccess! End-user application works without rustconf dependency.");
    
    Ok(())
}
