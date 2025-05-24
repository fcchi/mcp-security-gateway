use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Command execution request
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory
    pub cwd: Option<PathBuf>,
    /// Timeout (seconds)
    pub timeout: u32,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error output
    pub stderr: String,
    /// Resource usage
    pub resource_usage: ResourceUsage,
    /// Execution time (milliseconds)
    pub execution_time_ms: u64,
}

/// Resource usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Whether sandbox is enabled
    pub enabled: bool,
    /// Path to seccomp profile
    pub seccomp_profile: Option<PathBuf>,
    /// Paths with read-write permission
    pub rw_paths: Vec<PathBuf>,
    /// Paths with read-only permission
    pub ro_paths: Vec<PathBuf>,
    /// Denied paths
    pub denied_paths: Vec<PathBuf>,
    /// Network access configuration
    pub network_access: NetworkAccess,
    /// Resource limits configuration
    pub resource_limits: ResourceLimits,
}

/// Network access configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAccess {
    /// No network access allowed
    None,
    /// Access to the same network as the host
    Host,
    /// Access only to specific hosts
    Restricted(Vec<String>),
}

/// Resource limits configuration
#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    /// CPU limit (cores)
    pub cpu_limit: Option<f64>,
    /// Memory limit (bytes)
    pub memory_limit: Option<u64>,
    /// Process count limit
    pub pids_limit: Option<u32>,
    /// IO weight (priority)
    pub io_weight: Option<u32>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seccomp_profile: None,
            rw_paths: vec![PathBuf::from("/workspace")],
            ro_paths: vec![
                PathBuf::from("/usr/bin"),
                PathBuf::from("/usr/lib"),
                PathBuf::from("/lib"),
            ],
            denied_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                PathBuf::from("/home"),
            ],
            network_access: NetworkAccess::None,
            resource_limits: ResourceLimits::default(),
        }
    }
} 