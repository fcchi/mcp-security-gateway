//! MCPセキュリティゲートウェイのAPIゲートウェイ
//!
//! gRPCおよびRESTインターフェースを提供するゲートウェイサービス

pub mod error;
pub mod metrics;
pub mod proto;
pub mod server;
pub mod service;
pub mod tracing;

// 再エクスポート
use mcp_policy::PolicyEngine;
use mcp_sandbox::CommandExecutor;

#[cfg(test)]
mod service_tests;

use std::time::SystemTime;

pub use crate::proto::mcp;
pub use crate::service::McpServiceImpl;
pub use crate::error::ErrorHandler;
pub use crate::proto::mcp_service_server::McpServiceServer;

pub fn create_server(service: McpServiceImpl) -> McpServiceServer<McpServiceImpl> {
    server::create_server(service)
}

pub fn new_service(start_time: SystemTime) -> McpServiceImpl {
    McpServiceImpl::new(PolicyEngine::new(), CommandExecutor::new(), start_time)
} 