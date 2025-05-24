pub mod error;
pub mod grpc;
pub mod models;
pub mod utils;
#[cfg(test)]
mod grpc_mapping_tests;

pub use error::{McpError, McpResult, ErrorResponse, ErrorDetail, IntoMcpResult, ToMcpError};
pub use grpc::IntoStatus;

/// バージョン情報
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// クレート名
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME"); 