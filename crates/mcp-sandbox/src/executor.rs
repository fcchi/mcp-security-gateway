use crate::models::{ExecutionRequest, ExecutionResult, SandboxConfig};
use crate::runner::SandboxRunner;
use mcp_common::error::{McpError, McpResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use std::fmt;

/// Executor for running commands in a sandbox
#[derive(Clone)]
pub struct CommandExecutor {
    runner: Arc<SandboxRunner>,
    default_timeout: u32,
    default_sandbox_config: SandboxConfig,
}

impl fmt::Debug for CommandExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandExecutor")
            .field("default_timeout", &self.default_timeout)
            .field("default_sandbox_config", &self.default_sandbox_config)
            .finish()
    }
}

impl CommandExecutor {
    /// Create a new CommandExecutor
    pub fn new() -> Self {
        Self {
            runner: Arc::new(SandboxRunner::new()),
            default_timeout: 30, // 30 seconds
            default_sandbox_config: SandboxConfig::default(),
        }
    }

    /// Create an Executor with modified default settings
    pub fn with_config(timeout: u32, sandbox_config: SandboxConfig) -> Self {
        Self {
            runner: Arc::new(SandboxRunner::new()),
            default_timeout: timeout,
            default_sandbox_config: sandbox_config,
        }
    }

    /// Execute a command
    pub async fn execute(
        &self,
        command: &str,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<String>,
        timeout: Option<u32>,
    ) -> McpResult<ExecutionResult> {
        let timeout = timeout.unwrap_or(self.default_timeout);
        
        let cwd = cwd.map(PathBuf::from);
        
        info!("Executing command: {} {:?}", command, args);
        
        // Error if command is empty
        if command.is_empty() {
            return Err(McpError::InvalidRequest("Command is not specified".to_string()));
        }
        
        // Error if timeout is 0
        if timeout == 0 {
            return Err(McpError::InvalidRequest("Timeout must be at least 1 second".to_string()));
        }
        
        let request = ExecutionRequest {
            command: command.to_string(),
            args,
            env,
            cwd,
            timeout,
            sandbox_config: self.default_sandbox_config.clone(),
        };
        
        self.runner.run(request).await
    }
    
    /// Create an Executor with updated sandbox configuration
    pub fn with_sandbox_config(&self, config: SandboxConfig) -> Self {
        Self {
            runner: self.runner.clone(),
            default_timeout: self.default_timeout,
            default_sandbox_config: config,
        }
    }
    
    /// Create an Executor with updated timeout setting
    pub fn with_timeout(&self, timeout: u32) -> Self {
        Self {
            runner: self.runner.clone(),
            default_timeout: timeout,
            default_sandbox_config: self.default_sandbox_config.clone(),
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
} 