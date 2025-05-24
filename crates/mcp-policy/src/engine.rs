use crate::models::{PolicyDecision, PolicyInput};
use mcp_common::error::{McpError, McpResult, error_code};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};
use std::fmt;

/// ポリシー評価インターフェイス
pub trait PolicyEvaluator: Send + Sync {
    /// ポリシーを評価し、決定結果を返す
    fn evaluate(&self, input: &PolicyInput) -> McpResult<PolicyDecision>;
}

/// ポリシーエンジン
#[derive(Clone)]
pub struct PolicyEngine {
    evaluator: Arc<dyn PolicyEvaluator>,
}

impl fmt::Debug for PolicyEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyEngine").finish()
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEngine {
    /// 新しいポリシーエンジンを作成（デフォルトのStubPolicyEvaluatorを使用）
    pub fn new() -> Self {
        Self::with_evaluator(StubPolicyEvaluator::default())
    }
    
    /// 指定したポリシー評価器で新しいポリシーエンジンを作成
    pub fn with_evaluator(evaluator: impl PolicyEvaluator + 'static) -> Self {
        Self {
            evaluator: Arc::new(evaluator),
        }
    }

    /// コマンド実行を許可するかどうかを評価
    pub fn check_command_execution(&self, input: &PolicyInput) -> McpResult<()> {
        debug!("ポリシー評価: コマンド実行 command={}", input.command.name);
        
        let decision = self.evaluator.evaluate(input)?;
        
        if !decision.allow {
            let reason = decision.reasons.join(", ");
            let message = if reason.is_empty() {
                format!("コマンド '{}' の実行はポリシーにより拒否されました", input.command.name)
            } else {
                format!("コマンド '{}' の実行はポリシーにより拒否されました: {}", input.command.name, reason)
            };
            
            error!("ポリシー違反: {}", message);
            
            // 詳細情報を含めてエラーを返す
            let details = json!({
                "command": input.command.name,
                "reasons": decision.reasons,
                "user_id": input.user.id,
                "tenant_id": input.user.tenant_id
            });
            
            return Err(policy_violation(
                error_code::POLICY_COMMAND_NOT_ALLOWED,
                message,
                Some(details)
            ));
        }
        
        // 警告があれば記録
        if !decision.warnings.is_empty() {
            info!(
                "ポリシー警告: コマンド '{}' は許可されましたが、警告があります: {}",
                input.command.name,
                decision.warnings.join(", ")
            );
        }
        
        Ok(())
    }

    /// ファイルアクセスを許可するかどうかを評価
    pub fn check_file_access(&self, input: &PolicyInput) -> McpResult<()> {
        if let Some(file_info) = &input.file {
            debug!("ポリシー評価: ファイルアクセス path={}, mode={}", file_info.path, file_info.mode);
            
            let decision = self.evaluator.evaluate(input)?;
            
            if !decision.allow {
                let reason = decision.reasons.join(", ");
                let message = if reason.is_empty() {
                    format!("ファイル '{}' への {} アクセスはポリシーにより拒否されました", 
                        file_info.path, file_info.mode)
                } else {
                    format!("ファイル '{}' への {} アクセスはポリシーにより拒否されました: {}", 
                        file_info.path, file_info.mode, reason)
                };
                
                error!("ポリシー違反: {}", message);
                
                let details = json!({
                    "path": file_info.path,
                    "mode": file_info.mode,
                    "reasons": decision.reasons,
                    "user_id": input.user.id
                });
                
                return Err(policy_violation(
                    error_code::POLICY_FILE_ACCESS_DENIED,
                    message,
                    Some(details)
                ));
            }
            
            if !decision.warnings.is_empty() {
                info!(
                    "ポリシー警告: ファイル '{}' への {} アクセスは許可されましたが、警告があります: {}",
                    file_info.path, file_info.mode, decision.warnings.join(", ")
                );
            }
        }
        
        Ok(())
    }

    /// ネットワークアクセスを許可するかどうかを評価
    pub fn check_network_access(&self, input: &PolicyInput) -> McpResult<()> {
        if let Some(network_info) = &input.network {
            debug!("ポリシー評価: ネットワークアクセス host={}:{}, protocol={}", 
                network_info.host, network_info.port, network_info.protocol);
            
            let decision = self.evaluator.evaluate(input)?;
            
            if !decision.allow {
                let reason = decision.reasons.join(", ");
                let message = if reason.is_empty() {
                    format!("ホスト '{}:{}' への {} アクセスはポリシーにより拒否されました", 
                        network_info.host, network_info.port, network_info.protocol)
                } else {
                    format!("ホスト '{}:{}' への {} アクセスはポリシーにより拒否されました: {}", 
                        network_info.host, network_info.port, network_info.protocol, reason)
                };
                
                error!("ポリシー違反: {}", message);
                
                let details = json!({
                    "host": network_info.host,
                    "port": network_info.port,
                    "protocol": network_info.protocol,
                    "reasons": decision.reasons,
                    "user_id": input.user.id
                });
                
                return Err(policy_violation(
                    error_code::POLICY_NETWORK_ACCESS_DENIED,
                    message,
                    Some(details)
                ));
            }
            
            if !decision.warnings.is_empty() {
                info!(
                    "ポリシー警告: ホスト '{}:{}' への {} アクセスは許可されましたが、警告があります: {}",
                    network_info.host, network_info.port, network_info.protocol, decision.warnings.join(", ")
                );
            }
        }
        
        Ok(())
    }
}

/// ヘルパー関数：ポリシー違反エラーを生成
pub fn policy_violation(_code: u32, message: String, _details: Option<serde_json::Value>) -> McpError {
    McpError::PolicyViolation(message)
}

/// OPAポリシー評価器
#[allow(dead_code)]
pub struct OpaEvaluator {
    // OPAのポリシーモジュール（スタブ実装）
    query_path: String,
}

impl OpaEvaluator {
    /// 新しいOPAポリシー評価器を作成
    pub fn new(_wasm_policy_bytes: &[u8], query_path: &str) -> McpResult<Self> {
        // 注：opa-wasmのAPIは変更されている可能性があります。実際のAPI仕様に合わせて修正が必要です。
        // このスタブ実装では、APIの詳細を抽象化します。
        
        Ok(Self {
            query_path: query_path.to_string(),
        })
    }
}

impl PolicyEvaluator for OpaEvaluator {
    fn evaluate(&self, input: &PolicyInput) -> McpResult<PolicyDecision> {
        // 入力をJSON形式に変換
        let _input_json = serde_json::to_value(input)
            .map_err(|e| McpError::Internal(format!("入力のシリアライズに失敗しました: {}", e)))?;
        
        // 注：実際のOPA評価はここに実装する必要があります
        // この実装はスタブです
        
        // スタブの結果を返す
        Ok(PolicyDecision {
            allow: true,
            warnings: vec!["OPA WASM評価はスタブ実装です。".to_string()],
            reasons: vec![],
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// OPAの結果をPolicyDecisionに変換するヘルパー関数
#[allow(dead_code)]
fn parse_opa_result(result: serde_json::Value) -> McpResult<PolicyDecision> {
    // 基本的なフォールバック値
    let mut decision = PolicyDecision {
        allow: false,
        warnings: Vec::new(),
        reasons: Vec::new(),
        metadata: std::collections::HashMap::new(),
    };
    
    // 結果がオブジェクトでない場合
    if !result.is_object() {
        return Ok(decision);
    }
    
    let result_obj = result.as_object().unwrap();
    
    // allowフィールドを解析
    if let Some(allow) = result_obj.get("allow") {
        if let Some(allow_bool) = allow.as_bool() {
            decision.allow = allow_bool;
        }
    }
    
    // warningsフィールドを解析
    if let Some(warnings) = result_obj.get("warnings") {
        if let Some(warnings_arr) = warnings.as_array() {
            decision.warnings = warnings_arr
                .iter()
                .filter_map(|w| w.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    
    // reasonsフィールドを解析
    if let Some(reasons) = result_obj.get("reasons") {
        if let Some(reasons_arr) = reasons.as_array() {
            decision.reasons = reasons_arr
                .iter()
                .filter_map(|r| r.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    
    // 追加のメタデータを解析（将来の拡張用）
    if let Some(metadata) = result_obj.get("metadata") {
        if let Some(metadata_obj) = metadata.as_object() {
            for (key, value) in metadata_obj {
                if let Some(value_str) = value.as_str() {
                    // Value::Stringでラップする
                    decision.metadata.insert(key.clone(), serde_json::Value::String(value_str.to_string()));
                }
            }
        }
    }
    
    Ok(decision)
}

/// 強化されたスタブポリシー評価器（OPAの代わりに使用）
#[derive(Default)]
pub struct StubPolicyEvaluator {
    // スタブ実装
}

impl PolicyEvaluator for StubPolicyEvaluator {
    fn evaluate(&self, input: &PolicyInput) -> McpResult<PolicyDecision> {
        // コマンド実行ポリシー
        if !input.command.name.is_empty() {
            return self.evaluate_command(input);
        }
        
        // ファイルアクセスポリシー
        if let Some(file_info) = &input.file {
            return self.evaluate_file_access(input, file_info);
        }
        
        // ネットワークアクセスポリシー
        if let Some(network_info) = &input.network {
            return self.evaluate_network_access(input, network_info);
        }
        
        // デフォルトは許可（警告付き）
        Ok(PolicyDecision {
            allow: true,
            warnings: vec!["不明なリクエストタイプです。本番環境では拒否されます。".to_string()],
            reasons: vec![],
            metadata: Default::default(),
        })
    }
}

impl StubPolicyEvaluator {
    // コマンド実行ポリシーの評価
    fn evaluate_command(&self, input: &PolicyInput) -> McpResult<PolicyDecision> {
        let cmd = &input.command.name;
        
        // 許可されたコマンド
        let allowed_commands = [
            "ls", "echo", "cat", "grep", "find", 
            "python", "python3", "node", "npm"
        ];
        
        // 危険なコマンド
        let dangerous_commands = [
            "rm", "dd", "wget", "curl", "chmod", 
            "chown", "sudo", "su"
        ];
        
        // 管理者かどうか
        let is_admin = input.user.roles.iter().any(|r| r == "admin");
        
        // 危険なコマンドは拒否
        if dangerous_commands.contains(&cmd.as_str()) {
            return Ok(PolicyDecision {
                allow: false,
                warnings: vec![],
                reasons: vec![format!("コマンド '{}' は危険なため禁止されています", cmd)],
                metadata: Default::default(),
            });
        }
        
        // 許可されたコマンドリストにあるか管理者なら許可
        if allowed_commands.contains(&cmd.as_str()) || is_admin {
            let mut warnings = vec![];
            
            if is_admin {
                warnings.push("管理者として実行中。すべての操作が監査されます。".to_string());
            }
            
            // スタブ警告
            warnings.push("スタブポリシーエンジンを使用中のため、本番環境では使用しないでください".to_string());
            
            return Ok(PolicyDecision {
                allow: true,
                warnings,
                reasons: vec![],
                metadata: Default::default(),
            });
        }
        
        // それ以外は拒否
        Ok(PolicyDecision {
            allow: false,
            warnings: vec![],
            reasons: vec![format!("コマンド '{}' は許可リストにありません", cmd)],
            metadata: Default::default(),
        })
    }
    
    // ファイルアクセスポリシーの評価
    fn evaluate_file_access(&self, _input: &PolicyInput, file_info: &crate::models::FileInfo) -> McpResult<PolicyDecision> {
        // 読み取り可能なパス
        let readable_paths = [
            "/workspace/", "/tmp/", "/data/public/"
        ];
        
        // 書き込み可能なパス
        let writable_paths = [
            "/workspace/", "/tmp/"
        ];
        
        // 実行可能なパス
        let executable_paths = [
            "/workspace/bin/", "/usr/bin/", "/bin/"
        ];
        
        // 禁止パス
        let denied_paths = [
            "/etc/", "/var/", "/root/", "/home/"
        ];
        
        // パスのチェック関数
        let path_matches = |path: &str, pattern: &str| path.starts_with(pattern);
        
        // 禁止パスのチェック
        for pattern in denied_paths.iter() {
            if path_matches(&file_info.path, pattern) {
                return Ok(PolicyDecision {
                    allow: false,
                    warnings: vec![],
                    reasons: vec![format!("パス '{}' へのアクセスは禁止されています", file_info.path)],
                    metadata: Default::default(),
                });
            }
        }
        
        // モードに応じたチェック
        let allowed = match file_info.mode.as_str() {
            "read" => readable_paths.iter().any(|p| path_matches(&file_info.path, p)),
            "write" => writable_paths.iter().any(|p| path_matches(&file_info.path, p)),
            "execute" => executable_paths.iter().any(|p| path_matches(&file_info.path, p)),
            _ => false,
        };
        
        if allowed {
            let mut warnings = vec![];
            
            if file_info.mode == "write" {
                warnings.push("ファイル書き込み操作は監査されます".to_string());
            }
            
            // スタブ警告
            warnings.push("スタブポリシーエンジンを使用中のため、本番環境では使用しないでください".to_string());
            
            Ok(PolicyDecision {
                allow: true,
                warnings,
                reasons: vec![],
                metadata: Default::default(),
            })
        } else {
            Ok(PolicyDecision {
                allow: false,
                warnings: vec![],
                reasons: vec![format!("パス '{}' への '{}' アクセスは許可されていません", 
                                      file_info.path, file_info.mode)],
                metadata: Default::default(),
            })
        }
    }
    
    // ネットワークアクセスポリシーの評価
    fn evaluate_network_access(&self, _input: &PolicyInput, network_info: &crate::models::NetworkInfo) -> McpResult<PolicyDecision> {
        // 許可されたホスト
        let allowed_hosts = [
            "api.example.com", "cdn.example.com", "data.example.com"
        ];
        
        // 許可されたポート
        let allowed_ports = [80, 443, 8080];
        
        // 許可されたプロトコル
        let allowed_protocols = ["tcp", "https"];
        
        // ホストのチェック
        let host_allowed = allowed_hosts.contains(&network_info.host.as_str());
        
        // ポートのチェック
        let port_allowed = allowed_ports.contains(&network_info.port);
        
        // プロトコルのチェック
        let protocol_allowed = allowed_protocols.contains(&network_info.protocol.as_str());
        
        if host_allowed && port_allowed && protocol_allowed {
            Ok(PolicyDecision {
                allow: true,
                warnings: vec![
                    "ネットワークリクエストは監査されます".to_string(),
                    "スタブポリシーエンジンを使用中のため、本番環境では使用しないでください".to_string(),
                ],
                reasons: vec![],
                metadata: Default::default(),
            })
        } else {
            // 拒否理由を集める
            let mut reasons = vec![];
            
            if !host_allowed {
                reasons.push(format!("ホスト '{}' へのアクセスは許可されていません", network_info.host));
            }
            
            if !port_allowed {
                reasons.push(format!("ポート {} へのアクセスは許可されていません", network_info.port));
            }
            
            if !protocol_allowed {
                reasons.push(format!("プロトコル '{}' の使用は許可されていません", network_info.protocol));
            }
            
            Ok(PolicyDecision {
                allow: false,
                warnings: vec![],
                reasons,
                metadata: Default::default(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CommandInfo, UserInfo, FileInfo, NetworkInfo};
    use std::collections::HashMap;

    // スタブポリシーエバリュエータのテスト
    #[test]
    fn test_stub_policy_evaluator() {
        let evaluator = StubPolicyEvaluator::default();
        
        // 安全なコマンドのテスト
        let input_safe = PolicyInput {
            user: UserInfo {
                id: "user1".to_string(),
                tenant_id: "tenant1".to_string(),
                roles: vec!["user".to_string()],
                attributes: HashMap::new(),
            },
            command: CommandInfo {
                name: "ls".to_string(),
                args: vec!["-la".to_string()],
                cwd: "/workspace".to_string(),
                env: HashMap::new(),
            },
            file: None,
            network: None,
            resources: Default::default(),
            context: HashMap::new(),
        };
        
        let result_safe = evaluator.evaluate(&input_safe).unwrap();
        assert!(result_safe.allow);
        assert!(!result_safe.warnings.is_empty());
        
        // 危険なコマンドのテスト
        let mut input_dangerous = input_safe.clone();
        input_dangerous.command.name = "rm".to_string();
        
        let result_dangerous = evaluator.evaluate(&input_dangerous).unwrap();
        assert!(!result_dangerous.allow);
        assert!(!result_dangerous.reasons.is_empty());
    }

    // ポリシーエンジンのテスト
    #[test]
    fn test_policy_engine_with_stub() {
        let engine = PolicyEngine::new();
        
        // 安全なコマンドのテスト
        let input_safe = PolicyInput {
            user: UserInfo {
                id: "user1".to_string(),
                tenant_id: "tenant1".to_string(),
                roles: vec!["user".to_string()],
                attributes: HashMap::new(),
            },
            command: CommandInfo {
                name: "ls".to_string(),
                args: vec!["-la".to_string()],
                cwd: "/workspace".to_string(),
                env: HashMap::new(),
            },
            file: None,
            network: None,
            resources: Default::default(),
            context: HashMap::new(),
        };
        
        let result_safe = engine.check_command_execution(&input_safe);
        assert!(result_safe.is_ok());
        
        // 危険なコマンドのテスト
        let mut input_dangerous = input_safe.clone();
        input_dangerous.command.name = "rm".to_string();
        
        let result_dangerous = engine.check_command_execution(&input_dangerous);
        assert!(result_dangerous.is_err());
    }
    
    // ファイルアクセスポリシーのテスト
    #[test]
    fn test_file_access_policy() {
        let engine = PolicyEngine::new();
        
        // 許可されたファイル読み取り
        let input_read_allowed = PolicyInput {
            user: UserInfo::default(),
            command: CommandInfo::default(),
            file: Some(FileInfo {
                path: "/workspace/data.txt".to_string(),
                mode: "read".to_string(),
            }),
            network: None,
            resources: Default::default(),
            context: HashMap::new(),
        };
        
        assert!(engine.check_file_access(&input_read_allowed).is_ok());
        
        // 禁止されたファイル読み取り
        let input_read_denied = PolicyInput {
            user: UserInfo::default(),
            command: CommandInfo::default(),
            file: Some(FileInfo {
                path: "/etc/passwd".to_string(),
                mode: "read".to_string(),
            }),
            network: None,
            resources: Default::default(),
            context: HashMap::new(),
        };
        
        assert!(engine.check_file_access(&input_read_denied).is_err());
    }
    
    // ネットワークアクセスポリシーのテスト
    #[test]
    fn test_network_access_policy() {
        let engine = PolicyEngine::new();
        
        // 許可されたネットワークアクセス
        let input_network_allowed = PolicyInput {
            user: UserInfo::default(),
            command: CommandInfo::default(),
            file: None,
            network: Some(NetworkInfo {
                host: "api.example.com".to_string(),
                port: 443,
                protocol: "https".to_string(),
            }),
            resources: Default::default(),
            context: HashMap::new(),
        };
        
        assert!(engine.check_network_access(&input_network_allowed).is_ok());
        
        // 禁止されたネットワークアクセス
        let input_network_denied = PolicyInput {
            user: UserInfo::default(),
            command: CommandInfo::default(),
            file: None,
            network: Some(NetworkInfo {
                host: "malicious.example.com".to_string(),
                port: 8888,
                protocol: "https".to_string(),
            }),
            resources: Default::default(),
            context: HashMap::new(),
        };
        
        assert!(engine.check_network_access(&input_network_denied).is_err());
    }
} 