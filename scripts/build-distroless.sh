#!/bin/bash
set -euo pipefail

# ビルドタグの設定
TAG=${1:-latest}
REGISTRY=${REGISTRY:-localhost:5000}
IMAGE_NAME=${IMAGE_NAME:-mcp-security-gateway}
FULL_IMAGE_NAME="${REGISTRY}/${IMAGE_NAME}:${TAG}"

# ディレクトリチェック
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

echo "📦 MCPセキュリティゲートウェイのdistrolessイメージをビルドしています..."

# Dockerfileがあるか確認
if [ ! -f "$ROOT_DIR/Dockerfile.distroless" ]; then
    echo "エラー: Dockerfile.distroless が見つかりません"
    exit 1
fi

# Docker buildを実行
echo "🔨 イメージをビルド中: $FULL_IMAGE_NAME"
docker build -t "$FULL_IMAGE_NAME" -f Dockerfile.distroless .

echo "✅ ビルド完了: $FULL_IMAGE_NAME"
echo ""
echo "以下のコマンドで実行できます:"
echo "docker run --rm -p 8081:8081 $FULL_IMAGE_NAME"
echo ""
echo "イメージサイズ:"
docker images "$REGISTRY/$IMAGE_NAME" --format "{{.Repository}}:{{.Tag}} - {{.Size}}" 