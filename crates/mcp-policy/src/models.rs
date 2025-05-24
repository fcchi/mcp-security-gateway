use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// ポリシー評価の入力データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInput {
    /// ユーザー情報
    #[serde(default)]
    pub user: UserInfo,
    /// 実行コマンド情報
    #[serde(default)]
    pub command: CommandInfo,
    /// ファイルアクセス情報
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileInfo>,
    /// ネットワークアクセス情報
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkInfo>,
    /// リソース制限情報
    #[serde(default)]
    pub resources: ResourceLimits,
    /// コンテキスト情報（追加のメタデータ）
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,
}

/// ユーザー情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserInfo {
    /// ユーザーID
    pub id: String,
    /// テナントID
    pub tenant_id: String,
    /// ロール一覧
    #[serde(default)]
    pub roles: Vec<String>,
    /// 追加の属性
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// コマンド実行情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandInfo {
    /// コマンド名
    pub name: String,
    /// コマンド引数
    #[serde(default)]
    pub args: Vec<String>,
    /// 作業ディレクトリ
    #[serde(default)]
    pub cwd: String,
    /// 環境変数
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// ファイルアクセス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// ファイルパス
    pub path: String,
    /// アクセスモード（"read", "write", "execute"）
    pub mode: String,
}

/// ネットワークアクセス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// 宛先ホスト
    pub host: String,
    /// 宛先ポート
    pub port: u16,
    /// プロトコル（"tcp", "udp"）
    pub protocol: String,
}

/// リソース制限情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceLimits {
    /// CPU時間の制限（ミリ秒）
    #[serde(default)]
    pub cpu_time_ms: Option<u64>,
    /// メモリ使用量の制限（キロバイト）
    #[serde(default)]
    pub memory_kb: Option<u64>,
    /// ファイル数の制限
    #[serde(default)]
    pub max_files: Option<u32>,
    /// プロセス数の制限
    #[serde(default)]
    pub max_processes: Option<u32>,
}

/// ポリシー評価の決定結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// 許可されるかどうか
    pub allow: bool,
    /// 警告メッセージ（許可されるが警告あり）
    #[serde(default)]
    pub warnings: Vec<String>,
    /// 拒否の理由（拒否された場合）
    #[serde(default)]
    pub reasons: Vec<String>,
    /// 追加のメタデータ
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
} 