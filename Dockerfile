# ビルドステージ
FROM rust:latest AS builder

WORKDIR /app

# 必要なツールのインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# 依存関係のみを先にビルドしてキャッシュを活用
COPY Cargo.toml Cargo.lock ./
COPY crates/mcp-common/Cargo.toml ./crates/mcp-common/
COPY crates/mcp-gateway/Cargo.toml ./crates/mcp-gateway/
COPY crates/mcp-policy/Cargo.toml ./crates/mcp-policy/
COPY crates/mcp-sandbox/Cargo.toml ./crates/mcp-sandbox/
RUN mkdir -p crates/mcp-common/src crates/mcp-gateway/src crates/mcp-policy/src crates/mcp-sandbox/src \
    && touch crates/mcp-common/src/lib.rs crates/mcp-gateway/src/lib.rs crates/mcp-policy/src/lib.rs crates/mcp-sandbox/src/lib.rs \
    && cargo build --release

# ソースコードをコピーしてビルド
COPY proto ./proto
COPY crates ./crates

# 必要なパッチを適用（エラー修正）
RUN sed -i 's/ProcessCollector::new()\n                .unwrap()/ProcessCollector::new(\n                prometheus::process_collector::ProcessCollector::pid_t::from_inner(std::process::id() as i32),\n                "mcp_gateway"\n            )/g' crates/mcp-gateway/src/metrics.rs \
    && sed -i 's/use mcp_common::{McpError, IntoStatus, ErrorResponse, ErrorDetail};/use mcp_common::{McpError, grpc::IntoStatus, error::{ErrorResponse, ErrorDetail}};/g' crates/mcp-gateway/src/error.rs \
    && sed -i 's/use mcp_common::{McpError, McpResult};/use mcp_common::{McpError, error::McpResult};/g' crates/mcp-gateway/src/service.rs \
    && sed -i 's/use mcp_policy::engine::PolicyEngine;/use mcp_policy::PolicyEngine;/g' crates/mcp-gateway/src/service.rs \
    && sed -i 's/use mcp_policy::models::{CommandInfo, PolicyInput, UserInfo};/use mcp_policy::{CommandInfo, PolicyInput, UserInfo};/g' crates/mcp-gateway/src/service.rs \
    && sed -i 's/use mcp_policy::engine::PolicyEngine;/use mcp_policy::PolicyEngine;/g' crates/mcp-gateway/src/lib.rs

# ビルド実行
RUN cargo build --release

# 必要なバイナリを取り出す
RUN mkdir -p /app/dist/bin \
    && cp /app/target/release/mcp-gateway /app/dist/bin/ \
    && cp -r /app/proto /app/dist/

# 実行ステージ
FROM debian:bullseye-slim

# 必要なパッケージのインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    bubblewrap \
    libseccomp2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ビルドステージからバイナリをコピー
COPY --from=builder /app/dist/bin/mcp-gateway /usr/local/bin/
COPY --from=builder /app/dist/proto /app/proto

# ワークスペースディレクトリを作成
RUN mkdir -p /workspace
VOLUME /workspace

# サービスのポートを公開
EXPOSE 8081

# コマンドを設定
ENTRYPOINT ["mcp-gateway"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8081"] 