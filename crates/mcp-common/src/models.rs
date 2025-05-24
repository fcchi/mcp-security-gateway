use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// タスクのステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// タスクが作成され、まだ実行されていない
    Created,
    /// タスクが実行待ちキューに入っている
    Queued,
    /// タスクが実行中
    Running,
    /// タスクが正常に完了した
    Completed,
    /// タスクが失敗した
    Failed,
    /// タスクがキャンセルされた
    Cancelled,
    /// タスクの実行時間が制限を超えた
    TimedOut,
}

/// タスクのタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// コマンド実行タスク
    Command,
    /// ファイル操作タスク
    File,
    /// HTTPリクエストタスク
    HttpRequest,
}

/// タスクの基本情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// タスクのユニークID
    pub task_id: String,
    /// タスクのタイプ
    pub task_type: TaskType,
    /// タスクの現在のステータス
    pub status: TaskStatus,
    /// タスクの作成日時（ISO 8601形式）
    pub created_at: String,
    /// タスクの開始日時（ISO 8601形式）
    pub started_at: Option<String>,
    /// タスクの完了日時（ISO 8601形式）
    pub completed_at: Option<String>,
    /// タスクのメタデータ
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// コマンド実行タスクのリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// 実行するコマンド
    pub command: String,
    /// コマンドの引数
    #[serde(default)]
    pub args: Vec<String>,
    /// 環境変数
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// 作業ディレクトリ
    #[serde(default)]
    pub cwd: Option<String>,
    /// タイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    /// タスクのメタデータ
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// コマンド実行タスクの結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// タスクの基本情報
    pub task_info: TaskInfo,
    /// 終了コード
    pub exit_code: Option<i32>,
    /// 標準出力
    pub stdout: Option<String>,
    /// 標準エラー出力
    pub stderr: Option<String>,
    /// リソース使用量
    pub resource_usage: Option<ResourceUsage>,
}

/// リソース使用量の情報
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// ヘルスチェックの応答
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// サービスのステータス
    pub status: String,
    /// バージョン情報
    pub version: String,
    /// アップタイム（秒）
    pub uptime_seconds: u64,
}

/// デフォルトのタイムアウト値（30秒）
fn default_timeout() -> u32 {
    30
} 