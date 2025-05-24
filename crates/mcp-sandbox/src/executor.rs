use crate::models::{ExecutionRequest, ExecutionResult, SandboxConfig};
use crate::runner::SandboxRunner;
use mcp_common::error::{McpError, McpResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use std::fmt;

/// サンドボックス内でコマンドを実行するエグゼキュータ
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
    /// 新しいCommandExecutorを作成
    pub fn new() -> Self {
        Self {
            runner: Arc::new(SandboxRunner::new()),
            default_timeout: 30, // 30秒
            default_sandbox_config: SandboxConfig::default(),
        }
    }

    /// デフォルト設定を変更したExecutorを作成
    pub fn with_config(timeout: u32, sandbox_config: SandboxConfig) -> Self {
        Self {
            runner: Arc::new(SandboxRunner::new()),
            default_timeout: timeout,
            default_sandbox_config: sandbox_config,
        }
    }

    /// コマンドを実行
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
        
        info!("コマンド実行: {} {:?}", command, args);
        
        // コマンドが空の場合はエラー
        if command.is_empty() {
            return Err(McpError::InvalidRequest("コマンドが指定されていません".to_string()));
        }
        
        // タイムアウトが0の場合はエラー
        if timeout == 0 {
            return Err(McpError::InvalidRequest("タイムアウトは1秒以上で指定してください".to_string()));
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
    
    /// サンドボックス設定を更新したExecutorを作成
    pub fn with_sandbox_config(&self, config: SandboxConfig) -> Self {
        Self {
            runner: self.runner.clone(),
            default_timeout: self.default_timeout,
            default_sandbox_config: config,
        }
    }
    
    /// タイムアウト設定を更新したExecutorを作成
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