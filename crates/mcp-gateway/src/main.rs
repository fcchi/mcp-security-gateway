use mcp_gateway::{create_server, new_service};
use mcp_gateway::server::run_server;
use mcp_gateway::tracing::{init_tracing, shutdown_tracing, TracingConfig};
use std::net::SocketAddr;
use std::time::SystemTime;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // トレース設定を環境変数から構築
    let tracing_config = TracingConfig {
        enabled: std::env::var("OTEL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
        service_name: std::env::var("OTEL_SERVICE_NAME")
            .unwrap_or_else(|_| "mcp-security-gateway".to_string()),
        otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string()),
        sampling_ratio: std::env::var("OTEL_SAMPLER_RATIO")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse()
            .unwrap_or(1.0),
        batch_interval_secs: std::env::var("OTEL_BATCH_INTERVAL_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5),
        parent_base_trace_id_ratio: std::env::var("OTEL_PARENT_BASED_RATIO")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse()
            .unwrap_or(1.0),
        log_level: std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string()),
    };
    
    // トレーシングシステムを初期化
    init_tracing(tracing_config)?;
    
    info!("MCPセキュリティゲートウェイを起動しています...");
    
    // サービスの起動時間を記録
    let start_time = SystemTime::now();
    
    // サービス実装を作成
    let service = new_service(start_time);
    
    // バインドするアドレス
    let addr = std::env::var("MCP_BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .parse::<SocketAddr>()?;
    
    // gRPCサービスを作成
    let grpc_service = create_server(service);
    
    // サーバーを起動
    info!("サーバーを開始します: {}", addr);
    run_server(addr, grpc_service).await?;
    
    // トレーシングをシャットダウン
    shutdown_tracing();
    
    Ok(())
} 