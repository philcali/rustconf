//! Error Handling Example
//!
//! This example demonstrates comprehensive error handling patterns
//! for RESTful RPC operations, including custom error mappers and
//! recovery strategies.

use async_trait::async_trait;

// Include the generated code
include!(concat!(env!("OUT_DIR"), "/device_management.rs"));

/// Mock transport that simulates different error scenarios
struct ErrorSimulatorTransport {
    scenario: ErrorScenario,
}

#[derive(Clone, Copy)]
enum ErrorScenario {
    Success,
    InvalidInput,
    Unauthorized,
    NotFound,
    ServerError,
    NetworkError,
}

impl ErrorSimulatorTransport {
    fn new(scenario: ErrorScenario) -> Self {
        Self { scenario }
    }
}

#[async_trait]
impl HttpTransport for ErrorSimulatorTransport {
    async fn execute(&self, _request: HttpRequest) -> Result<HttpResponse, RpcError> {
        match self.scenario {
            ErrorScenario::Success => Ok(HttpResponse {
                status_code: 200,
                headers: vec![],
                body: br#"{"hostname":"test-device","version":"1.0.0"}"#.to_vec(),
            }),
            ErrorScenario::InvalidInput => Ok(HttpResponse {
                status_code: 400,
                headers: vec![],
                body: b"Invalid input: Missing required field".to_vec(),
            }),
            ErrorScenario::Unauthorized => Ok(HttpResponse {
                status_code: 401,
                headers: vec![],
                body: b"Unauthorized: Token expired".to_vec(),
            }),
            ErrorScenario::NotFound => Ok(HttpResponse {
                status_code: 404,
                headers: vec![],
                body: b"Not found: Resource does not exist".to_vec(),
            }),
            ErrorScenario::ServerError => Ok(HttpResponse {
                status_code: 500,
                headers: vec![],
                body: b"Internal server error: Database connection failed".to_vec(),
            }),
            ErrorScenario::NetworkError => {
                Err(RpcError::TransportError("Connection refused".to_string()))
            }
        }
    }
}

/// Convert RpcError to user-friendly message
fn user_friendly_message(error: &RpcError) -> String {
    match error {
        RpcError::TransportError(_) => {
            "Unable to connect to the server. Please check your network connection.".to_string()
        }
        RpcError::SerializationError(_) => {
            "Failed to prepare the request. Please check your input data.".to_string()
        }
        RpcError::DeserializationError(_) => {
            "Received an unexpected response from the server.".to_string()
        }
        RpcError::ValidationError(msg) => {
            format!("Invalid input: {}", msg)
        }
        RpcError::HttpError {
            status_code,
            message,
        } => match *status_code {
            400 => format!("Bad request: {}", message),
            401 => "Authentication failed. Please check your credentials.".to_string(),
            404 => format!("Resource not found: {}", message),
            500..=599 => format!("Server error: {}", message),
            _ => format!("HTTP error {}: {}", status_code, message),
        },
        RpcError::ConfigurationError(msg) => {
            format!("Configuration error: {}", msg)
        }
        RpcError::NotImplemented => "This operation is not implemented".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Error Handling Example ===\n");

    // Example 1: Handling different error types
    println!("Example 1: Handling Different Error Types");
    println!("------------------------------------------\n");

    let scenarios = vec![
        ("Success", ErrorScenario::Success),
        ("Invalid Input (400)", ErrorScenario::InvalidInput),
        ("Unauthorized (401)", ErrorScenario::Unauthorized),
        ("Not Found (404)", ErrorScenario::NotFound),
        ("Server Error (500)", ErrorScenario::ServerError),
        ("Network Error", ErrorScenario::NetworkError),
    ];

    for (name, scenario) in scenarios {
        println!("Scenario: {}", name);
        let transport = ErrorSimulatorTransport::new(scenario);
        let client = RestconfClient::new("https://device.example.com", transport)?;

        match operations::get_system_info(&client).await {
            Ok(output) => {
                println!("   ✓ Success!");
                println!("   Hostname: {}", output.hostname.unwrap_or_default());
                println!("   Version: {}", output.version.unwrap_or_default());
            }
            Err(error) => {
                println!("   ✗ Error type: {:?}", std::mem::discriminant(&error));
                println!("   Technical: {}", error);
                println!("   User-friendly: {}", user_friendly_message(&error));
            }
        }
        println!();
    }

    // Example 2: Pattern matching on error types
    println!("Example 2: Pattern Matching on Errors");
    println!("--------------------------------------\n");

    let transport = ErrorSimulatorTransport::new(ErrorScenario::Unauthorized);
    let client = RestconfClient::new("https://device.example.com", transport)?;

    match operations::get_system_info(&client).await {
        Ok(output) => {
            println!("Success: {:?}", output);
        }
        Err(RpcError::HttpError {
            status_code: 401,
            message,
        }) => {
            println!("   Caught Unauthorized error!");
            println!("   Message: {}", message);
            println!("   Action: Refresh authentication token and retry");
        }
        Err(RpcError::HttpError {
            status_code: 500..=599,
            message,
        }) => {
            println!("   Caught Server error!");
            println!("   Message: {}", message);
            println!("   Action: Retry with exponential backoff");
        }
        Err(RpcError::TransportError(msg)) => {
            println!("   Caught Transport error!");
            println!("   Message: {}", msg);
            println!("   Action: Check network connectivity");
        }
        Err(e) => {
            println!("   Caught other error: {}", e);
            println!("   Action: Log and report to monitoring system");
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nWhat this example demonstrated:");
    println!("✓ Handling all RpcError variants");
    println!("✓ Converting technical errors to user-friendly messages");
    println!("✓ Pattern matching for specific error handling");
    println!("✓ Simulating different error scenarios");

    println!("\nBest practices:");
    println!("• Match on specific error types for targeted handling");
    println!("• Provide user-friendly error messages");
    println!("• Log detailed technical information for debugging");
    println!("• Implement retry logic for transient errors");
    println!("• Don't expose sensitive information in error messages");

    Ok(())
}
