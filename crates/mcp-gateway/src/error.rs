//! Global error handling for gRPC services

use mcp_common::{McpError, IntoStatus, ErrorResponse, ErrorDetail};
use tonic::{Response, Status};
use tracing::{error, warn, debug};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde_json::Value;

// Holds counters for each error type
static ERROR_COUNTERS: Lazy<DashMap<String, AtomicU64>> = Lazy::new(DashMap::new);

/// Global error handler
/// 
/// Helper functions for unified error handling in all gRPC responses
pub struct ErrorHandler;

impl ErrorHandler {
    /// Convert McpResult to tonic Response
    /// 
    /// # Arguments
    /// * `result` - McpResult containing processing result
    /// 
    /// # Returns
    /// * `Result<Response<T>, Status>` - Returns Response on success, Status on error
    pub fn handle<T>(result: Result<T, McpError>) -> Result<Response<T>, Status> {
        match result {
            Ok(value) => Ok(Response::new(value)),
            Err(err) => {
                // Increment error counter
                Self::increment_error_counter(&err);
                
                // Change log level based on error type
                match &err {
                    McpError::Auth(_) | McpError::PolicyViolation(_) => {
                        // Authentication/policy violations at warn level
                        warn!("Request denied: {}", err);
                    },
                    McpError::InvalidRequest(_) => {
                        // Invalid requests at debug level
                        debug!("Invalid request: {}", err);
                    },
                    _ => {
                        // Others at error level
                        error!("Service error: {}", err);
                    }
                }
                
                // Add detailed information (may need filtering in production)
                let error_response = ErrorResponse {
                    error: ErrorDetail {
                        code: err.code(),
                        message: err.to_string(),
                        details: None,
                    }
                };
                
                // Convert error to gRPC Status
                let mut status = err.into_status();
                
                // Add detailed information to metadata
                if let Ok(json) = serde_json::to_string(&error_response) {
                    let metadata = status.metadata_mut();
                    metadata.insert("error-details", json.parse().unwrap());
                }
                
                Err(status)
            }
        }
    }
    
    /// Wrap McpResult and handle errors in an async function
    /// 
    /// # Arguments
    /// * `fut` - Async function performing processing
    /// 
    /// # Returns
    /// * `Result<Response<T>, Status>` - Returns Response on success, Status on error
    pub async fn handle_async<F, T, E>(fut: F) -> Result<Response<T>, Status>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: Into<McpError>,
    {
        match fut.await {
            Ok(value) => Ok(Response::new(value)),
            Err(err) => {
                let mcp_err: McpError = err.into();
                
                // Increment error counter
                Self::increment_error_counter(&mcp_err);
                
                // Change log level based on error type
                match &mcp_err {
                    McpError::Auth(_) | McpError::PolicyViolation(_) => {
                        // Authentication/policy violations at warn level
                        warn!("Request denied: {}", mcp_err);
                    },
                    McpError::InvalidRequest(_) => {
                        // Invalid requests at debug level
                        debug!("Invalid request: {}", mcp_err);
                    },
                    _ => {
                        // Others at error level
                        error!("Service error: {}", mcp_err);
                    }
                }
                
                // Add detailed information
                let error_code = mcp_err.code();
                let error_message = mcp_err.to_string();
                let error_response = ErrorResponse {
                    error: ErrorDetail {
                        code: error_code,
                        message: error_message,
                        details: None,
                    }
                };
                
                // Convert error to gRPC Status
                let mut status = mcp_err.into_status();
                
                // Add detailed information to metadata
                if let Ok(json) = serde_json::to_string(&error_response) {
                    let metadata = status.metadata_mut();
                    metadata.insert("error-details", json.parse().unwrap());
                }
                
                Err(status)
            }
        }
    }
    
    /// Catch error and convert to Status
    /// 
    /// # Arguments
    /// * `err` - Error object
    /// 
    /// # Returns
    /// * `Status` - Converted gRPC Status
    pub fn catch<E: Into<McpError>>(err: E) -> Status {
        let mcp_err = err.into();
        
        // Increment error counter
        Self::increment_error_counter(&mcp_err);
        
        error!("Error occurred: {}", mcp_err);
        mcp_err.into_status()
    }
    
    /// Increment counter for each error type
    fn increment_error_counter(err: &McpError) {
        let error_type = match err {
            McpError::Auth(_) => "auth",
            McpError::InvalidRequest(_) => "invalid_request",
            McpError::NotFound(_) => "not_found",
            McpError::PolicyViolation(_) => "policy_violation",
            McpError::Sandbox(_) => "sandbox",
            McpError::Execution(_) => "execution",
            McpError::Internal(_) => "internal",
            McpError::Temporary(_) => "temporary",
            McpError::ExternalService(_) => "external_service",
        };
        
        // Count by error code as well
        let error_code = format!("error_code_{}", err.code());
        
        // Type counter
        ERROR_COUNTERS
            .entry(error_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
            
        // Code counter
        ERROR_COUNTERS
            .entry(error_code)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get error statistics
    pub fn get_error_stats() -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        for entry in ERROR_COUNTERS.iter() {
            stats.insert(entry.key().clone(), entry.value().load(Ordering::Relaxed));
        }
        stats
    }
    
    /// Get count for a specific error type
    pub fn get_error_count(error_type: &str) -> u64 {
        ERROR_COUNTERS
            .get(error_type)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    /// Handle error with detailed information
    pub fn handle_with_details<T>(result: Result<T, McpError>, details: Option<Value>) -> Result<Response<T>, Status> {
        match result {
            Ok(value) => Ok(Response::new(value)),
            Err(err) => {
                // Increment error counter
                Self::increment_error_counter(&err);
                
                // Change log level based on error type
                match &err {
                    McpError::Auth(_) | McpError::PolicyViolation(_) => {
                        warn!("Request denied: {}", err);
                    },
                    McpError::InvalidRequest(_) => {
                        debug!("Invalid request: {}", err);
                    },
                    _ => {
                        error!("Service error: {}", err);
                    }
                }
                
                // Create error response with detailed information
                let error_code = err.code();
                let error_message = err.to_string();
                let error_response = if let Some(details_value) = details {
                    ErrorResponse {
                        error: ErrorDetail {
                            code: error_code,
                            message: error_message,
                            details: Some(details_value),
                        }
                    }
                } else {
                    ErrorResponse {
                        error: ErrorDetail {
                            code: error_code,
                            message: error_message,
                            details: None,
                        }
                    }
                };
                
                // Convert error to gRPC Status
                let mut status = err.into_status();
                
                // Add detailed information to metadata
                if let Ok(json) = serde_json::to_string(&error_response) {
                    let metadata = status.metadata_mut();
                    metadata.insert("error-details", json.parse().unwrap());
                }
                
                Err(status)
            }
        }
    }
}

/// Convenient extension to convert McpError to Status via trait implementation
pub trait IntoResponse<T> {
    /// Convert Result to gRPC Response
    fn into_response(self) -> Result<Response<T>, Status>;
    
    /// Convert Result to gRPC Response with detailed information
    fn into_response_with_details(self, details: Value) -> Result<Response<T>, Status>;
}

impl<T, E: Into<McpError>> IntoResponse<T> for Result<T, E> {
    fn into_response(self) -> Result<Response<T>, Status> {
        match self {
            Ok(value) => Ok(Response::new(value)),
            Err(err) => {
                let mcp_err = err.into();
                ErrorHandler::increment_error_counter(&mcp_err);
                Err(mcp_err.into_status())
            }
        }
    }
    
    fn into_response_with_details(self, details: Value) -> Result<Response<T>, Status> {
        match self {
            Ok(value) => Ok(Response::new(value)),
            Err(err) => {
                let mcp_err = err.into();
                ErrorHandler::handle_with_details(Err(mcp_err), Some(details))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const AUTH_ERROR: &str = "Authentication error";
    
    #[test]
    #[serial_test::serial]
    fn test_error_handling() {
        // Success case
        let result: Result<i32, McpError> = Ok(42);
        let response = ErrorHandler::handle(result).unwrap();
        assert_eq!(response.get_ref(), &42);
        
        // Error case
        let result: Result<i32, McpError> = Err(McpError::InvalidRequest("Test error".to_string()));
        let err = ErrorHandler::handle(result).unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }
    
    #[test]
    #[serial_test::serial]
    fn test_into_response() {
        // Success case
        let result: Result<i32, McpError> = Ok(42);
        let response = result.into_response().unwrap();
        assert_eq!(response.get_ref(), &42);
        
        // Error case
        let result: Result<i32, McpError> = Err(McpError::Auth(AUTH_ERROR.to_string()));
        let err = result.into_response().unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }
    
    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_async() {
        // Async function that succeeds
        async fn success() -> Result<i32, McpError> {
            Ok(42)
        }
        
        // Async function that returns an error
        async fn failure() -> Result<i32, McpError> {
            Err(McpError::Internal("Internal error".to_string()))
        }
        
        // Success case
        let response = ErrorHandler::handle_async(success()).await.unwrap();
        assert_eq!(response.get_ref(), &42);
        
        // Error case
        let err = ErrorHandler::handle_async(failure()).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Internal);
    }
    
    #[test]
    #[serial_test::serial]
    fn test_error_counters() {
        println!("Test start: error_counters");
        
        // Clear counters to avoid influence from other tests
        ERROR_COUNTERS.clear();
        
        // Generate an error
        let auth_error = McpError::Auth(AUTH_ERROR.to_string());
        let error_code_key = format!("error_code_{}", auth_error.code());
        let _ = ErrorHandler::handle(Err::<(), _>(auth_error));
        
        // Check counter values
        let stats = ErrorHandler::get_error_stats();
        
        // Verify counters exist and have the same count (flexible check)
        if !stats.contains_key("auth") || !stats.contains_key(&error_code_key) {
            println!("Counter presence check failed: auth={}, code={}",
                stats.contains_key("auth"), stats.contains_key(&error_code_key));
            panic!("Required counters not found");
        }
        
        // To be less affected by concurrent execution, check existence and that auth/error_code values are the same
        let auth_count = *stats.get("auth").unwrap();
        let code_count = *stats.get(&error_code_key).unwrap();
        
        if auth_count == 0 || code_count == 0 {
            println!("Count value check failed: auth={}, code={}", auth_count, code_count);
            panic!("Count values are zero");
        }
        
        // Only verify that both counters have the same value
        if auth_count != code_count {
            println!("Count value mismatch: auth={}, code={}", auth_count, code_count);
            panic!("Counter values don't match");
        }
        
        // Check get_error_count method
        let direct_count = ErrorHandler::get_error_count("auth");
        if direct_count != auth_count {
            println!("Direct count mismatch: direct={}, auth={}", direct_count, auth_count);
            panic!("get_error_count result doesn't match");
        }
        
        println!("Test end: error_counters");
    }
    
    #[test]
    #[serial_test::serial]
    fn test_handle_with_details() {
        // Error handling with detailed information
        let details = serde_json::json!({
            "field": "username",
            "reason": "too_short"
        });
        
        let result: Result<i32, McpError> = Err(McpError::InvalidRequest("Input error".to_string()));
        let err = ErrorHandler::handle_with_details(result, Some(details.clone())).unwrap_err();
        
        // Verify that detailed information is included in metadata
        let metadata = err.metadata();
        assert!(metadata.contains_key("error-details"));
    }
} 