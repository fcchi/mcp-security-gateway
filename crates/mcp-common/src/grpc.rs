//! gRPCステータスとエラーコードのマッピング

use crate::error::{error_code, McpError};
use tonic::{Code, Status};

/// McpErrorをtonicのStatusに変換するトレイト
pub trait IntoStatus {
    /// エラーをgRPC Statusに変換
    fn into_status(self) -> Status;
}

impl IntoStatus for McpError {
    fn into_status(self) -> Status {
        match self {
            McpError::Auth(_) => Status::new(Code::Unauthenticated, self.to_string()),
            McpError::InvalidRequest(_) => Status::new(Code::InvalidArgument, self.to_string()),
            McpError::NotFound(_) => Status::new(Code::NotFound, self.to_string()),
            McpError::PolicyViolation(_) => Status::new(Code::PermissionDenied, self.to_string()),
            McpError::Sandbox(_) => Status::new(Code::FailedPrecondition, self.to_string()),
            McpError::Execution(_) => Status::new(Code::Internal, self.to_string()),
            McpError::Internal(_) => Status::new(Code::Internal, self.to_string()),
            McpError::Temporary(_) => Status::new(Code::Unavailable, self.to_string()),
            McpError::ExternalService(_) => Status::new(Code::Unavailable, self.to_string()),
        }
    }
}

// From<McpError> for Statusの実装を追加
// これにより、?演算子がMcpError -> Statusの自動変換を行えるようになります
impl From<McpError> for Status {
    fn from(error: McpError) -> Self {
        error.into_status()
    }
}

/// エラーコードとStatusコードの詳細マッピング
pub fn get_status_code_from_error_code(error_code: u32) -> Code {
    match error_code {
        // 認証エラー
        error_code::AUTH_INVALID_CREDENTIALS => Code::Unauthenticated,
        error_code::AUTH_EXPIRED_TOKEN => Code::Unauthenticated,
        error_code::AUTH_INSUFFICIENT_PERMISSIONS => Code::PermissionDenied,

        // 入力検証エラー
        error_code::INPUT_INVALID_PARAMETER => Code::InvalidArgument,
        error_code::INPUT_MISSING_REQUIRED => Code::InvalidArgument,
        error_code::INPUT_INVALID_FORMAT => Code::InvalidArgument,

        // ポリシーエラー
        error_code::POLICY_COMMAND_NOT_ALLOWED => Code::PermissionDenied,
        error_code::POLICY_NETWORK_ACCESS_DENIED => Code::PermissionDenied,
        error_code::POLICY_FILE_ACCESS_DENIED => Code::PermissionDenied,
        error_code::POLICY_RESOURCE_LIMIT_EXCEEDED => Code::ResourceExhausted,

        // サンドボックスエラー
        error_code::SANDBOX_SETUP_FAILED => Code::Internal,
        error_code::SANDBOX_EXECUTION_FAILED => Code::Aborted,
        error_code::SANDBOX_RESOURCE_LIMIT_EXCEEDED => Code::ResourceExhausted,

        // 内部エラー
        error_code::INTERNAL_UNEXPECTED => Code::Internal,
        error_code::INTERNAL_DATABASE_ERROR => Code::Internal,
        error_code::INTERNAL_DEPENDENCY_FAILED => Code::Internal,

        // リソースエラー
        error_code::RESOURCE_NOT_FOUND => Code::NotFound,

        // その他のエラー
        _ => Code::Unknown,
    }
}

/// Statusからエラーコードへの変換（近似値を返す）
pub fn get_error_code_from_status(status: &Status) -> u32 {
    match status.code() {
        Code::Unauthenticated => error_code::AUTH_INVALID_CREDENTIALS,
        Code::PermissionDenied => error_code::AUTH_INSUFFICIENT_PERMISSIONS,
        Code::InvalidArgument => error_code::INPUT_INVALID_PARAMETER,
        Code::ResourceExhausted => error_code::POLICY_RESOURCE_LIMIT_EXCEEDED,
        Code::Aborted => error_code::SANDBOX_EXECUTION_FAILED,
        Code::Internal => error_code::INTERNAL_UNEXPECTED,
        Code::Unavailable => error_code::INTERNAL_DEPENDENCY_FAILED,
        Code::NotFound => error_code::RESOURCE_NOT_FOUND,
        _ => error_code::INTERNAL_UNEXPECTED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_status_conversion() {
        // 認証エラー
        let auth_error = McpError::Auth("認証失敗".to_string());
        assert_eq!(auth_error.into_status().code(), Code::Unauthenticated);

        // 入力エラー
        let input_error = McpError::InvalidRequest("無効なパラメータ".to_string());
        assert_eq!(input_error.into_status().code(), Code::InvalidArgument);

        // ポリシーエラー
        let policy_error = McpError::PolicyViolation("アクセス拒否".to_string());
        assert_eq!(policy_error.into_status().code(), Code::PermissionDenied);
    }

    #[test]
    fn test_error_code_mapping() {
        // 認証エラー
        assert_eq!(
            get_status_code_from_error_code(error_code::AUTH_INVALID_CREDENTIALS),
            Code::Unauthenticated
        );
        assert_eq!(
            get_status_code_from_error_code(error_code::AUTH_INSUFFICIENT_PERMISSIONS),
            Code::PermissionDenied
        );

        // 入力エラー
        assert_eq!(
            get_status_code_from_error_code(error_code::INPUT_INVALID_PARAMETER),
            Code::InvalidArgument
        );

        // ポリシーエラー
        assert_eq!(
            get_status_code_from_error_code(error_code::POLICY_COMMAND_NOT_ALLOWED),
            Code::PermissionDenied
        );

        // リソース
        assert_eq!(
            get_status_code_from_error_code(error_code::POLICY_RESOURCE_LIMIT_EXCEEDED),
            Code::ResourceExhausted
        );
    }

    #[test]
    fn test_bidirectional_mapping() {
        let original_code = error_code::POLICY_COMMAND_NOT_ALLOWED;
        let status_code = get_status_code_from_error_code(original_code);
        let status = Status::new(status_code, "テスト");
        let recovered_code = get_error_code_from_status(&status);

        // 完全に双方向ではないが、カテゴリは保持される
        assert_eq!(get_status_code_from_error_code(recovered_code), status_code);
    }

    #[test]
    fn test_from_impl() {
        // McpErrorからStatusへの自動変換
        let error: Status = McpError::Internal("テストエラー".to_string()).into();
        assert_eq!(error.code(), Code::Internal);
        assert!(error.message().contains("テストエラー"));
    }
}
