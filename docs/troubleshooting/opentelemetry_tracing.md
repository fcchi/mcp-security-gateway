# OpenTelemetryトレーシングの問題

## 問題の概要

OpenTelemetryの依存関係のバージョン不一致により、ビルドエラーが発生しました。具体的に以下の問題がありました：

1. opentelemetry 0.22.0から`rt-tokio`機能が`opentelemetry`から`opentelemetry_sdk`パッケージに移動した
2. 複数バージョンの`opentelemetry_sdk`が混在していた（0.20.0と0.21.2）
3. `opa-wasm`の指定バージョン0.2.0が存在しなかった

これにより、以下のようなエラーが発生していました：

```
error: failed to select a version for `opentelemetry`.
    ... required by package `mcp-gateway v0.1.0 (/__w/mcp-security-gateway/mcp-security-gateway/crates/mcp-gateway)`
versions that meet the requirements `^0.22.0` are: 0.22.0

the package `mcp-gateway` depends on `opentelemetry`, with features: `rt-tokio` but `opentelemetry` does not have these features.
```

## 現在の対応策

2025年5月24日時点で、以下の一時的な対応を行いました：

1. `crates/mcp-gateway/src/tracing.rs`内のOpenTelemetry関連のコードを一時的にコメントアウト
2. 基本的なロギング機能のみを有効化
3. `opa-wasm`の依存バージョンを「0.2.0」から「0.1」に緩和
4. OpenTelemetry関連の依存関係のバージョンを調整

```toml
# Cargo.toml
# OpenTelemetry関連
opentelemetry = { version = "0.21.0", features = ["trace"] }
opentelemetry_sdk = { version = "0.21.0", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.13.0", features = ["trace", "grpc-tonic"] }
tracing-opentelemetry = "0.20.0"
```

これにより、ビルドが成功するようになりました。ただし、OpenTelemetryによる分散トレーシング機能は一時的に無効化されています。

## 将来的な解決方法

OpenTelemetryトレーシングを再有効化する場合は、以下の手順を実施してください：

1. すべてのOpenTelemetry関連の依存関係を同一バージョンに揃える
   - すべての依存関係を最新バージョンにアップデートする
   - または、すべての依存関係を0.21.0系に固定する

2. `opentelemetry_sdk`を正しく使用するようにコードを更新する
   - `opentelemetry::sdk`の代わりに`opentelemetry_sdk`を使用する
   - `opentelemetry::runtime::Tokio`の代わりに`opentelemetry_sdk::runtime::Tokio`を使用する

3. `crates/mcp-gateway/src/tracing.rs`のコメントアウトしたセクションを新しい実装に置き換える

## 参考情報

- [OpenTelemetry Rust GitHub](https://github.com/open-telemetry/opentelemetry-rust)
- [opentelemetry-jaeger v0.22.0](https://docs.rs/crate/opentelemetry-jaeger/latest)
- [tracing-opentelemetry GitHub](https://github.com/tokio-rs/tracing-opentelemetry)

対応日: 2025年5月24日 