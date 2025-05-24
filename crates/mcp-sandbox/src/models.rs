use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// コマンド実行リクエスト
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    /// 実行するコマンド
    pub command: String,
    /// コマンド引数
    pub args: Vec<String>,
    /// 環境変数
    pub env: HashMap<String, String>,
    /// 作業ディレクトリ
    pub cwd: Option<PathBuf>,
    /// タイムアウト（秒）
    pub timeout: u32,
    /// サンドボックス設定
    pub sandbox_config: SandboxConfig,
}

/// コマンド実行結果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 終了コード
    pub exit_code: Option<i32>,
    /// 標準出力
    pub stdout: String,
    /// 標準エラー出力
    pub stderr: String,
    /// リソース使用状況
    pub resource_usage: ResourceUsage,
    /// 実行時間（ミリ秒）
    pub execution_time_ms: u64,
}

/// リソース使用状況
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU使用時間（ミリ秒）
    pub cpu_time_ms: u64,
    /// 最大メモリ使用量（キロバイト）
    pub max_memory_kb: u64,
    /// 読み込みバイト数
    pub io_read_bytes: u64,
    /// 書き込みバイト数
    pub io_write_bytes: u64,
}

/// サンドボックス設定
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// サンドボックスが有効かどうか
    pub enabled: bool,
    /// seccompプロファイルへのパス
    pub seccomp_profile: Option<PathBuf>,
    /// 読み書き許可パス
    pub rw_paths: Vec<PathBuf>,
    /// 読み取り専用許可パス
    pub ro_paths: Vec<PathBuf>,
    /// アクセス禁止パス
    pub denied_paths: Vec<PathBuf>,
    /// ネットワークアクセス設定
    pub network_access: NetworkAccess,
    /// リソース制限設定
    pub resource_limits: ResourceLimits,
}

/// ネットワークアクセス設定
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAccess {
    /// ネットワークへのアクセスを許可しない
    None,
    /// ホストと同じネットワークへのアクセスを許可
    Host,
    /// 特定のホストへのアクセスのみ許可
    Restricted(Vec<String>),
}

/// リソース制限設定
#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    /// CPU制限（コア数）
    pub cpu_limit: Option<f64>,
    /// メモリ制限（バイト）
    pub memory_limit: Option<u64>,
    /// プロセス数制限
    pub pids_limit: Option<u32>,
    /// IOウェイト（優先度）
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