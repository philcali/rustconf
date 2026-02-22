//! Error types for RESTCONF operations.

use crate::transport::HttpResponse;
use std::fmt;

/// Error type for RESTCONF RPC operations.
///
/// This enum covers all error conditions that can occur during
/// RESTCONF operations, from transport failures to validation errors.
///
/// # Examples
///
/// Handling different error types:
///
/// ```
/// use rustconf_runtime::RpcError;
///
/// fn handle_error(error: RpcError) {
///     match error {
///         RpcError::TransportError(msg) => {
///             eprintln!("Network error: {}", msg);
///         }
///         RpcError::HttpError { status_code, message } => {
///             eprintln!("HTTP {} error: {}", status_code, message);
///         }
///         RpcError::ValidationError(msg) => {
///             eprintln!("Validation failed: {}", msg);
///         }
///         _ => {
///             eprintln!("Other error: {}", error);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum RpcError {
    /// Error occurred in the HTTP transport layer.
    ///
    /// This typically indicates network connectivity issues, DNS resolution
    /// failures, or other low-level transport problems.
    TransportError(String),

    /// Error serializing request data to JSON.
    ///
    /// This occurs when request data cannot be converted to JSON format,
    /// usually indicating a programming error or invalid data structure.
    SerializationError(String),

    /// Error deserializing response data from JSON.
    ///
    /// This occurs when the server response cannot be parsed as expected,
    /// which may indicate API version mismatch or malformed responses.
    DeserializationError(String),

    /// Validation error (e.g., YANG constraint violation).
    ///
    /// This occurs when data fails YANG-defined constraints such as
    /// range checks, pattern matching, or mandatory field requirements.
    ValidationError(String),

    /// HTTP error response from server.
    ///
    /// This represents HTTP-level errors (4xx, 5xx status codes) returned
    /// by the RESTCONF server, including the status code and error message.
    HttpError {
        /// HTTP status code (e.g., 404, 500)
        status_code: u16,
        /// Error message from the server
        message: String,
    },

    /// Configuration error (e.g., invalid base URL).
    ///
    /// This occurs when the client is misconfigured, such as providing
    /// an empty or malformed base URL.
    ConfigurationError(String),

    /// Operation not implemented.
    ///
    /// This indicates that a requested operation is not supported by
    /// the current implementation or configuration.
    NotImplemented,
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcError::TransportError(msg) => write!(f, "Transport error: {}", msg),
            RpcError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            RpcError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            RpcError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RpcError::HttpError {
                status_code,
                message,
            } => {
                write!(f, "HTTP error {}: {}", status_code, message)
            }
            RpcError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            RpcError::NotImplemented => write!(f, "Operation not implemented"),
        }
    }
}

impl std::error::Error for RpcError {}

/// Trait for mapping HTTP responses to RpcError.
///
/// This allows customization of error handling for different RESTCONF servers
/// that may have different error response formats.
pub trait ErrorMapper: Send + Sync {
    /// Map an HTTP response to an RpcError.
    fn map_error(&self, response: &HttpResponse) -> RpcError;
}

/// Default error mapper implementation.
///
/// This mapper attempts to parse JSON error responses following the
/// RESTCONF error format (RFC 8040), and falls back to generic HTTP errors.
pub struct DefaultErrorMapper;

impl ErrorMapper for DefaultErrorMapper {
    fn map_error(&self, response: &HttpResponse) -> RpcError {
        // Try to parse as JSON error response
        if let Ok(body_str) = std::str::from_utf8(&response.body) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body_str) {
                // Try to extract error message from RESTCONF error format
                if let Some(errors) = json.get("ietf-restconf:errors") {
                    if let Some(error_array) = errors.get("error").and_then(|e| e.as_array()) {
                        if let Some(first_error) = error_array.first() {
                            if let Some(message) =
                                first_error.get("error-message").and_then(|m| m.as_str())
                            {
                                return RpcError::HttpError {
                                    status_code: response.status_code,
                                    message: message.to_string(),
                                };
                            }
                        }
                    }
                }

                // Try generic "message" or "error" fields
                if let Some(message) = json
                    .get("message")
                    .or_else(|| json.get("error"))
                    .and_then(|m| m.as_str())
                {
                    return RpcError::HttpError {
                        status_code: response.status_code,
                        message: message.to_string(),
                    };
                }
            }
        }

        // Fall back to generic HTTP error
        RpcError::HttpError {
            status_code: response.status_code,
            message: format!("HTTP {} error", response.status_code),
        }
    }
}
