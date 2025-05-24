use crate::error::{McpError, McpResult};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Get current UNIX timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

/// Get current time string in ISO 8601 format
pub fn current_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Generate a unique task ID
pub fn generate_task_id() -> String {
    format!("task-{}", Uuid::new_v4().simple())
}

/// Return an iterator that splits a slice into chunks of specified size
pub fn chunk_slice<T>(slice: &[T], chunk_size: usize) -> impl Iterator<Item = &[T]> {
    slice.chunks(chunk_size)
}

/// Truncate string (replace end with "..." if exceeds maximum length)
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        // For extremely short cases, use only dots
        ".".repeat(max_len)
    } else {
        format!("{}...", &s[0..max_len.saturating_sub(3)])
    }
}

/// Safe JSON parsing (converts errors to McpError::InvalidRequest)
pub fn parse_json<T>(json_str: &str) -> McpResult<T> 
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(json_str)
        .map_err(|e| McpError::InvalidRequest(format!("Failed to parse JSON: {}", e)))
}

/// Get environment variable (returns McpError::Internal if not exists)
pub fn get_env_var(name: &str) -> McpResult<String> {
    std::env::var(name)
        .map_err(|_| McpError::Internal(format!("Environment variable {} is not set", name)))
}

/// Get environment variable with default value
pub fn get_env_var_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("abc", 2), ".."); // Extreme short case
    }

    #[test]
    fn test_parse_json() {
        #[derive(serde::Deserialize)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let json = r#"{"name": "test", "value": 42}"#;
        let result: McpResult<TestStruct> = parse_json(json);
        assert!(result.is_ok());
        
        let test_struct = result.unwrap();
        assert_eq!(test_struct.name, "test");
        assert_eq!(test_struct.value, 42);

        let invalid_json = r#"{"name": "test", "value": "not a number"}"#;
        let result: McpResult<TestStruct> = parse_json(invalid_json);
        assert!(result.is_err());
    }
} 