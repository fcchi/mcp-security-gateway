use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy evaluation input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInput {
    /// User information
    #[serde(default)]
    pub user: UserInfo,
    /// Command execution information
    #[serde(default)]
    pub command: CommandInfo,
    /// File access information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileInfo>,
    /// Network access information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkInfo>,
    /// Resource limit information
    #[serde(default)]
    pub resources: ResourceLimits,
    /// Context information (additional metadata)
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserInfo {
    /// User ID
    pub id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// List of roles
    #[serde(default)]
    pub roles: Vec<String>,
    /// Additional attributes
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// Command execution information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandInfo {
    /// Command name
    pub name: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory
    #[serde(default)]
    pub cwd: String,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// File access information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path
    pub path: String,
    /// Access mode ("read", "write", "execute")
    pub mode: String,
}

/// Network access information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Destination host
    pub host: String,
    /// Destination port
    pub port: u16,
    /// Protocol ("tcp", "udp")
    pub protocol: String,
}

/// Resource limit information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceLimits {
    /// CPU time limit (milliseconds)
    #[serde(default)]
    pub cpu_time_ms: Option<u64>,
    /// Memory usage limit (kilobytes)
    #[serde(default)]
    pub memory_kb: Option<u64>,
    /// File count limit
    #[serde(default)]
    pub max_files: Option<u32>,
    /// Process count limit
    #[serde(default)]
    pub max_processes: Option<u32>,
}

/// Policy evaluation decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Whether the action is allowed
    pub allow: bool,
    /// Warning messages (allowed but with warnings)
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Denial reasons (if rejected)
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}
