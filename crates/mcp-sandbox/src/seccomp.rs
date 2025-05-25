use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, error};

/// Seccomp profile types
#[derive(Debug, Clone, Copy)]
pub enum SeccompProfileType {
    /// Basic profile (allows only file operations and basic process operations)
    Basic,
    /// Profile that allows internet access
    Network,
}

/// Seccomp profile management
#[derive(Debug)]
pub struct SeccompProfileManager {
    profile_dir: PathBuf,
}

impl SeccompProfileManager {
    /// Create a new SeccompProfileManager
    pub fn new(profile_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&profile_dir).unwrap_or_else(|e| {
            error!("Failed to create seccomp profile directory: {}", e);
        });

        Self { profile_dir }
    }
}

impl Default for SeccompProfileManager {
    fn default() -> Self {
        let tmp_dir = std::env::temp_dir().join("mcp-seccomp-profiles");
        Self::new(tmp_dir)
    }
}

impl SeccompProfileManager {
    /// Get the path to a seccomp profile
    pub fn get_profile_path(&self, profile_type: SeccompProfileType) -> McpResult<PathBuf> {
        let profile_name = match profile_type {
            SeccompProfileType::Basic => "basic.json",
            SeccompProfileType::Network => "network.json",
        };

        let profile_path = self.profile_dir.join(profile_name);

        if !profile_path.exists() {
            self.generate_profile(profile_type, &profile_path)?;
        }

        Ok(profile_path)
    }

    /// Generate a seccomp profile
    fn generate_profile(&self, profile_type: SeccompProfileType, path: &PathBuf) -> McpResult<()> {
        let profile_content = match profile_type {
            SeccompProfileType::Basic => include_str!("../profiles/basic.json"),
            SeccompProfileType::Network => include_str!("../profiles/network.json"),
        };

        let mut file = File::create(path).map_err(|e| {
            error!("Failed to create seccomp profile: {}", e);
            McpError::Internal(format!("Failed to create seccomp profile: {}", e))
        })?;

        file.write_all(profile_content.as_bytes()).map_err(|e| {
            error!("Failed to write seccomp profile: {}", e);
            McpError::Internal(format!("Failed to write seccomp profile: {}", e))
        })?;

        debug!("Generated seccomp profile: {:?}", path);

        Ok(())
    }
}

use mcp_common::error::{McpError, McpResult};
