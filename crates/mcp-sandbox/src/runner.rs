use crate::models::{ExecutionRequest, ExecutionResult, ResourceUsage, NetworkAccess};
use crate::bubblewrap::BubblewrapWrapper;
use crate::seccomp::{SeccompProfileManager, SeccompProfileType};
use mcp_common::error::{McpError, McpResult};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use tokio::process::Command;
use tokio::time::timeout;
use std::time::Duration;

/// サンドボックス内でコマンドを実行するランナー
#[derive(Debug)]
pub struct SandboxRunner {
    bubblewrap: Option<BubblewrapWrapper>,
    seccomp_manager: SeccompProfileManager,
}

impl SandboxRunner {
    /// 新しいSandboxRunnerを作成
    pub fn new() -> Self {
        let bubblewrap = BubblewrapWrapper::new();
        let seccomp_manager = SeccompProfileManager::default();
        
        if bubblewrap.is_none() {
            warn!("bubblewrapが利用できないため、サンドボックスなしで実行します。セキュリティ的に脆弱です。");
        } else {
            info!("bubblewrapサンドボックスを使用します。");
        }
        
        Self {
            bubblewrap,
            seccomp_manager,
        }
    }

    /// サンドボックス内でコマンドを実行
    pub async fn run(&self, request: ExecutionRequest) -> McpResult<ExecutionResult> {
        let _start_time = Instant::now();
        debug!("コマンド実行開始: {} {:?}", request.command, request.args);

        // サンドボックスを使用するかどうか判断
        let use_sandbox = request.sandbox_config.enabled && self.bubblewrap.is_some();
        
        if use_sandbox {
            info!("bubblewrapサンドボックスモードで実行します");
            self.execute_in_sandbox(&request).await
        } else {
            if request.sandbox_config.enabled {
                warn!("bubblewrapが無効化または利用できないため、サンドボックスなしで実行します！");
            } else {
                warn!("サンドボックスが無効化されています！安全でない環境で実行します。");
            }
            self.execute_without_sandbox(&request).await
        }
    }

    /// サンドボックス内でコマンドを実行
    async fn execute_in_sandbox(&self, request: &ExecutionRequest) -> McpResult<ExecutionResult> {
        let bubblewrap = self.bubblewrap.as_ref().unwrap();
        let start_time = Instant::now();
        
        // seccompプロファイルを取得
        let seccomp_profile = match &request.sandbox_config.network_access {
            NetworkAccess::None => self.seccomp_manager.get_profile_path(SeccompProfileType::Basic),
            _ => self.seccomp_manager.get_profile_path(SeccompProfileType::Network),
        };
        
        // サンドボックス設定を複製して修正
        let mut sandbox_config = request.sandbox_config.clone();
        if seccomp_profile.is_ok() {
            sandbox_config.seccomp_profile = Some(seccomp_profile?);
        }
        
        // bubblewrapコマンドを構築
        let mut cmd = bubblewrap.build_command(
            &sandbox_config,
            &request.command,
            &request.args,
        );
        
        // 環境変数を設定
        for (key, value) in &request.env {
            cmd.env(key, value);
        }
        
        // 作業ディレクトリを設定（ただし、サンドボックス内で有効なパスである必要がある）
        if let Some(cwd) = &request.cwd {
            cmd.env("PWD", cwd);
            // 注: bubblewrapではcurrent_dirは機能しないため、環境変数PWDを設定
        }

        // タイムアウトを設定
        let timeout_duration = Duration::from_secs(request.timeout as u64);
        
        debug!("bubblewrapコマンド: {:?}", cmd);
        
        // コマンドを実行
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(result) => match result {
                Ok(output) => output,
                Err(e) => {
                    error!("bubblewrapコマンド実行エラー: {}", e);
                    return Err(McpError::Execution(format!("サンドボックス実行に失敗しました: {}", e)));
                }
            },
            Err(_) => {
                error!("bubblewrapコマンド実行がタイムアウトしました: {}秒", request.timeout);
                return Err(McpError::Execution(format!(
                    "サンドボックス実行がタイムアウトしました: {}秒",
                    request.timeout
                )));
            }
        };

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;
        
        // リソース使用状況を計測（将来的にはcgroupsなどから取得）
        // 現在はダミーの値を返す
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

    /// サンドボックスなしでコマンドを実行（マイルストーン1の実装を再利用）
    async fn execute_without_sandbox(&self, request: &ExecutionRequest) -> McpResult<ExecutionResult> {
        let start_time = Instant::now();
        let mut cmd = Command::new(&request.command);
        
        // 引数を設定
        cmd.args(&request.args);
        
        // 環境変数を設定
        for (key, value) in &request.env {
            cmd.env(key, value);
        }
        
        // 作業ディレクトリを設定
        if let Some(cwd) = &request.cwd {
            cmd.current_dir(cwd);
        }

        // タイムアウトを設定
        let timeout_duration = Duration::from_secs(request.timeout as u64);
        
        // コマンドを実行
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(result) => match result {
                Ok(output) => output,
                Err(e) => {
                    error!("コマンド実行エラー: {}", e);
                    return Err(McpError::Execution(format!("コマンド実行に失敗しました: {}", e)));
                }
            },
            Err(_) => {
                error!("コマンド実行がタイムアウトしました: {}秒", request.timeout);
                return Err(McpError::Execution(format!(
                    "コマンド実行がタイムアウトしました: {}秒",
                    request.timeout
                )));
            }
        };

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;
        
        // リソース使用状況を計測（マイルストーン2で実装予定）
        // 現在はダミーの値を返す
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