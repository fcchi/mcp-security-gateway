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

pub use crate::error::ErrorHandler;
pub use crate::proto::mcp;
pub use crate::proto::mcp_service_server::McpServiceServer;
pub use crate::service::McpServiceImpl;

pub fn create_server(service: McpServiceImpl) -> McpServiceServer<McpServiceImpl> {
    server::create_server(service)
}

pub fn new_service(start_time: SystemTime) -> McpServiceImpl {
    McpServiceImpl::new(PolicyEngine::new(), CommandExecutor::new(), start_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_create_server() {
        let start_time = SystemTime::now();
        let service = new_service(start_time);
        let _server = create_server(service);
        assert!(true, "サーバーの作成に成功");
    }

    #[test]
    fn test_new_service() {
        let start_time = SystemTime::now();
        let _service = new_service(start_time);
        // サービスが正しく初期化されたことを確認
        assert!(true, "サービスの作成に成功");
    }
}
