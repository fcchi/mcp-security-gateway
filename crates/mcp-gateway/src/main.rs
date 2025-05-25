use mcp_gateway::server::run_server;
use mcp_gateway::tracing::{init_tracing, shutdown_tracing, TracingConfig};
use mcp_gateway::{create_server, new_service};
use std::net::SocketAddr;
use std::time::SystemTime;
use tracing::info;
use std::env;

/// 環境変数からトレーシング設定を構築する
pub fn build_tracing_config() -> TracingConfig {
    TracingConfig {
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
        log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    }
}

/// バインドアドレスを環境変数から取得
pub fn get_bind_address() -> Result<SocketAddr, Box<dyn std::error::Error>> {
    Ok(std::env::var("MCP_BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .parse::<SocketAddr>()?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // トレース設定を環境変数から構築
    let tracing_config = build_tracing_config();

    // トレーシングシステムを初期化
    init_tracing(tracing_config)?;

    info!("MCPセキュリティゲートウェイを起動しています...");

    // サービスの起動時間を記録
    let start_time = SystemTime::now();

    // サービス実装を作成
    let service = new_service(start_time);

    // バインドするアドレス
    let addr = get_bind_address()?;

    // gRPCサービスを作成
    let grpc_service = create_server(service);

    // サーバーを起動
    info!("サーバーを開始します: {}", addr);
    run_server(addr, grpc_service).await?;

    // トレーシングをシャットダウン
    shutdown_tracing();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tracing_config() {
        // デフォルト値のテスト
        let config = build_tracing_config();
        assert_eq!(config.enabled, false);
        assert_eq!(config.service_name, "mcp-security-gateway");

        // 環境変数設定テスト
        env::set_var("OTEL_ENABLED", "true");
        env::set_var("OTEL_SERVICE_NAME", "test-service");

        let config = build_tracing_config();
        assert_eq!(config.enabled, true);
        assert_eq!(config.service_name, "test-service");

        // 環境変数をクリア
        env::remove_var("OTEL_ENABLED");
        env::remove_var("OTEL_SERVICE_NAME");
    }

    #[test]
    fn test_get_bind_address() {
        // デフォルト値のテスト
        let addr = get_bind_address().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8081");

        // 環境変数設定テスト
        env::set_var("MCP_BIND_ADDRESS", "0.0.0.0:9000");
        let addr = get_bind_address().unwrap();
        assert_eq!(addr.to_string(), "0.0.0.0:9000");

        // 環境変数をクリア
        env::remove_var("MCP_BIND_ADDRESS");
    }
    
    #[test]
    fn test_invalid_bind_address() {
        // 無効なアドレス形式のテスト
        env::set_var("MCP_BIND_ADDRESS", "invalid-address");
        let result = get_bind_address();
        assert!(result.is_err());
        
        // 環境変数をクリア
        env::remove_var("MCP_BIND_ADDRESS");
    }
    
    #[test]
    fn test_additional_tracing_config() {
        // 追加の環境変数設定テスト
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:9999");
        env::set_var("OTEL_SAMPLER_RATIO", "0.5");
        env::set_var("OTEL_BATCH_INTERVAL_SECS", "10");
        env::set_var("OTEL_PARENT_BASED_RATIO", "0.75");
        env::set_var("RUST_LOG", "debug");
        
        let config = build_tracing_config();
        assert_eq!(config.otlp_endpoint, "http://localhost:9999");
        assert_eq!(config.sampling_ratio, 0.5);
        assert_eq!(config.batch_interval_secs, 10);
        assert_eq!(config.parent_base_trace_id_ratio, 0.75);
        assert_eq!(config.log_level, "debug");
        
        // 環境変数をクリア
        env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        env::remove_var("OTEL_SAMPLER_RATIO");
        env::remove_var("OTEL_BATCH_INTERVAL_SECS");
        env::remove_var("OTEL_PARENT_BASED_RATIO");
        env::remove_var("RUST_LOG");
    }
    
    #[test]
    fn test_invalid_tracing_config_values() {
        // 無効な数値形式のテスト
        env::set_var("OTEL_ENABLED", "not-a-bool");
        env::set_var("OTEL_SAMPLER_RATIO", "not-a-float");
        env::set_var("OTEL_BATCH_INTERVAL_SECS", "not-a-number");
        env::set_var("OTEL_PARENT_BASED_RATIO", "not-a-float");
        
        let config = build_tracing_config();
        // デフォルト値が使用されるはず
        assert_eq!(config.enabled, false);  // 無効な値はfalseとして扱われる
        assert_eq!(config.sampling_ratio, 1.0);
        assert_eq!(config.batch_interval_secs, 5);
        assert_eq!(config.parent_base_trace_id_ratio, 1.0);
        
        // 環境変数をクリア
        env::remove_var("OTEL_ENABLED");
        env::remove_var("OTEL_SAMPLER_RATIO");
        env::remove_var("OTEL_BATCH_INTERVAL_SECS");
        env::remove_var("OTEL_PARENT_BASED_RATIO");
    }
}
