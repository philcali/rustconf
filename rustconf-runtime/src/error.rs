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

/// Error type for server-side RESTCONF operations.
///
/// This enum covers all error conditions that can occur during
/// server-side RESTCONF request handling, from validation failures
/// to handler implementation errors.
///
/// # Examples
///
/// Handling different error types:
///
/// ```
/// use rustconf_runtime::ServerError;
///
/// fn handle_error(error: ServerError) {
///     let status = error.status_code();
///     match error {
///         ServerError::ValidationError(msg) => {
///             eprintln!("Validation failed ({}): {}", status, msg);
///         }
///         ServerError::NotFound(msg) => {
///             eprintln!("Resource not found ({}): {}", status, msg);
///         }
///         _ => {
///             eprintln!("Server error ({}): {}", status, error);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ServerError {
    /// Request validation failed.
    ///
    /// This occurs when incoming request data fails YANG-defined constraints
    /// such as range checks, pattern matching, or mandatory field requirements.
    ValidationError(String),

    /// Multiple validation errors occurred.
    ///
    /// This variant allows reporting multiple validation failures in a single
    /// response, which is useful when multiple fields violate constraints.
    MultipleValidationErrors(Vec<String>),

    /// Request deserialization failed.
    ///
    /// This occurs when the request body cannot be parsed as expected,
    /// usually indicating malformed JSON or XML.
    DeserializationError(String),

    /// Response serialization failed.
    ///
    /// This occurs when handler output cannot be converted to the requested
    /// format (JSON or XML), usually indicating a programming error.
    SerializationError(String),

    /// Handler implementation error.
    ///
    /// This represents errors that occur within handler implementations,
    /// such as business logic failures or internal processing errors.
    HandlerError(String),

    /// Resource not found.
    ///
    /// This occurs when the requested resource path does not match any
    /// YANG-defined data node or RPC operation.
    NotFound(String),

    /// Internal server error.
    ///
    /// This represents unexpected errors that don't fit other categories,
    /// such as system failures or unhandled edge cases.
    InternalError(String),
}

impl ServerError {
    /// Map error to HTTP status code.
    ///
    /// Returns the appropriate HTTP status code for this error type:
    /// - ValidationError: 400 Bad Request
    /// - MultipleValidationErrors: 400 Bad Request
    /// - DeserializationError: 400 Bad Request
    /// - NotFound: 404 Not Found
    /// - SerializationError: 500 Internal Server Error
    /// - HandlerError: 500 Internal Server Error
    /// - InternalError: 500 Internal Server Error
    pub fn status_code(&self) -> u16 {
        match self {
            ServerError::ValidationError(_) => 400,
            ServerError::MultipleValidationErrors(_) => 400,
            ServerError::DeserializationError(_) => 400,
            ServerError::NotFound(_) => 404,
            ServerError::SerializationError(_) => 500,
            ServerError::HandlerError(_) => 500,
            ServerError::InternalError(_) => 500,
        }
    }

    /// Format as RESTCONF error response according to RFC 8040.
    ///
    /// Returns a JSON string containing the error formatted according to
    /// the RESTCONF error response structure defined in RFC 8040 section 7.1.
    /// Supports multiple errors in a single response.
    ///
    /// # Example
    ///
    /// ```
    /// use rustconf_runtime::ServerError;
    ///
    /// let error = ServerError::ValidationError("port must be between 1 and 65535".to_string());
    /// let json = error.to_restconf_error();
    /// assert!(json.contains("invalid-value"));
    /// ```
    pub fn to_restconf_error(&self) -> String {
        let errors = match self {
            ServerError::ValidationError(msg) => {
                vec![("application", "invalid-value", msg.as_str())]
            }
            ServerError::MultipleValidationErrors(messages) => messages
                .iter()
                .map(|msg| ("application", "invalid-value", msg.as_str()))
                .collect(),
            ServerError::DeserializationError(msg) => {
                vec![("protocol", "malformed-message", msg.as_str())]
            }
            ServerError::NotFound(msg) => vec![("application", "invalid-value", msg.as_str())],
            ServerError::SerializationError(msg) => {
                vec![("application", "operation-failed", msg.as_str())]
            }
            ServerError::HandlerError(msg) => {
                vec![("application", "operation-failed", msg.as_str())]
            }
            ServerError::InternalError(msg) => {
                vec![("application", "operation-failed", msg.as_str())]
            }
        };

        // Build error array
        let error_array: Vec<serde_json::Value> = errors
            .into_iter()
            .map(|(error_type, error_tag, error_message)| {
                serde_json::json!({
                    "error-type": error_type,
                    "error-tag": error_tag,
                    "error-message": error_message
                })
            })
            .collect();

        serde_json::json!({
            "ietf-restconf:errors": {
                "error": error_array
            }
        })
        .to_string()
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServerError::MultipleValidationErrors(messages) => {
                write!(f, "Multiple validation errors: ")?;
                for (i, msg) in messages.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}", msg)?;
                }
                Ok(())
            }
            ServerError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            ServerError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ServerError::HandlerError(msg) => write!(f, "Handler error: {}", msg),
            ServerError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServerError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

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
