//! MCPセキュリティゲートウェイのサンドボックス
//!
//! bubblewrapとseccompを使用してコマンド実行を分離・制限するサンドボックス機能を提供します。

pub mod bubblewrap;
pub mod executor;
pub mod models;
pub mod runner;
pub mod seccomp;

#[cfg(test)]
mod executor_tests;
#[cfg(test)]
mod runner_tests;

pub use executor::CommandExecutor;
pub use models::{ExecutionRequest, ExecutionResult, ResourceUsage, SandboxConfig};
pub use runner::SandboxRunner;
