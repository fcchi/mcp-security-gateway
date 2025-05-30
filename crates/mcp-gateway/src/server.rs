use crate::proto::mcp_service_server::McpServiceServer;
use crate::McpServiceImpl;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::info;
use axum::{Router, routing::get, response::Response, body::Body, http::{header, StatusCode}};
use prometheus::Encoder;
use prometheus::TextEncoder;
use crate::metrics;

/// gRPCサーバーの作成
///
/// # 引数
/// * `service` - サービス実装
///
/// # 戻り値
/// * サーバーインスタンス
pub fn create_server(service: McpServiceImpl) -> McpServiceServer<McpServiceImpl> {
    McpServiceServer::new(service)
}

/// サーバーを実行する
pub async fn run_server(
    addr: SocketAddr,
    service: McpServiceServer<McpServiceImpl>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("gRPCサーバーを起動します: {}", addr);

    // メトリクスを初期化
    metrics::init_metrics();

    // メトリクスサーバーを起動
    start_metrics_server();

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}

/// メトリクスサーバーを起動する
fn start_metrics_server() {
    // メトリクスサーバーのエンドポイントを定義
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler));

    // メトリクスサーバーを別スレッドで起動
    let metrics_addr = std::net::SocketAddr::from(([0, 0, 0, 0], 9090));
    info!("メトリクスサーバーを起動します: {}", metrics_addr);

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(metrics_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                tracing::error!("メトリクスサーバーのバインドに失敗しました: {}", e);
                return;
            }
        };
        
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("メトリクスサーバーの起動に失敗しました: {}", e);
        }
    });
}

/// メトリクスエンドポイントのハンドラー
async fn metrics_handler() -> Response<Body> {
    let encoder = TextEncoder::new();
    let metrics = metrics::get_registry().gather();
    
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metrics, &mut buffer) {
        tracing::error!("メトリクスのエンコードに失敗しました: {}", e);
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("メトリクスのエンコードに失敗しました"))
            .unwrap();
    }
    
    match String::from_utf8(buffer) {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, encoder.format_type())
            .body(Body::from(body))
            .unwrap(),
        Err(e) => {
            tracing::error!("メトリクスのUTF-8変換に失敗しました: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("メトリクスのUTF-8変換に失敗しました"))
                .unwrap()
        }
    }
}

/// ヘルスチェックエンドポイントのハンドラー
async fn health_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"status":"healthy"}"#))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_server() {
        // サービス実装を作成
        let service = crate::McpServiceImpl::new(
            mcp_policy::engine::PolicyEngine::new(),
            mcp_sandbox::CommandExecutor::new(),
            std::time::SystemTime::now(),
        );
        
        // サーバーを作成
        let _server = create_server(service);
        assert!(true); // 型チェックのみで確認
    }
} 