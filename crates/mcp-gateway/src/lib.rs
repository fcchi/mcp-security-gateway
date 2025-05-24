//! MCPセキュリティゲートウェイのAPIゲートウェイ
//!
//! gRPCおよびRESTインターフェースを提供するゲートウェイサービス

pub mod error;
pub mod metrics;
pub mod server;
pub mod service;
pub mod proto;
pub mod tracing;

pub use crate::proto::mcp;
pub use crate::service::McpServiceImpl;
pub use crate::error::ErrorHandler;
pub use crate::proto::mcp_service_server::McpServiceServer;

use mcp_policy::engine::PolicyEngine;
use mcp_sandbox::CommandExecutor;
use std::time::SystemTime;

pub fn create_server(service: McpServiceImpl) -> McpServiceServer<McpServiceImpl> {
    server::create_server(service)
}

pub fn new_service(start_time: SystemTime) -> McpServiceImpl {
    McpServiceImpl::new(PolicyEngine::new(), CommandExecutor::new(), start_time)
}

#[cfg(test)]
mod service_tests; 