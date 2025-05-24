# MCP Security Gateway

MCPセキュリティゲートウェイは、AI/MLモデルと外部システム間の安全な通信を実現するためのセキュリティアダプターです。サンドボックス化されたコマンド実行、ファイルアクセス制御、セキュリティポリシー適用を提供します。

## 主な機能

- **安全なコマンド実行**: bubblewrap, seccompによるサンドボックス化
- **ポリシーベースの制御**: OPA (Rego)によるきめ細かなセキュリティポリシー
- **監査とトレース**: 完全な監査ログと可観測性
- **高可用性**: Active-Active / Active-Passiveトポロジをサポート
- **マルチインターフェース**: REST API / gRPC / クライアントライブラリ

## クイックスタート

### 必要なもの

- Docker と Docker Compose
- Rust (開発時のみ、1.73以上を推奨)
- Protocol Buffers コンパイラ (protoc)

### Protocol Buffersコンパイラのインストール

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y protobuf-compiler
```

**macOS:**
```bash
brew install protobuf
```

**Windows:**
1. [Protocol Buffers リリースページ](https://github.com/protocolbuffers/protobuf/releases)から、最新のWindows用リリース（例：`protoc-25.2-win64.zip`）をダウンロード
2. ダウンロードしたZIPファイルを任意のフォルダに解凍
3. 解凍したフォルダの`bin`ディレクトリ（例：`C:\protoc\bin`）を環境変数PATHに追加
4. コマンドプロンプトまたはPowerShellを再起動
5. `protoc --version`コマンドでインストールを確認

**検証方法:**
以下のコマンドで正しくインストールされているか確認できます：
```bash
protoc --version
```

### Dockerを使ったクイックスタート

リポジトリをクローンして、Docker Composeで起動できます：

```bash
# リポジトリをクローン
git clone https://github.com/fcchi/mcp-security-gateway.git
cd mcp-security-gateway

# Docker Composeで起動
docker-compose up -d
```

サービスが起動したら、以下のURLでヘルスチェックができます：

```
http://localhost:8081/health
```

### セキュアなdistrolessイメージの使用

本番環境ではセキュリティを強化するために、distrolessイメージの使用を推奨します：

```bash
# distrolessイメージを使用したDockerビルド
./scripts/build-distroless.sh

# または直接実行
docker run -d --name mcp-gateway \
  --cap-add SYS_ADMIN \
  --security-opt seccomp=unconfined \
  -p 8081:8081 \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/policies:/app/policies \
  -v $(pwd)/workspace:/workspace \
  ghcr.io/fcchi/mcp-security-gateway:latest
```

distrolessイメージは攻撃対象領域を最小限に抑え、シェルやデバッグツールを含まないため、セキュリティリスクを大幅に低減します。詳細は[デプロイガイド](docs/operations/DEPLOY_GUIDE.md#3.2-distrolessイメージの利用)を参照してください。

### ローカル開発環境のセットアップ

```bash
# リポジトリをクローン
git clone https://github.com/fcchi/mcp-security-gateway.git
cd mcp-security-gateway

# 依存関係をインストール
# Linux (Ubuntu/Debian):
sudo apt-get install -y protobuf-compiler bubblewrap libseccomp-dev

# macOS:
brew install protobuf

# Windows:
# Protocol Buffersコンパイラのインストール手順に従ってインストール

# ビルド
cargo build

# 単体テストの実行
cargo test

# 実行
cargo run -- serve --host 127.0.0.1 --port 8081
```

### 基本的な使い方

#### コマンド実行APIの例

```bash
# gRPCurlを使用した例（インストールが必要）
grpcurl -plaintext -d '{
  "command": "echo",
  "args": ["hello world"],
  "timeout": 30
}' localhost:8081 mcp.McpService/ExecuteCommand

# タスク状態の確認
grpcurl -plaintext -d '{"task_id": "task-xxxxx"}' localhost:8081 mcp.McpService/GetTaskStatus
```

## ドキュメント

### アーキテクチャ

- [アーキテクチャ概要](docs/architecture/ARCHITECTURE_OVERVIEW.md) - システム全体構成と要点
- [ユーザーガイド](docs/architecture/USER_GUIDE.md) - 基本的な使い方と操作方法

### API

- [MCP プロトコル仕様](docs/api/MCP_PROTOCOL.md) - プロトコルの詳細
- [API リファレンス](docs/api/API_REFERENCE.md) - APIエンドポイントと機能

### セキュリティ

- [セキュリティ機能](docs/security/SECURITY_FEATURES.md) - セキュリティモデルと機能
- [脅威モデル](docs/security/THREAT_MODEL.md) - セキュリティリスクと対策

### 運用

- [デプロイガイド](docs/operations/DEPLOY_GUIDE.md) - インストールと設定
- [運用手順書](docs/operations/OPERATIONS_RUNBOOK.md) - 運用とメンテナンス
- [パフォーマンスSLO](docs/operations/PERFORMANCE_SLO.md) - サービスレベル目標

### 品質

- [エラー処理](docs/quality/ERROR_HANDLING.md) - エラー分類と対応
- [テスト戦略](docs/quality/TEST_STRATEGY.md) - テスト手法とカバレッジ

### 可観測性

- [可観測性](docs/observability/OBSERVABILITY.md) - ログ、メトリクス、トレース

### メタ情報

- [用語集](docs/meta/GLOSSARY.md) - 主要用語と定義
- [変更履歴](docs/meta/CHANGELOG.md) - バージョン変更記録
- [バージョニングとアップグレード](docs/meta/VERSIONING_UPGRADE.md) - バージョン管理とアップグレード手順
- [実装ロードマップ](docs/meta/ROADMAP.md) - 実装計画とマイルストーン

## クロスリファレンス

| カテゴリ | 関連するドキュメント | 関連する指標/アラート |
|---------|-------------------|---------------------|
| エラーコード | [MCP_PROTOCOL.md](docs/api/MCP_PROTOCOL.md), [ERROR_HANDLING.md](docs/quality/ERROR_HANDLING.md) | MCPHighErrorRate |
| パフォーマンス | [PERFORMANCE_SLO.md](docs/operations/PERFORMANCE_SLO.md), [TEST_STRATEGY.md](docs/quality/TEST_STRATEGY.md) | MCPLatencyP99, MCPThroughputAPI |
| セキュリティ | [SECURITY_FEATURES.md](docs/security/SECURITY_FEATURES.md), [DEPLOY_GUIDE.md](docs/operations/DEPLOY_GUIDE.md) | MCPHighPolicyViolationRate |
| 可用性 | [ARCHITECTURE_OVERVIEW.md](docs/architecture/ARCHITECTURE_OVERVIEW.md), [OPERATIONS_RUNBOOK.md](docs/operations/OPERATIONS_RUNBOOK.md) | MCPAvailabilityProd |

## 貢献

貢献いただける方は、[CONTRIBUTING.md](CONTRIBUTING.md)を参照してください。すべてのコントリビューターに[行動規範](CODE_OF_CONDUCT.md)の遵守をお願いしています。

## 開発状況

現在の開発状況:

- ✅ マイルストーン0（プロジェクト雛形）: 基本的なRustワークスペース構成の完了
  - ✅ リポジトリ構造とワークスペース設定
  - ✅ 基本的なgRPCサーバー実装
  - ✅ GitHub Actions CI設定
  - ✅ コード品質ゲート（rustfmt & clippy）
  - ✅ CONTRIBUTING & ISSUE_TEMPLATE
  - ✅ レビューと最終調整
- ✅ マイルストーン1（Core MVP α）: 基本機能の実装完了
  - ✅ gRPCサーバーの基本実装
  - ✅ タスク実行プロトコル定義とスタブ実装
  - ✅ コマンド実行アダプター
  - ✅ 構造化ログ出力
  - ✅ 単体テスト
  - ✅ クイックスタート設定
- ✅ マイルストーン2（Policy & Security）: 完了
  - ✅ OPA統合（Regoポリシー）の基本実装
  - ✅ bubblewrapサンドボックスプロファイルの実装
  - ✅ エラーコードとgRPC statusのマッピング
  - ✅ 共通Result<T, McpError>ヘルパーの実装
  - ✅ セキュリティドキュメントの更新
  - ✅ グローバルエラーハンドラーの実装
  - ✅ レビューと最終調整
- ✅ マイルストーン3（Observability Stack）: 完了
  - ✅ Prometheusヒストグラムとして公開（task_latency_ms）
  - ✅ OTLP トレースエクスポーター追加
  - ✅ e2eスクリプト（pexpect）によるlsフロー検証
  - ✅ Grafanaダッシュボード設定（overview, performance）
  - ✅ ドキュメントのmkdocsビルドとリンクチェック（CI）
  - ✅ マイルストーン3の最終レビューと調整
- ✅ マイルストーン4（CI Gate & Error Strategy）: 完了
  - ✅ Trivy & cargo-audit SCA step追加（T040）
  - ✅ カバレッジゲート追加（T041）
  - ✅ McpError ↔ gRPC status mapping matrix（T042）
  - ✅ ERROR_HANDLING link from API_REFERENCE（T043）
  - ✅ action: Release Drafter 設定（T044）
  - ✅ レビュー & バグ修正（M4）（T045）
- ✅ マイルストーン5（Packaging & Deployment）: 完了
  - ✅ create distroless Dockerfile + ko build（T050）
  - ✅ build .deb & .rpm via fpm（T051）
  - ✅ create chart mcp-gateway（T052）
  - ✅ DEPLOY_GUIDE update (helm + deb)（T053）
  - ✅ action: MkDocs-deploy (GitHub Pages)（T054）
  - ✅ レビュー & バグ修正（M5）（T055）
- ✅ マイルストーン6（Performance & SLO Validation）: 完了
  - ✅ locust script 100 RPS × 5 min baseline（T060）
  - ✅ Prometheus alert rule SLO_violation（T061）
  - ✅ PERFORMANCE_SLO link to alert rule（T062）
  - ✅ action: nightly dependency-update w/ Renovate（T063）
  - ✅ レビュー & バグ修正（M6）（T064）

詳細な進捗状況は[実装ロードマップ](docs/meta/ROADMAP.md)と[変更履歴](docs/meta/CHANGELOG.md)を参照してください。

## 次のステップ

優先して取り組む予定のタスク:

1. マイルストーン7（Release β & Docs Polish）の実装開始
   - bump to v0.2.0 + CHANGELOG entry（T070）
   - README badges (CI, Coverage, Go-Report)（T071）
   - USER_GUIDE (旧 ARCH_OVERVIEW.md) final polish（T072）
   - GitHub Release draft w/ binaries & checksums（T073）

## ライセンス

Apache License 2.0 - 詳細は[LICENSE](LICENSE)ファイルを参照してください。 