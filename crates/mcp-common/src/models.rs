use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task has been created but not yet executed
    Created,
    /// Task is in the execution queue
    Queued,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task execution time exceeded the limit
    TimedOut,
}

/// Task type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Command execution task
    Command,
    /// File operation task
    File,
    /// HTTP request task
    HttpRequest,
}

/// Basic task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Unique task ID
    pub task_id: String,
    /// Task type
    pub task_type: TaskType,
    /// Current task status
    pub status: TaskStatus,
    /// Task creation date (ISO 8601 format)
    pub created_at: String,
    /// Task start date (ISO 8601 format)
    pub started_at: Option<String>,
    /// Task completion date (ISO 8601 format)
    pub completed_at: Option<String>,
    /// Task metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Command execution task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    /// Task metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Command execution task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Basic task information
    pub task_info: TaskInfo,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: Option<String>,
    /// Standard error output
    pub stderr: Option<String>,
    /// Resource usage
    pub resource_usage: Option<ResourceUsage>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage time (milliseconds)
    pub cpu_time_ms: u64,
    /// Maximum memory usage (kilobytes)
    pub max_memory_kb: u64,
    /// Number of bytes read
    pub io_read_bytes: u64,
    /// Number of bytes written
    pub io_write_bytes: u64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Version information
    pub version: String,
    /// Uptime (seconds)
    pub uptime_seconds: u64,
}

/// Default timeout value (30 seconds)
fn default_timeout() -> u32 {
    30
}
