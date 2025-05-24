//! MCPセキュリティゲートウェイのポリシーエンジン
//!
//! OPA (Open Policy Agent) Regoポリシーを評価するためのエンジンを提供します。

pub mod engine;
pub mod models;

/// Re-export the main components
pub use engine::{PolicyEngine, PolicyEvaluator, StubPolicyEvaluator};
pub use models::{PolicyDecision, PolicyInput, CommandInfo, UserInfo, FileInfo, NetworkInfo, ResourceLimits};

/// Provide version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 