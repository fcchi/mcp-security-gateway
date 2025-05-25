use crate::models::{PolicyDecision, PolicyInput};
use mcp_common::error::{error_code, McpError, McpResult};
use serde_json::json;
use std::fmt;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Policy evaluation interface
pub trait PolicyEvaluator: Send + Sync {
    /// Evaluate policy and return decision result
    fn evaluate(&self, input: &PolicyInput) -> McpResult<PolicyDecision>;
}

/// Policy engine
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
    /// Create a new policy engine (using default StubPolicyEvaluator)
    pub fn new() -> Self {
        Self::with_evaluator(StubPolicyEvaluator::default())
    }

    /// Create a new policy engine with specified policy evaluator
    pub fn with_evaluator(evaluator: impl PolicyEvaluator + 'static) -> Self {
        Self {
            evaluator: Arc::new(evaluator),
        }
    }

    /// Evaluate whether to allow command execution
    pub fn check_command_execution(&self, input: &PolicyInput) -> McpResult<()> {
        debug!(
            "Policy evaluation: Command execution command={}",
            input.command.name
        );

        let decision = self.evaluator.evaluate(input)?;

        if !decision.allow {
            let reason = decision.reasons.join(", ");
            let message = if reason.is_empty() {
                format!(
                    "Command '{}' execution was denied by policy",
                    input.command.name
                )
            } else {
                format!(
                    "Command '{}' execution was denied by policy: {}",
                    input.command.name, reason
                )
            };

            error!("Policy violation: {}", message);

            // Return error with details
            let details = json!({
                "command": input.command.name,
                "reasons": decision.reasons,
                "user_id": input.user.id,
                "tenant_id": input.user.tenant_id
            });

            return Err(policy_violation(
                error_code::POLICY_COMMAND_NOT_ALLOWED,
                message,
                Some(details),
            ));
        }

        // Log warnings if any
        if !decision.warnings.is_empty() {
            info!(
                "Policy warning: Command '{}' was allowed, but with warnings: {}",
                input.command.name,
                decision.warnings.join(", ")
            );
        }

        Ok(())
    }

    /// Evaluate whether to allow file access
    pub fn check_file_access(&self, input: &PolicyInput) -> McpResult<()> {
        if let Some(file_info) = &input.file {
            debug!(
                "Policy evaluation: File access path={}, mode={}",
                file_info.path, file_info.mode
            );

            let decision = self.evaluator.evaluate(input)?;

            if !decision.allow {
                let reason = decision.reasons.join(", ");
                let message = if reason.is_empty() {
                    format!(
                        "{} access to file '{}' was denied by policy",
                        file_info.mode, file_info.path
                    )
                } else {
                    format!(
                        "{} access to file '{}' was denied by policy: {}",
                        file_info.mode, file_info.path, reason
                    )
                };

                error!("Policy violation: {}", message);

                let details = json!({
                    "path": file_info.path,
                    "mode": file_info.mode,
                    "reasons": decision.reasons,
                    "user_id": input.user.id
                });

                return Err(policy_violation(
                    error_code::POLICY_FILE_ACCESS_DENIED,
                    message,
                    Some(details),
                ));
            }

            if !decision.warnings.is_empty() {
                info!(
                    "Policy warning: {} access to file '{}' was allowed, but with warnings: {}",
                    file_info.mode,
                    file_info.path,
                    decision.warnings.join(", ")
                );
            }
        }

        Ok(())
    }

    /// Evaluate whether to allow network access
    pub fn check_network_access(&self, input: &PolicyInput) -> McpResult<()> {
        if let Some(network_info) = &input.network {
            debug!(
                "Policy evaluation: Network access host={}:{}, protocol={}",
                network_info.host, network_info.port, network_info.protocol
            );

            let decision = self.evaluator.evaluate(input)?;

            if !decision.allow {
                let reason = decision.reasons.join(", ");
                let message = if reason.is_empty() {
                    format!(
                        "{} access to host '{}:{}' was denied by policy",
                        network_info.protocol, network_info.host, network_info.port
                    )
                } else {
                    format!(
                        "{} access to host '{}:{}' was denied by policy: {}",
                        network_info.protocol, network_info.host, network_info.port, reason
                    )
                };

                error!("Policy violation: {}", message);

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
                    Some(details),
                ));
            }

            if !decision.warnings.is_empty() {
                info!(
                    "Policy warning: {} access to host '{}:{}' was allowed, but with warnings: {}",
                    network_info.protocol,
                    network_info.host,
                    network_info.port,
                    decision.warnings.join(", ")
                );
            }
        }

        Ok(())
    }
}

/// Helper function: Generate policy violation error
pub fn policy_violation(
    _code: u32,
    message: String,
    _details: Option<serde_json::Value>,
) -> McpError {
    McpError::PolicyViolation(message)
}

/// OPA policy evaluator
#[allow(dead_code)]
pub struct OpaEvaluator {
    // OPA policy module (stub implementation)
    query_path: String,
}

impl OpaEvaluator {
    /// Create a new OPA policy evaluator
    pub fn new(_wasm_policy_bytes: &[u8], query_path: &str) -> McpResult<Self> {
        // Note: The opa-wasm API may have changed. Adjustments may be needed based on actual API specs.
        // This stub implementation abstracts the API details.

        Ok(Self {
            query_path: query_path.to_string(),
        })
    }
}

impl PolicyEvaluator for OpaEvaluator {
    fn evaluate(&self, input: &PolicyInput) -> McpResult<PolicyDecision> {
        // Convert input to JSON format
        let _input_json = serde_json::to_value(input)
            .map_err(|e| McpError::Internal(format!("Failed to serialize input: {}", e)))?;

        // Note: Actual OPA evaluation needs to be implemented here
        // This is a stub implementation

        // Return stub result
        Ok(PolicyDecision {
            allow: true,
            warnings: vec!["OPA WASM evaluation is a stub implementation.".to_string()],
            reasons: vec![],
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// Helper function to convert OPA result to PolicyDecision
#[allow(dead_code)]
fn parse_opa_result(result: serde_json::Value) -> McpResult<PolicyDecision> {
    // Basic fallback values
    let mut decision = PolicyDecision {
        allow: false,
        warnings: Vec::new(),
        reasons: Vec::new(),
        metadata: std::collections::HashMap::new(),
    };

    // If the result is not an object
    if !result.is_object() {
        return Ok(decision);
    }

    let result_obj = result.as_object().unwrap();

    // Parse allow field
    if let Some(allow) = result_obj.get("allow") {
        if let Some(allow_bool) = allow.as_bool() {
            decision.allow = allow_bool;
        }
    }

    // Parse warnings field
    if let Some(warnings) = result_obj.get("warnings") {
        if let Some(warnings_arr) = warnings.as_array() {
            decision.warnings = warnings_arr
                .iter()
                .filter_map(|w| w.as_str().map(|s| s.to_string()))
                .collect();
        }
    }

    // Parse reasons field
    if let Some(reasons) = result_obj.get("reasons") {
        if let Some(reasons_arr) = reasons.as_array() {
            decision.reasons = reasons_arr
                .iter()
                .filter_map(|r| r.as_str().map(|s| s.to_string()))
                .collect();
        }
    }

    // Parse additional metadata (for future extensions)
    if let Some(metadata) = result_obj.get("metadata") {
        if let Some(metadata_obj) = metadata.as_object() {
            for (key, value) in metadata_obj {
                if let Some(value_str) = value.as_str() {
                    // Wrap with Value::String
                    decision.metadata.insert(
                        key.clone(),
                        serde_json::Value::String(value_str.to_string()),
                    );
                }
            }
        }
    }

    Ok(decision)
}

/// Enhanced stub policy evaluator (used instead of OPA)
#[derive(Default)]
pub struct StubPolicyEvaluator {
    // Stub implementation
}

impl PolicyEvaluator for StubPolicyEvaluator {
    fn evaluate(&self, input: &PolicyInput) -> McpResult<PolicyDecision> {
        // Command execution policy
        if !input.command.name.is_empty() {
            return self.evaluate_command(input);
        }

        // File access policy
        if let Some(file_info) = &input.file {
            return self.evaluate_file_access(input, file_info);
        }

        // Network access policy
        if let Some(network_info) = &input.network {
            return self.evaluate_network_access(input, network_info);
        }

        // Default is to allow (with warning)
        Ok(PolicyDecision {
            allow: true,
            warnings: vec![
                "Unknown request type. Would be denied in production environment.".to_string(),
            ],
            reasons: vec![],
            metadata: Default::default(),
        })
    }
}

impl StubPolicyEvaluator {
    // Command execution policy evaluation
    fn evaluate_command(&self, input: &PolicyInput) -> McpResult<PolicyDecision> {
        let cmd = &input.command.name;

        // Allowed commands
        let allowed_commands = [
            "ls", "echo", "cat", "grep", "find", "python", "python3", "node", "npm",
        ];

        // Dangerous commands
        let dangerous_commands = ["rm", "dd", "wget", "curl", "chmod", "chown", "sudo", "su"];

        // Check if user is admin
        let is_admin = input.user.roles.iter().any(|r| r == "admin");

        // Deny dangerous commands
        if dangerous_commands.contains(&cmd.as_str()) {
            return Ok(PolicyDecision {
                allow: false,
                warnings: vec![],
                reasons: vec![format!("Command '{}' is forbidden as it is dangerous", cmd)],
                metadata: Default::default(),
            });
        }

        // Allow if in allowed commands list or user is admin
        if allowed_commands.contains(&cmd.as_str()) || is_admin {
            let mut warnings = vec![];

            if is_admin {
                warnings
                    .push("Executing as administrator. All operations are audited.".to_string());
            }

            // Stub warning
            warnings
                .push("Using stub policy engine, do not use in production environment".to_string());

            return Ok(PolicyDecision {
                allow: true,
                warnings,
                reasons: vec![],
                metadata: Default::default(),
            });
        }

        // Otherwise deny
        Ok(PolicyDecision {
            allow: false,
            warnings: vec![],
            reasons: vec![format!("Command '{}' is not in the allowed list", cmd)],
            metadata: Default::default(),
        })
    }

    // File access policy evaluation
    fn evaluate_file_access(
        &self,
        _input: &PolicyInput,
        file_info: &crate::models::FileInfo,
    ) -> McpResult<PolicyDecision> {
        // Readable paths
        let readable_paths = ["/workspace/", "/tmp/", "/data/public/"];

        // Writable paths
        let writable_paths = ["/workspace/", "/tmp/"];

        // Executable paths
        let executable_paths = ["/workspace/bin/", "/usr/bin/", "/bin/"];

        // Denied paths
        let denied_paths = ["/etc/", "/var/", "/root/", "/home/"];

        // Path checking function
        let path_matches = |path: &str, pattern: &str| path.starts_with(pattern);

        // Check denied paths
        for pattern in denied_paths.iter() {
            if path_matches(&file_info.path, pattern) {
                return Ok(PolicyDecision {
                    allow: false,
                    warnings: vec![],
                    reasons: vec![format!("Access to path '{}' is forbidden", file_info.path)],
                    metadata: Default::default(),
                });
            }
        }

        // Check based on mode
        let allowed = match file_info.mode.as_str() {
            "read" => readable_paths
                .iter()
                .any(|p| path_matches(&file_info.path, p)),
            "write" => writable_paths
                .iter()
                .any(|p| path_matches(&file_info.path, p)),
            "execute" => executable_paths
                .iter()
                .any(|p| path_matches(&file_info.path, p)),
            _ => false,
        };

        if allowed {
            let mut warnings = vec![];

            if file_info.mode == "write" {
                warnings.push("File write operations are audited".to_string());
            }

            // Stub warning
            warnings
                .push("Using stub policy engine, do not use in production environment".to_string());

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
                reasons: vec![format!(
                    "'{}' access to path '{}' is not allowed",
                    file_info.mode, file_info.path
                )],
                metadata: Default::default(),
            })
        }
    }

    // Network access policy evaluation
    fn evaluate_network_access(
        &self,
        _input: &PolicyInput,
        network_info: &crate::models::NetworkInfo,
    ) -> McpResult<PolicyDecision> {
        // Allowed hosts
        let allowed_hosts = ["api.example.com", "cdn.example.com", "data.example.com"];

        // Allowed ports
        let allowed_ports = [80, 443, 8080];

        // Allowed protocols
        let allowed_protocols = ["tcp", "https"];

        // Check host
        let host_allowed = allowed_hosts.contains(&network_info.host.as_str());

        // Check port
        let port_allowed = allowed_ports.contains(&network_info.port);

        // Check protocol
        let protocol_allowed = allowed_protocols.contains(&network_info.protocol.as_str());

        if host_allowed && port_allowed && protocol_allowed {
            Ok(PolicyDecision {
                allow: true,
                warnings: vec![
                    "Network requests are audited".to_string(),
                    "Using stub policy engine, do not use in production environment".to_string(),
                ],
                reasons: vec![],
                metadata: Default::default(),
            })
        } else {
            // Collect denial reasons
            let mut reasons = vec![];

            if !host_allowed {
                reasons.push(format!(
                    "Access to host '{}' is not allowed",
                    network_info.host
                ));
            }

            if !port_allowed {
                reasons.push(format!(
                    "Access to port {} is not allowed",
                    network_info.port
                ));
            }

            if !protocol_allowed {
                reasons.push(format!(
                    "Use of protocol '{}' is not allowed",
                    network_info.protocol
                ));
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
    use crate::models::{CommandInfo, FileInfo, NetworkInfo, UserInfo};
    use std::collections::HashMap;

    // Test for stub policy evaluator
    #[test]
    fn test_stub_policy_evaluator() {
        let evaluator = StubPolicyEvaluator::default();

        // Test for safe command
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

        // Test for dangerous command
        let mut input_dangerous = input_safe.clone();
        input_dangerous.command.name = "rm".to_string();

        let result_dangerous = evaluator.evaluate(&input_dangerous).unwrap();
        assert!(!result_dangerous.allow);
        assert!(!result_dangerous.reasons.is_empty());
    }

    // Test for policy engine
    #[test]
    fn test_policy_engine_with_stub() {
        let engine = PolicyEngine::new();

        // Test for safe command
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

        // Test for dangerous command
        let mut input_dangerous = input_safe.clone();
        input_dangerous.command.name = "rm".to_string();

        let result_dangerous = engine.check_command_execution(&input_dangerous);
        assert!(result_dangerous.is_err());
    }

    // Test for file access policy
    #[test]
    fn test_file_access_policy() {
        let engine = PolicyEngine::new();

        // Allowed file read
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

        // Denied file read
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

    // Test for network access policy
    #[test]
    fn test_network_access_policy() {
        let engine = PolicyEngine::new();

        // Allowed network access
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

        // Denied network access
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
