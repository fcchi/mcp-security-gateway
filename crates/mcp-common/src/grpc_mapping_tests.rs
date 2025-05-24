//! エラーコードとgRPCステータスのマッピングテスト
//! 
//! このモジュールでは、McpErrorからtonicのStatusへの変換と、
//! エラーコードからStatusコードへの変換を包括的にテストします。

#[cfg(test)]
mod tests {
    use tonic::{Status, Code};
    use crate::error::{McpError, error_code, ErrorResponse, ErrorDetail};
    use crate::grpc::{IntoStatus, get_status_code_from_error_code, get_error_code_from_status};
    use std::collections::HashMap;
    use serde_json::json;

    /// マッピングテーブルを保存する構造体
    struct MappingMatrix {
        /// エラーコードとステータスコードのマッピング
        error_to_status: HashMap<u32, Code>,
        /// ステータスコードとエラーコードの逆マッピング
        status_to_error: HashMap<Code, Vec<u32>>,
    }

    impl MappingMatrix {
        /// マッピングマトリックスを初期化
        fn new() -> Self {
            let mut matrix = Self {
                error_to_status: HashMap::new(),
                status_to_error: HashMap::new(),
            };

            // 認証エラー
            matrix.add(error_code::AUTH_INVALID_CREDENTIALS, Code::Unauthenticated);
            matrix.add(error_code::AUTH_EXPIRED_TOKEN, Code::Unauthenticated);
            matrix.add(error_code::AUTH_INSUFFICIENT_PERMISSIONS, Code::PermissionDenied);
            
            // 入力検証エラー
            matrix.add(error_code::INPUT_INVALID_PARAMETER, Code::InvalidArgument);
            matrix.add(error_code::INPUT_MISSING_REQUIRED, Code::InvalidArgument);
            matrix.add(error_code::INPUT_INVALID_FORMAT, Code::InvalidArgument);
            
            // ポリシーエラー
            matrix.add(error_code::POLICY_COMMAND_NOT_ALLOWED, Code::PermissionDenied);
            matrix.add(error_code::POLICY_NETWORK_ACCESS_DENIED, Code::PermissionDenied);
            matrix.add(error_code::POLICY_FILE_ACCESS_DENIED, Code::PermissionDenied);
            matrix.add(error_code::POLICY_RESOURCE_LIMIT_EXCEEDED, Code::ResourceExhausted);
            
            // サンドボックスエラー
            matrix.add(error_code::SANDBOX_SETUP_FAILED, Code::Internal);
            matrix.add(error_code::SANDBOX_EXECUTION_FAILED, Code::Aborted);
            matrix.add(error_code::SANDBOX_RESOURCE_LIMIT_EXCEEDED, Code::ResourceExhausted);
            
            // 内部エラー
            matrix.add(error_code::INTERNAL_UNEXPECTED, Code::Internal);
            matrix.add(error_code::INTERNAL_DATABASE_ERROR, Code::Internal);
            matrix.add(error_code::INTERNAL_DEPENDENCY_FAILED, Code::Internal);
            
            // リソースエラー
            matrix.add(error_code::RESOURCE_NOT_FOUND, Code::NotFound);

            matrix
        }

        /// マッピングを追加
        fn add(&mut self, error_code: u32, status_code: Code) {
            self.error_to_status.insert(error_code, status_code);
            self.status_to_error.entry(status_code)
                .or_insert_with(Vec::new)
                .push(error_code);
        }

        /// エラーコードに対応するステータスコードを取得
        fn get_status(&self, error_code: u32) -> Code {
            *self.error_to_status.get(&error_code).unwrap_or(&Code::Unknown)
        }

        /// ステータスコードに対応する可能性のあるエラーコードを取得
        fn get_errors(&self, status_code: Code) -> Vec<u32> {
            self.status_to_error.get(&status_code)
                .cloned()
                .unwrap_or_default()
        }
    }

    #[test]
    fn test_all_error_codes_have_status_mapping() {
        // マッピングマトリックスを初期化
        let matrix = MappingMatrix::new();
        
        // すべてのエラーコードが実装済みであることを確認
        for error_code in get_all_error_codes() {
            let status_code = get_status_code_from_error_code(error_code);
            assert_eq!(status_code, matrix.get_status(error_code), 
                "エラーコード {} のステータスマッピングが期待値と異なります", error_code);
        }
    }

    #[test]
    fn test_all_status_codes_map_to_sensible_error_code() {
        // マッピングマトリックスを初期化
        let matrix = MappingMatrix::new();
        
        // よく使われるステータスコード
        let status_codes = vec![
            Code::Ok,
            Code::Cancelled,
            Code::Unknown,
            Code::InvalidArgument,
            Code::DeadlineExceeded,
            Code::NotFound,
            Code::AlreadyExists,
            Code::PermissionDenied,
            Code::ResourceExhausted,
            Code::FailedPrecondition,
            Code::Aborted,
            Code::OutOfRange,
            Code::Unimplemented,
            Code::Internal,
            Code::Unavailable,
            Code::DataLoss,
            Code::Unauthenticated,
        ];
        
        for status_code in status_codes {
            let status = Status::new(status_code, "Test error");
            let error_code = get_error_code_from_status(&status);
            assert!(error_code > 0, "ステータスコード {:?} のエラーマッピングが存在しません", status_code);
            
            // 逆マッピングの整合性チェック
            let expected_errors = matrix.get_errors(status_code);
            if !expected_errors.is_empty() {
                assert!(expected_errors.contains(&error_code) || 
                    get_status_code_from_error_code(error_code) == status_code, 
                    "ステータスコード {:?} のエラーマッピングが期待値と異なります", status_code);
            }
        }
    }

    #[test]
    fn test_mcp_error_to_status_mapping() {
        // すべてのMcpErrorタイプをテスト
        let errors = vec![
            (McpError::Auth("認証エラー".to_string()), Code::Unauthenticated),
            (McpError::InvalidRequest("無効なリクエスト".to_string()), Code::InvalidArgument),
            (McpError::NotFound("リソースなし".to_string()), Code::NotFound),
            (McpError::PolicyViolation("ポリシー違反".to_string()), Code::PermissionDenied),
            (McpError::Sandbox("サンドボックスエラー".to_string()), Code::FailedPrecondition),
            (McpError::Execution("実行エラー".to_string()), Code::Internal),
            (McpError::Internal("内部エラー".to_string()), Code::Internal),
            (McpError::Temporary("一時エラー".to_string()), Code::Unavailable),
            (McpError::ExternalService("外部サービスエラー".to_string()), Code::Unavailable),
        ];
        
        for (error, expected_code) in errors {
            // エラー情報をフォーマット文字列として事前に取得
            let error_debug = format!("{:?}", error);
            // ステータスを取得
            let status = error.into_status();
            // 期待値と比較
            assert_eq!(status.code(), expected_code, 
                "エラー {} のステータスコードが期待値と異なります", error_debug);
        }
    }

    #[test]
    fn test_error_response_serialization() {
        // エラー応答の生成と検証
        let error = McpError::PolicyViolation("コマンド実行が許可されていません".to_string());
        let error_code = error.code();
        let response = error.to_response();
        
        assert_eq!(response.error.code, error_code);
        assert!(response.error.message.contains("ポリシー違反"));
        assert!(response.error.details.is_none());
        
        // 詳細情報付きのエラー応答
        let details = json!({
            "command": "sudo",
            "reason": "privileged execution not allowed"
        });
        
        let detailed_response = error.with_details(details.clone());
        assert_eq!(detailed_response.error.code, error_code);
        assert!(detailed_response.error.message.contains("ポリシー違反"));
        assert_eq!(detailed_response.error.details.unwrap(), details);
    }

    #[test]
    fn test_error_metadata_in_status() {
        // エラーステータスのメタデータ
        let error = McpError::PolicyViolation("コマンド実行が許可されていません".to_string());
        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: error.code(),
                message: error.to_string(),
                details: Some(json!({ "command": "rm", "reason": "dangerous command" })),
            }
        };
        
        // シリアライズ
        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("dangerous command"));
        
        // デシリアライズ
        let parsed: ErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.error.code, error.code());
        assert!(parsed.error.message.contains("ポリシー違反"));
        assert!(parsed.error.details.is_some());
    }

    // すべての定義済みエラーコードを取得
    fn get_all_error_codes() -> Vec<u32> {
        vec![
            // 認証エラー
            error_code::AUTH_INVALID_CREDENTIALS,
            error_code::AUTH_EXPIRED_TOKEN,
            error_code::AUTH_INSUFFICIENT_PERMISSIONS,
            
            // 入力検証エラー
            error_code::INPUT_INVALID_PARAMETER,
            error_code::INPUT_MISSING_REQUIRED,
            error_code::INPUT_INVALID_FORMAT,
            
            // ポリシーエラー
            error_code::POLICY_COMMAND_NOT_ALLOWED,
            error_code::POLICY_NETWORK_ACCESS_DENIED,
            error_code::POLICY_FILE_ACCESS_DENIED,
            error_code::POLICY_RESOURCE_LIMIT_EXCEEDED,
            
            // サンドボックスエラー
            error_code::SANDBOX_SETUP_FAILED,
            error_code::SANDBOX_EXECUTION_FAILED,
            error_code::SANDBOX_RESOURCE_LIMIT_EXCEEDED,
            
            // 内部エラー
            error_code::INTERNAL_UNEXPECTED,
            error_code::INTERNAL_DATABASE_ERROR,
            error_code::INTERNAL_DEPENDENCY_FAILED,
            
            // リソースエラー
            error_code::RESOURCE_NOT_FOUND,
        ]
    }
} 