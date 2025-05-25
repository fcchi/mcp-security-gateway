use serde_json::Value;
use std::fmt;
use thiserror::Error;
use tracing::{debug, error};

/// Common error type used in MCP Security Gateway
#[derive(Error, Debug)]
pub enum McpError {
    /// Authentication and authorization errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid request or input errors
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Resource not found errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Policy evaluation errors
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Sandbox execution errors
    #[error("Sandbox error: {0}")]
    Sandbox(String),

    /// Command execution errors
    #[error("Execution error: {0}")]
    Execution(String),

    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Temporary errors (retryable)
    #[error("Temporary error: {0}")]
    Temporary(String),

    /// External service communication errors
    #[error("External service error: {0}")]
    ExternalService(String),
}

/// Error code range definitions
pub mod error_code {
    // Authentication errors (1000-1999)
    pub const AUTH_INVALID_CREDENTIALS: u32 = 1001;
    pub const AUTH_EXPIRED_TOKEN: u32 = 1002;
    pub const AUTH_INSUFFICIENT_PERMISSIONS: u32 = 1003;

    // Input validation errors (2000-2999)
    pub const INPUT_INVALID_PARAMETER: u32 = 2001;
    pub const INPUT_MISSING_REQUIRED: u32 = 2002;
    pub const INPUT_INVALID_FORMAT: u32 = 2003;

    // Policy errors (3000-3999)
    pub const POLICY_COMMAND_NOT_ALLOWED: u32 = 3001;
    pub const POLICY_NETWORK_ACCESS_DENIED: u32 = 3002;
    pub const POLICY_FILE_ACCESS_DENIED: u32 = 3003;
    pub const POLICY_RESOURCE_LIMIT_EXCEEDED: u32 = 3004;

    // Sandbox errors (4000-4999)
    pub const SANDBOX_SETUP_FAILED: u32 = 4001;
    pub const SANDBOX_EXECUTION_FAILED: u32 = 4002;
    pub const SANDBOX_RESOURCE_LIMIT_EXCEEDED: u32 = 4003;

    // Internal errors (5000-5999)
    pub const INTERNAL_UNEXPECTED: u32 = 5001;
    pub const INTERNAL_DATABASE_ERROR: u32 = 5002;
    pub const INTERNAL_DEPENDENCY_FAILED: u32 = 5003;

    // New error codes
    pub const RESOURCE_NOT_FOUND: u32 = 6001;
}

/// Standard error response format
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Error details
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorDetail {
    pub code: u32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Macro to convert Result<T, E> to Result<T, McpError>
///
/// # Examples
///
/// ```
/// use mcp_common::to_mcp_result;
/// use std::fs::File;
///
/// fn read_config() -> mcp_common::error::McpResult<String> {
///     let file = to_mcp_result!(File::open("config.json"), "Cannot open config file");
///     // Further processing...
///     Ok("Configuration content".to_string())
/// }
/// ```
#[macro_export]
macro_rules! to_mcp_result {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return Err(mcp_common::error::McpError::Internal(format!(
                    "{}: {}",
                    $msg, err
                )))
            }
        }
    };
    ($expr:expr, $err_type:ident, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return Err(mcp_common::error::McpError::$err_type(format!(
                    "{}: {}",
                    $msg, err
                )))
            }
        }
    };
}

impl McpError {
    /// Get error code
    pub fn code(&self) -> u32 {
        match self {
            McpError::Auth(msg) if msg.contains("invalid") => error_code::AUTH_INVALID_CREDENTIALS,
            McpError::Auth(msg) if msg.contains("expired") => error_code::AUTH_EXPIRED_TOKEN,
            McpError::Auth(_) => error_code::AUTH_INSUFFICIENT_PERMISSIONS,

            McpError::InvalidRequest(msg) if msg.contains("missing") => {
                error_code::INPUT_MISSING_REQUIRED
            }
            McpError::InvalidRequest(msg) if msg.contains("format") => {
                error_code::INPUT_INVALID_FORMAT
            }
            McpError::InvalidRequest(_) => error_code::INPUT_INVALID_PARAMETER,

            McpError::NotFound(_) => error_code::RESOURCE_NOT_FOUND,

            McpError::PolicyViolation(msg) if msg.contains("command") => {
                error_code::POLICY_COMMAND_NOT_ALLOWED
            }
            McpError::PolicyViolation(msg) if msg.contains("network") => {
                error_code::POLICY_NETWORK_ACCESS_DENIED
            }
            McpError::PolicyViolation(msg) if msg.contains("file") => {
                error_code::POLICY_FILE_ACCESS_DENIED
            }
            McpError::PolicyViolation(msg) if msg.contains("resource") => {
                error_code::POLICY_RESOURCE_LIMIT_EXCEEDED
            }
            McpError::PolicyViolation(_) => error_code::POLICY_COMMAND_NOT_ALLOWED,

            McpError::Sandbox(msg) if msg.contains("setup") => error_code::SANDBOX_SETUP_FAILED,
            McpError::Sandbox(msg) if msg.contains("resource") => {
                error_code::SANDBOX_RESOURCE_LIMIT_EXCEEDED
            }
            McpError::Sandbox(_) => error_code::SANDBOX_EXECUTION_FAILED,

            McpError::Execution(_) => error_code::SANDBOX_EXECUTION_FAILED,

            McpError::Internal(msg) if msg.contains("database") => {
                error_code::INTERNAL_DATABASE_ERROR
            }
            McpError::Internal(msg) if msg.contains("dependency") => {
                error_code::INTERNAL_DEPENDENCY_FAILED
            }
            McpError::Internal(_) => error_code::INTERNAL_UNEXPECTED,

            McpError::Temporary(_) => error_code::INTERNAL_UNEXPECTED,
            McpError::ExternalService(_) => error_code::INTERNAL_DEPENDENCY_FAILED,
        }
    }

    /// Generate error response
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.code(),
                message: self.to_string(),
                details: None,
            },
        }
    }

    /// Generate error response with details
    pub fn with_details(&self, details: Value) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.code(),
                message: self.to_string(),
                details: Some(details),
            },
        }
    }

    /// Create policy violation error with custom code
    pub fn policy_violation(
        message: impl Into<String>,
        _code: u32,
        details: Option<Value>,
    ) -> Self {
        // Create basic error
        let error = McpError::PolicyViolation(message.into());

        // Log details
        if let Some(details_json) = &details {
            debug!("Policy violation details: {}", details_json);
        }

        error
    }

    /// Create sandbox error with custom code
    pub fn sandbox_error(message: impl Into<String>, _code: u32, details: Option<Value>) -> Self {
        // Create basic error
        let error = McpError::Sandbox(message.into());

        // Log details
        if let Some(details_json) = &details {
            debug!("Sandbox error details: {}", details_json);
        }

        error
    }

    /// Helper function to create error response from other McpError
    pub fn from_error<E: fmt::Display>(err: E, _code: u32) -> Self {
        McpError::Internal(format!("{}", err))
    }
}

/// Result type alias
pub type McpResult<T> = std::result::Result<T, McpError>;

// Conversion implementation from standard errors to McpError
impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => McpError::NotFound(format!("File not found: {}", err)),
            std::io::ErrorKind::PermissionDenied => {
                McpError::PolicyViolation(format!("Permission denied: {}", err))
            }
            std::io::ErrorKind::ConnectionRefused => {
                McpError::ExternalService(format!("Connection refused: {}", err))
            }
            std::io::ErrorKind::ConnectionReset => {
                McpError::Temporary(format!("Connection reset: {}", err))
            }
            std::io::ErrorKind::ConnectionAborted => {
                McpError::Temporary(format!("Connection aborted: {}", err))
            }
            std::io::ErrorKind::NotConnected => {
                McpError::ExternalService(format!("Not connected: {}", err))
            }
            std::io::ErrorKind::TimedOut => McpError::Execution(format!("Timed out: {}", err)),
            _ => McpError::Internal(format!("I/O error: {}", err)),
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::InvalidRequest(format!("JSON parsing error: {}", err))
    }
}

impl From<std::str::Utf8Error> for McpError {
    fn from(err: std::str::Utf8Error) -> Self {
        McpError::InvalidRequest(format!("Invalid UTF-8 sequence: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for McpError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        McpError::InvalidRequest(format!("Invalid UTF-8 sequence: {}", err))
    }
}

/// Wrapper for traceable errors
#[macro_export]
macro_rules! trace_err {
    ($expr:expr, $level:ident, $msg:expr) => {
        match $expr {
            Ok(val) => Ok(val),
            Err(err) => {
                tracing::$level!("{}: {}", $msg, err);
                Err(err)
            }
        }
    };
}

/// Error trace with early return
#[macro_export]
macro_rules! try_trace {
    ($expr:expr, $level:ident, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                tracing::$level!("{}: {}", $msg, err);
                return Err(err.into());
            }
        }
    };
}

/// Helper to convert generic Result to McpResult
pub trait IntoMcpResult<T, E> {
    /// Convert Result<T, E> to McpResult<T>
    fn into_mcp_result(self) -> McpResult<T>;

    /// Convert Result<T, E> to McpResult<T> with custom message
    fn into_mcp_result_with_msg(self, msg: impl Into<String>) -> McpResult<T>;
}

impl<T, E: std::fmt::Display> IntoMcpResult<T, E> for Result<T, E> {
    fn into_mcp_result(self) -> McpResult<T> {
        self.map_err(|e| McpError::Internal(format!("{}", e)))
    }

    fn into_mcp_result_with_msg(self, msg: impl Into<String>) -> McpResult<T> {
        self.map_err(|e| McpError::Internal(format!("{}: {}", msg.into(), e)))
    }
}

/// Conversion trait for converting various errors to McpError
pub trait ToMcpError {
    /// Convert to McpError
    fn to_mcp_error(self) -> McpError;

    /// Convert to McpError with custom message
    fn to_mcp_error_with_msg(self, msg: impl Into<String>) -> McpError;
}

// Generic implementation of ToMcpError (applies only to types that implement From<E>)
// Note: This implementation doesn't apply to mcp-error's ToMcpError itself
// This provides necessary conversions while avoiding conflicts
impl<E> ToMcpError for E
where
    E: std::fmt::Display,
    E: Into<McpError>,
    E: Sized,
    E: 'static,
{
    fn to_mcp_error(self) -> McpError {
        self.into()
    }

    fn to_mcp_error_with_msg(self, msg: impl Into<String>) -> McpError {
        let error = self.into();
        match error {
            McpError::Internal(e) => McpError::Internal(format!("{}: {}", msg.into(), e)),
            McpError::InvalidRequest(e) => {
                McpError::InvalidRequest(format!("{}: {}", msg.into(), e))
            }
            McpError::PolicyViolation(e) => {
                McpError::PolicyViolation(format!("{}: {}", msg.into(), e))
            }
            McpError::Auth(e) => McpError::Auth(format!("{}: {}", msg.into(), e)),
            McpError::NotFound(e) => McpError::NotFound(format!("{}: {}", msg.into(), e)),
            McpError::Sandbox(e) => McpError::Sandbox(format!("{}: {}", msg.into(), e)),
            McpError::Execution(e) => McpError::Execution(format!("{}: {}", msg.into(), e)),
            McpError::Temporary(e) => McpError::Temporary(format!("{}: {}", msg.into(), e)),
            McpError::ExternalService(e) => {
                McpError::ExternalService(format!("{}: {}", msg.into(), e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_into_mcp_result() {
        let ok_result: Result<i32, &str> = Ok(42);
        let err_result: Result<i32, &str> = Err("Error occurred");

        assert_eq!(ok_result.into_mcp_result().unwrap(), 42);
        assert!(matches!(
            err_result.into_mcp_result(),
            Err(McpError::Internal(_))
        ));
    }

    #[test]
    fn test_io_error_conversion() {
        let not_found = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let perm_denied =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission error");

        assert!(matches!(not_found.to_mcp_error(), McpError::NotFound(_)));
        assert!(matches!(
            perm_denied.to_mcp_error(),
            McpError::PolicyViolation(_)
        ));
    }
}
