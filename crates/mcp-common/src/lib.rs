pub mod error;
pub mod grpc;
#[cfg(test)]
mod grpc_mapping_tests;
pub mod models;
pub mod utils;

pub use error::{ErrorDetail, ErrorResponse, IntoMcpResult, McpError, McpResult, ToMcpError};
pub use grpc::IntoStatus;

/// バージョン情報
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// クレート名
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
