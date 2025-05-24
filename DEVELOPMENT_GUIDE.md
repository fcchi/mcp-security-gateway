# MCP Security Gateway 開発ガイド

このドキュメントでは、MCP Security Gatewayプロジェクトの開発作業に関するガイドラインを提供します。

## 目次

1. [Git ワークフロー](#git-ワークフロー)
2. [ブランチ戦略](#ブランチ戦略)
3. [コミットメッセージのガイドライン](#コミットメッセージのガイドライン)
4. [プルリクエストプロセス](#プルリクエストプロセス)
5. [リリースプロセス](#リリースプロセス)
6. [コード品質](#コード品質)

## Git ワークフロー

当プロジェクトでは、GitFlow風のワークフローを採用しています。

### 初期設定

リモートリポジトリをクローンする場合：

```bash
# リポジトリをクローン
git clone https://github.com/username/mcp-security-gateway.git
cd mcp-security-gateway

# 依存関係をインストール
cargo build
```

リモートリポジトリを追加する場合：

```bash
# 既存のローカルリポジトリにリモートを追加
git remote add origin https://github.com/username/mcp-security-gateway.git

# リモートの変更をプル
git pull origin main
```

## ブランチ戦略

以下のブランチを使用しています：

- **main**: 本番環境用の安定コード
- **develop**: 次回リリース用の開発コード
- **feature/X**: 個別機能の開発 (`feature/milestone7-beta-release` など)
- **bugfix/X**: バグ修正
- **release/X.Y.Z**: リリース準備 (`release/0.2.0` など)
- **hotfix/X**: 緊急修正

### 新機能開発の場合

```bash
# developから最新の変更を取得
git checkout develop
git pull origin develop

# 機能ブランチを作成
git checkout -b feature/new-feature-name

# (開発作業)

# 変更をコミット
git add .
git commit -m "機能の追加: 〇〇機能の実装"

# developにマージする前に、最新変更を取り込む
git checkout develop
git pull origin develop
git checkout feature/new-feature-name
git rebase develop

# developにマージ
git checkout develop
git merge --no-ff feature/new-feature-name
git push origin develop

# 機能ブランチを削除（オプション）
git branch -d feature/new-feature-name
```

## コミットメッセージのガイドライン

コミットメッセージは以下のフォーマットに従ってください：

```
[種別]: 要約（50文字以内）

詳細な説明（オプション、72文字で改行）
```

種別の例:
- **機能**: 新機能
- **修正**: バグ修正
- **改善**: 既存機能の改善
- **リファクタ**: 機能変更のないコード変更
- **文書**: ドキュメントのみの変更
- **テスト**: テストの追加・修正
- **設定**: CI/CD、ビルド設定の変更

例:
```
機能: OPAポリシーエンジンとの統合

- Regoポリシーの動的読み込み機能を追加
- ポリシー評価のキャッシュ機構を実装
- エラーハンドリングの改善
```

## プルリクエストプロセス

1. 機能ブランチで開発を完了する
2. テストを実行し、コードスタイルを確認
   ```bash
   cargo test
   cargo fmt --all -- --check
   cargo clippy
   ```
3. 変更を説明するPRを作成
4. コードレビューを受ける
5. CIパイプラインが成功することを確認
6. マージ承認を得る

## リリースプロセス

1. developからリリースブランチを作成
   ```bash
   git checkout develop
   git checkout -b release/0.2.0
   ```

2. バージョン番号を更新
   - `Cargo.toml`のversion
   - CHANGELOGの更新

3. 最終テストとQA

4. mainへのマージ
   ```bash
   git checkout main
   git merge --no-ff release/0.2.0
   git tag -a v0.2.0 -m "MCP Security Gateway v0.2.0"
   git push origin main --tags
   ```

5. developへのマージ
   ```bash
   git checkout develop
   git merge --no-ff release/0.2.0
   git push origin develop
   ```

## コード品質

- コミット前に以下を実行
  ```bash
  cargo test
  cargo fmt
  cargo clippy
  ```

- コードカバレッジの確認
  ```bash
  cargo tarpaulin
  ```

- セキュリティチェック
  ```bash
  cargo audit
  ```

## CI/CD

GitHub Actionsワークフローは以下を自動化しています：

- コンパイルテスト
- 単体テスト実行
- コードスタイルチェック
- 静的解析
- セキュリティスキャン
- コードカバレッジレポート
- ドキュメント生成 