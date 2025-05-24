use crate::models::{ExecutionRequest, ExecutionResult, ResourceUsage, NetworkAccess};
use crate::bubblewrap::BubblewrapWrapper;
use crate::seccomp::{SeccompProfileManager, SeccompProfileType};
use mcp_common::error::{McpError, McpResult};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use tokio::process::Command;
use tokio::time::timeout;
use std::time::Duration;

/// Runner for executing commands in a sandbox
#[derive(Debug)]
pub struct SandboxRunner {
    bubblewrap: Option<BubblewrapWrapper>,
    seccomp_manager: SeccompProfileManager,
}

impl SandboxRunner {
    /// Create a new SandboxRunner
    pub fn new() -> Self {
        let bubblewrap = BubblewrapWrapper::new();
        let seccomp_manager = SeccompProfileManager::default();
        
        if bubblewrap.is_none() {
            warn!("bubblewrap is not available, executing without sandbox. This is a security vulnerability.");
        } else {
            info!("Using bubblewrap sandbox.");
        }
        
        Self {
            bubblewrap,
            seccomp_manager,
        }
    }

    /// Execute command in sandbox
    pub async fn run(&self, request: ExecutionRequest) -> McpResult<ExecutionResult> {
        let _start_time = Instant::now();
        debug!("Starting command execution: {} {:?}", request.command, request.args);

        // Determine whether to use sandbox
        let use_sandbox = request.sandbox_config.enabled && self.bubblewrap.is_some();
        
        if use_sandbox {
            info!("Executing in bubblewrap sandbox mode");
            self.execute_in_sandbox(&request).await
        } else {
            if request.sandbox_config.enabled {
                warn!("bubblewrap is disabled or not available, executing without sandbox!");
            } else {
                warn!("Sandbox is disabled! Executing in unsafe environment.");
            }
            self.execute_without_sandbox(&request).await
        }
    }

    /// Execute command in sandbox
    async fn execute_in_sandbox(&self, request: &ExecutionRequest) -> McpResult<ExecutionResult> {
        let bubblewrap = self.bubblewrap.as_ref().unwrap();
        let start_time = Instant::now();
        
        // Get seccomp profile
        let seccomp_profile = match &request.sandbox_config.network_access {
            NetworkAccess::None => self.seccomp_manager.get_profile_path(SeccompProfileType::Basic),
            _ => self.seccomp_manager.get_profile_path(SeccompProfileType::Network),
        };
        
        // Clone and modify sandbox configuration
        let mut sandbox_config = request.sandbox_config.clone();
        if seccomp_profile.is_ok() {
            sandbox_config.seccomp_profile = Some(seccomp_profile?);
        }
        
        // Build bubblewrap command
        let mut cmd = bubblewrap.build_command(
            &sandbox_config,
            &request.command,
            &request.args,
        );
        
        // Set environment variables
        for (key, value) in &request.env {
            cmd.env(key, value);
        }
        
        // Set working directory (must be a valid path within the sandbox)
        if let Some(cwd) = &request.cwd {
            cmd.env("PWD", cwd);
            // Note: current_dir doesn't work with bubblewrap, so we set the PWD environment variable
        }

        // Set timeout
        let timeout_duration = Duration::from_secs(request.timeout as u64);
        
        debug!("bubblewrap command: {:?}", cmd);
        
        // Execute command
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(result) => match result {
                Ok(output) => output,
                Err(e) => {
                    error!("bubblewrap command execution error: {}", e);
                    return Err(McpError::Execution(format!("Sandbox execution failed: {}", e)));
                }
            },
            Err(_) => {
                error!("bubblewrap command execution timed out: {} seconds", request.timeout);
                return Err(McpError::Execution(format!(
                    "Sandbox execution timed out: {} seconds",
                    request.timeout
                )));
            }
        };

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;
        
        // Measure resource usage (in the future, this will be obtained from cgroups, etc.)
        // Currently returns dummy values
        let resource_usage = ResourceUsage {
            cpu_time_ms: execution_time_ms,
            max_memory_kb: 0,
            io_read_bytes: 0,
            io_write_bytes: 0,
        };

        Ok(ExecutionResult {
            exit_code: Some(output.status.code().unwrap_or(-1)),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            resource_usage,
            execution_time_ms,
        })
    }

    /// Execute command without sandbox (reusing milestone 1 implementation)
    async fn execute_without_sandbox(&self, request: &ExecutionRequest) -> McpResult<ExecutionResult> {
        let start_time = Instant::now();
        let mut cmd = Command::new(&request.command);
        
        // Set arguments
        cmd.args(&request.args);
        
        // Set environment variables
        for (key, value) in &request.env {
            cmd.env(key, value);
        }
        
        // Set working directory
        if let Some(cwd) = &request.cwd {
            cmd.current_dir(cwd);
        }

        // Set timeout
        let timeout_duration = Duration::from_secs(request.timeout as u64);
        
        // Execute command
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(result) => match result {
                Ok(output) => output,
                Err(e) => {
                    error!("Command execution error: {}", e);
                    return Err(McpError::Execution(format!("Command execution failed: {}", e)));
                }
            },
            Err(_) => {
                error!("Command execution timed out: {} seconds", request.timeout);
                return Err(McpError::Execution(format!(
                    "Command execution timed out: {} seconds",
                    request.timeout
                )));
            }
        };

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;
        
        // Measure resource usage (to be implemented in milestone 2)
        // Currently returns dummy values
        let resource_usage = ResourceUsage {
            cpu_time_ms: execution_time_ms,
            max_memory_kb: 0,
            io_read_bytes: 0,
            io_write_bytes: 0,
        };

        Ok(ExecutionResult {
            exit_code: Some(output.status.code().unwrap_or(-1)),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            resource_usage,
            execution_time_ms,
        })
    }
}

impl Default for SandboxRunner {
    fn default() -> Self {
        Self::new()
    }
} 