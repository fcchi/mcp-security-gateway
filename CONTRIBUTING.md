# MCPセキュリティゲートウェイへの貢献

MCPセキュリティゲートウェイプロジェクトへの貢献に関心をお持ちいただき、ありがとうございます。このドキュメントでは、貢献プロセスについて説明します。

## 行動規範

このプロジェクトは[行動規範](CODE_OF_CONDUCT.md)に従います。プロジェクトに参加することで、この規範に従うことに同意したものとみなします。

## 開発環境のセットアップ

1. リポジトリをクローンする：
   ```bash
   git clone https://github.com/your-username/mcp-security-gateway.git
   cd mcp-security-gateway
   ```

2. 依存関係をインストールする：
   - **Linux (Ubuntu/Debian):**
     ```bash
     sudo apt-get update
     sudo apt-get install -y protobuf-compiler bubblewrap libseccomp-dev
     ```
   - **Fedora/CentOS:**
     ```bash
     sudo dnf install -y protobuf-compiler bubblewrap libseccomp-devel
     ```
   - **macOS:**
     ```bash
     brew install protobuf
     ```
   - **Windows:**
     1. [Protocol Buffers リリースページ](https://github.com/protocolbuffers/protobuf/releases)から、最新のWindows用リリース（例：`protoc-25.2-win64.zip`）をダウンロード
     2. ダウンロードしたZIPファイルを任意のフォルダに解凍
     3. 解凍したフォルダの`bin`ディレクトリ（例：`C:\protoc\bin`）を環境変数PATHに追加
     4. コマンドプロンプトまたはPowerShellを再起動
     5. `protoc --version`コマンドでインストールを確認

3. ビルドとテスト：
   ```bash
   cargo build
   cargo test
   ```

## 貢献のプロセス

1. [GitHub Issues](https://github.com/your-username/mcp-security-gateway/issues)で既存の問題を確認するか、新しい問題を作成します。
2. リポジトリをフォークし、ローカルにクローンします。
3. 機能ブランチを作成します：`git checkout -b feature/your-feature-name`
4. 変更を加え、テストが通ることを確認してください。
5. コードスタイルを確認します：`cargo fmt -- --check` および `cargo clippy -- -D warnings`
6. 変更をコミットし、プッシュします：`git push origin feature/your-feature-name`
7. プルリクエストを作成します。

## コーディング規約

- Rustの標準スタイルに従います（`cargo fmt`を使用）
- 警告は解決してください（`cargo clippy -- -D warnings`でチェック）
- 新機能には単体テストを追加してください（カバレッジ目標 ≥ 80%）
- ドキュメントコメントを追加してください（公開API/関数には必須）

## コミットメッセージの規約

コミットメッセージは以下の形式に従ってください：
```
[分類]: 短い説明 (50文字以内)

より詳細な説明文 (必要な場合)。72文字で改行。

関連するIssueを明記：fixes #123
```

分類の例：
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメントのみの変更
- `test`: テストのみの変更
- `refactor`: リファクタリング（機能変更なし）
- `style`: コードスタイルの変更（空白、フォーマットなど）
- `chore`: ビルドプロセスなどの変更

## プルリクエストのレビュープロセス

1. CI通過：GitHub Actionsがパスすることを確認
2. レビュー承認：少なくとも1人のメンテナからのレビュー承認が必要
3. マージ：すべての要件が満たされたらマージされます

## リリースプロセス

リリースは[セマンティックバージョニング](https://semver.org/lang/ja/)に従います：
- パッチリリース（1.0.x）：バグ修正のみ
- マイナーリリース（1.x.0）：下位互換性のある機能追加
- メジャーリリース（x.0.0）：下位互換性のない変更

## 質問や困ったことがある場合

質問がある場合は、issueを作成するか、[ディスカッション](https://github.com/your-username/mcp-security-gateway/discussions)セクションに投稿してください。

ご協力ありがとうございます！ 