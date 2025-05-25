// 生成されたprotoコードをインポート
// build.rsでは生成先をsrc/protoに指定しているため、mod.rsとして読み込む
pub mod mcp {
    // protoディレクトリでmcp.rsが自動生成されるため、それをincludeする
    include!("proto/mcp.rs");
}

// 便利なtypenamesをreexport
pub use mcp::mcp_service_client::McpServiceClient;
pub use mcp::mcp_service_server::{McpService, McpServiceServer};
pub use mcp::*;
