use std::time::Duration;
use anyhow::{Context, Result};
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_opentelemetry::OpenTelemetryLayer;

/// トレーシングモジュール
///
/// このモジュールはMCPセキュリティゲートウェイの分散トレーシング機能を提供します。
/// OpenTelemetryを使用してトレースデータを収集し、外部システム（JaegerやZipkinなど）に送信します。
///
/// 主な機能:
/// - トレーシングの初期化と設定
/// - OTLPエクスポーターの構成
/// - サンプリング設定
/// - ログ出力との統合
///
/// 使用例:
/// ```
/// use mcp_gateway::tracing::{TracingConfig, init_tracing};
/// 
/// let config = TracingConfig {
///     enabled: true,
///     service_name: "my-service".to_string(),
///     otlp_endpoint: "http://jaeger:4317".to_string(),
///     sampling_ratio: 0.1,
///     ..Default::default()
/// };
/// # // init_tracingはTokioランタイムが必要なため、ドキュメントテストでは実行しません
/// # // let result = init_tracing(config);
/// ```
/// OpenTelemetryトレーシング設定
#[derive(Clone, Debug)]
pub struct TracingConfig {
    /// トレースの有効・無効
    pub enabled: bool,
    /// サービス名
    pub service_name: String,
    /// OTLPエンドポイント
    pub otlp_endpoint: String,
    /// サンプリングレート (0.0 - 1.0)
    pub sampling_ratio: f64,
    /// バッチ送信間隔 (秒)
    pub batch_interval_secs: u64,
    /// レイテンシ確率
    pub parent_base_trace_id_ratio: f64,
    /// ログレベル
    pub log_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_name: "mcp-security-gateway".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            sampling_ratio: 1.0,
            batch_interval_secs: 5,
            parent_base_trace_id_ratio: 1.0,
            log_level: "info".to_string(),
        }
    }
}

/// トレーシングシステムを初期化する
///
/// この関数は、指定された設定に基づいてトレーシングシステムを初期化します。
/// トレースが有効な場合、OpenTelemetryエクスポーターを設定し、トレースデータを
/// 指定されたエンドポイントに送信します。
///
/// # 引数
/// * `config` - トレーシング設定
///
/// # 戻り値
/// * `Result<()>` - 初期化の成功または失敗
///
/// # エラー
/// 以下の状況でエラーが返される可能性があります：
/// - OTLPエクスポーターの初期化に失敗した場合
/// - トレースプロバイダーの登録に失敗した場合
pub fn init_tracing(config: TracingConfig) -> Result<()> {
    // ログレベルフィルターを設定
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // JSONフォーマットのレイヤーを設定
    let fmt_layer = fmt::layer()
        .json()
        .with_ansi(true)
        .with_timer(fmt::time::UtcTime::rfc_3339());

    // OpenTelemetryが有効な場合、OTLPエクスポーターを設定
    if config.enabled {
        // グローバルプロパゲーターを設定
        let propagator = TraceContextPropagator::new();
        opentelemetry::global::set_text_map_propagator(propagator);

        // リソース情報を設定
        let resource = Resource::new(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
            KeyValue::new("deployment.environment", std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())),
        ]);

        // エンドポイントを事前にクローンしておく
        let endpoint = config.otlp_endpoint.clone();

        // トレースプロバイダーを設定
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(config.otlp_endpoint)
                    .with_timeout(Duration::from_secs(config.batch_interval_secs))
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                        config.sampling_ratio
                    ))))
                    .with_resource(resource)
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .context("Failed to create OpenTelemetry tracer")?;

        // OpenTelemetryレイヤーを設定
        let otel_layer = OpenTelemetryLayer::new(tracer);

        // トレーシングサブスクライバーを構築
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        info!("OpenTelemetryトレーシングが有効化されました: endpoint={}", endpoint);
    } else {
        // OpenTelemetryなしでトレーシングサブスクライバーを構築
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        info!("OpenTelemetryトレーシングは無効です");
    }

    Ok(())
}

/// トレーシングシステムをシャットダウンする
///
/// この関数は、OpenTelemetryのトレーシングプロバイダーをシャットダウンし、
/// 残りのスパンデータがエクスポートされるようにします。
/// アプリケーションの終了時に呼び出すことを推奨します。
pub fn shutdown_tracing() {
    info!("トレーシングシステムをシャットダウンしています...");
    opentelemetry::global::shutdown_tracer_provider();
}

/// 新しいトレーシングスパンを作成する
///
/// この関数は、指定された名前と属性で新しいスパンを作成します。
///
/// # 引数
/// * `name` - スパン名
/// * `attributes` - スパン属性（キーと値のペア）
///
/// # 使用例
/// ```
/// use mcp_gateway::tracing::create_span;
/// 
/// let span = create_span("process_request", vec![("http.method", "GET"), ("http.path", "/api/v1/status")]);
/// let _guard = span.enter();
/// // この範囲内の処理はスパンに記録される
/// ```
#[allow(dead_code)]
pub fn create_span(name: &str, attributes: Vec<(&str, &str)>) -> tracing::Span {
    // 動的な名前を使う場合は、以下のようにマクロを使わずに直接作成する
    use tracing::span;
    use tracing::Level;
    
    let span = span!(Level::INFO, "span", name = name);
    
    for (key, value) in attributes {
        span.record(key, value);
    }
    
    span
} 