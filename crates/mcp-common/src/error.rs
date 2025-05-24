use thiserror::Error;
use std::fmt;
use serde_json::Value;
use tracing::{debug, error};

/// MCPセキュリティゲートウェイで使用される共通エラー型
#[derive(Error, Debug)]
pub enum McpError {
    /// 認証や認可に関するエラー
    #[error("認証エラー: {0}")]
    Auth(String),

    /// 無効なリクエストや入力に関するエラー
    #[error("無効なリクエスト: {0}")]
    InvalidRequest(String),

    /// リソースが見つからない場合のエラー
    #[error("リソースが見つかりません: {0}")]
    NotFound(String),

    /// ポリシー評価に関するエラー
    #[error("ポリシー違反: {0}")]
    PolicyViolation(String),

    /// サンドボックス実行に関するエラー
    #[error("サンドボックスエラー: {0}")]
    Sandbox(String),

    /// コマンド実行に関するエラー
    #[error("実行エラー: {0}")]
    Execution(String),

    /// 内部的なエラー
    #[error("内部エラー: {0}")]
    Internal(String),

    /// 一時的なエラー（再試行可能）
    #[error("一時的なエラー: {0}")]
    Temporary(String),

    /// 外部サービスとの通信エラー
    #[error("外部サービスエラー: {0}")]
    ExternalService(String),
}

/// エラーコード範囲の定義
pub mod error_code {
    // 認証エラー (1000-1999)
    pub const AUTH_INVALID_CREDENTIALS: u32 = 1001;
    pub const AUTH_EXPIRED_TOKEN: u32 = 1002;
    pub const AUTH_INSUFFICIENT_PERMISSIONS: u32 = 1003;

    // 入力検証エラー (2000-2999)
    pub const INPUT_INVALID_PARAMETER: u32 = 2001;
    pub const INPUT_MISSING_REQUIRED: u32 = 2002;
    pub const INPUT_INVALID_FORMAT: u32 = 2003;

    // ポリシーエラー (3000-3999)
    pub const POLICY_COMMAND_NOT_ALLOWED: u32 = 3001;
    pub const POLICY_NETWORK_ACCESS_DENIED: u32 = 3002;
    pub const POLICY_FILE_ACCESS_DENIED: u32 = 3003;
    pub const POLICY_RESOURCE_LIMIT_EXCEEDED: u32 = 3004;

    // サンドボックスエラー (4000-4999)
    pub const SANDBOX_SETUP_FAILED: u32 = 4001;
    pub const SANDBOX_EXECUTION_FAILED: u32 = 4002;
    pub const SANDBOX_RESOURCE_LIMIT_EXCEEDED: u32 = 4003;

    // 内部エラー (5000-5999)
    pub const INTERNAL_UNEXPECTED: u32 = 5001;
    pub const INTERNAL_DATABASE_ERROR: u32 = 5002;
    pub const INTERNAL_DEPENDENCY_FAILED: u32 = 5003;

    // 新しいエラーコード
    pub const RESOURCE_NOT_FOUND: u32 = 6001;
}

/// エラー応答の標準形式
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// エラー詳細
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorDetail {
    pub code: u32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Result<T, E> からResult<T, McpError>への変換マクロ
/// 
/// # Examples
/// 
/// ```
/// use mcp_common::to_mcp_result;
/// use std::fs::File;
/// 
/// fn read_config() -> mcp_common::error::McpResult<String> {
///     let file = to_mcp_result!(File::open("config.json"), "設定ファイルが開けません");
///     // 以降の処理...
///     Ok("設定内容".to_string())
/// }
/// ```
#[macro_export]
macro_rules! to_mcp_result {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(mcp_common::error::McpError::Internal(
                format!("{}: {}", $msg, err)
            )),
        }
    };
    ($expr:expr, $err_type:ident, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(mcp_common::error::McpError::$err_type(
                format!("{}: {}", $msg, err)
            )),
        }
    };
}

impl McpError {
    /// エラーコードを取得
    pub fn code(&self) -> u32 {
        match self {
            McpError::Auth(msg) if msg.contains("invalid") => error_code::AUTH_INVALID_CREDENTIALS,
            McpError::Auth(msg) if msg.contains("expired") => error_code::AUTH_EXPIRED_TOKEN,
            McpError::Auth(_) => error_code::AUTH_INSUFFICIENT_PERMISSIONS,
            
            McpError::InvalidRequest(msg) if msg.contains("missing") => error_code::INPUT_MISSING_REQUIRED,
            McpError::InvalidRequest(msg) if msg.contains("format") => error_code::INPUT_INVALID_FORMAT,
            McpError::InvalidRequest(_) => error_code::INPUT_INVALID_PARAMETER,
            
            McpError::NotFound(_) => error_code::RESOURCE_NOT_FOUND,
            
            McpError::PolicyViolation(msg) if msg.contains("command") => error_code::POLICY_COMMAND_NOT_ALLOWED,
            McpError::PolicyViolation(msg) if msg.contains("network") => error_code::POLICY_NETWORK_ACCESS_DENIED,
            McpError::PolicyViolation(msg) if msg.contains("file") => error_code::POLICY_FILE_ACCESS_DENIED,
            McpError::PolicyViolation(msg) if msg.contains("resource") => error_code::POLICY_RESOURCE_LIMIT_EXCEEDED,
            McpError::PolicyViolation(_) => error_code::POLICY_COMMAND_NOT_ALLOWED,
            
            McpError::Sandbox(msg) if msg.contains("setup") => error_code::SANDBOX_SETUP_FAILED,
            McpError::Sandbox(msg) if msg.contains("resource") => error_code::SANDBOX_RESOURCE_LIMIT_EXCEEDED,
            McpError::Sandbox(_) => error_code::SANDBOX_EXECUTION_FAILED,
            
            McpError::Execution(_) => error_code::SANDBOX_EXECUTION_FAILED,
            
            McpError::Internal(msg) if msg.contains("database") => error_code::INTERNAL_DATABASE_ERROR,
            McpError::Internal(msg) if msg.contains("dependency") => error_code::INTERNAL_DEPENDENCY_FAILED,
            McpError::Internal(_) => error_code::INTERNAL_UNEXPECTED,
            
            McpError::Temporary(_) => error_code::INTERNAL_UNEXPECTED,
            McpError::ExternalService(_) => error_code::INTERNAL_DEPENDENCY_FAILED,
        }
    }

    /// エラー応答を生成
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.code(),
                message: self.to_string(),
                details: None,
            }
        }
    }

    /// 詳細情報付きのエラー応答を生成
    pub fn with_details(&self, details: Value) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.code(),
                message: self.to_string(),
                details: Some(details),
            }
        }
    }
    
    /// カスタムコード付きのポリシー違反エラーを生成
    pub fn policy_violation(message: impl Into<String>, _code: u32, details: Option<Value>) -> Self {
        // 基本的なエラーを作成
        let error = McpError::PolicyViolation(message.into());
        
        // 詳細情報をログに記録
        if let Some(details_json) = &details {
            debug!("Policy violation details: {}", details_json);
        }
        
        error
    }
    
    /// カスタムコード付きのサンドボックスエラーを生成
    pub fn sandbox_error(message: impl Into<String>, _code: u32, details: Option<Value>) -> Self {
        // 基本的なエラーを作成
        let error = McpError::Sandbox(message.into());
        
        // 詳細情報をログに記録
        if let Some(details_json) = &details {
            debug!("Sandbox error details: {}", details_json);
        }
        
        error
    }
    
    /// その他のMcpErrorからエラー応答を作成する補助関数
    pub fn from_error<E: fmt::Display>(err: E, _code: u32) -> Self {
        McpError::Internal(format!("{}", err))
    }
}

/// Result型のエイリアス
pub type McpResult<T> = std::result::Result<T, McpError>;

// standardエラーからMcpErrorへの変換実装
impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => McpError::NotFound(format!("ファイルが見つかりません: {}", err)),
            std::io::ErrorKind::PermissionDenied => McpError::PolicyViolation(format!("アクセス権限がありません: {}", err)),
            std::io::ErrorKind::ConnectionRefused => McpError::ExternalService(format!("接続が拒否されました: {}", err)),
            std::io::ErrorKind::ConnectionReset => McpError::Temporary(format!("接続がリセットされました: {}", err)),
            std::io::ErrorKind::ConnectionAborted => McpError::Temporary(format!("接続が中断されました: {}", err)),
            std::io::ErrorKind::NotConnected => McpError::ExternalService(format!("接続されていません: {}", err)),
            std::io::ErrorKind::TimedOut => McpError::Execution(format!("タイムアウトしました: {}", err)),
            _ => McpError::Internal(format!("I/Oエラー: {}", err)),
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::InvalidRequest(format!("JSONパースエラー: {}", err))
    }
}

impl From<std::str::Utf8Error> for McpError {
    fn from(err: std::str::Utf8Error) -> Self {
        McpError::InvalidRequest(format!("不正なUTF-8シーケンス: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for McpError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        McpError::InvalidRequest(format!("不正なUTF-8シーケンス: {}", err))
    }
}

/// トレース可能なエラーのラッパー
#[macro_export]
macro_rules! trace_err {
    ($expr:expr, $level:ident, $msg:expr) => {
        match $expr {
            Ok(val) => Ok(val),
            Err(err) => {
                tracing::$level!("{}: {}", $msg, err);
                Err(err)
            }
        }
    };
}

/// 早期リターン付きエラートレース
#[macro_export]
macro_rules! try_trace {
    ($expr:expr, $level:ident, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                tracing::$level!("{}: {}", $msg, err);
                return Err(err.into());
            }
        }
    };
}

/// 汎用的なResultをMcpResultに変換するヘルパー
pub trait IntoMcpResult<T, E> {
    /// Result<T, E>をMcpResult<T>に変換
    fn into_mcp_result(self) -> McpResult<T>;
    
    /// Result<T, E>をMcpResult<T>に変換し、カスタムメッセージを付ける
    fn into_mcp_result_with_msg(self, msg: impl Into<String>) -> McpResult<T>;
}

impl<T, E: std::fmt::Display> IntoMcpResult<T, E> for Result<T, E> {
    fn into_mcp_result(self) -> McpResult<T> {
        self.map_err(|e| McpError::Internal(format!("{}", e)))
    }
    
    fn into_mcp_result_with_msg(self, msg: impl Into<String>) -> McpResult<T> {
        self.map_err(|e| McpError::Internal(format!("{}: {}", msg.into(), e)))
    }
}

/// 様々なエラーをMcpErrorに変換するための変換トレイト
pub trait ToMcpError {
    /// McpErrorに変換
    fn to_mcp_error(self) -> McpError;
    
    /// カスタムメッセージつきでMcpErrorに変換
    fn to_mcp_error_with_msg(self, msg: impl Into<String>) -> McpError;
}

// ToMcpErrorの汎用実装（From<E>トレイトを実装している型のみに適用）
// 警告: この実装はmcp-errorのToMcpError自体には適用されません
// これにより、競合を避けつつ必要な変換を提供します
impl<E> ToMcpError for E 
where 
    E: std::fmt::Display,
    E: Into<McpError>,
    E: Sized,
    E: 'static,
{
    fn to_mcp_error(self) -> McpError {
        self.into()
    }
    
    fn to_mcp_error_with_msg(self, msg: impl Into<String>) -> McpError {
        let error = self.into();
        match error {
            McpError::Internal(e) => McpError::Internal(format!("{}: {}", msg.into(), e)),
            McpError::InvalidRequest(e) => McpError::InvalidRequest(format!("{}: {}", msg.into(), e)),
            McpError::PolicyViolation(e) => McpError::PolicyViolation(format!("{}: {}", msg.into(), e)),
            McpError::Auth(e) => McpError::Auth(format!("{}: {}", msg.into(), e)),
            McpError::NotFound(e) => McpError::NotFound(format!("{}: {}", msg.into(), e)),
            McpError::Sandbox(e) => McpError::Sandbox(format!("{}: {}", msg.into(), e)),
            McpError::Execution(e) => McpError::Execution(format!("{}: {}", msg.into(), e)),
            McpError::Temporary(e) => McpError::Temporary(format!("{}: {}", msg.into(), e)),
            McpError::ExternalService(e) => McpError::ExternalService(format!("{}: {}", msg.into(), e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_result_into_mcp_result() {
        let ok_result: Result<i32, &str> = Ok(42);
        let err_result: Result<i32, &str> = Err("エラー発生");
        
        assert_eq!(ok_result.into_mcp_result().unwrap(), 42);
        assert!(matches!(err_result.into_mcp_result(), Err(McpError::Internal(_))));
    }
    
    #[test]
    fn test_io_error_conversion() {
        let not_found = std::io::Error::new(std::io::ErrorKind::NotFound, "ファイルなし");
        let perm_denied = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "権限エラー");
        
        assert!(matches!(not_found.to_mcp_error(), McpError::NotFound(_)));
        assert!(matches!(perm_denied.to_mcp_error(), McpError::PolicyViolation(_)));
    }
} 