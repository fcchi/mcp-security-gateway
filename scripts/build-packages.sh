#!/bin/bash
set -euo pipefail

# ビルドバージョンの設定
VERSION=${1:-"0.1.0"}
ARCH="amd64"
PACKAGE_DIR="$(pwd)/packaging"
BUILD_DIR="$(pwd)/target/release"
DIST_DIR="$(pwd)/dist"

# ディレクトリチェック
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

echo "📦 MCPセキュリティゲートウェイのパッケージをビルドしています..."

# 依存関係チェック
if ! command -v fpm &> /dev/null; then
    echo "エラー: fpm が見つかりません。インストールしてください。"
    echo "gem install fpm"
    exit 1
fi

# ディレクトリ作成
mkdir -p "$PACKAGE_DIR/usr/bin"
mkdir -p "$PACKAGE_DIR/etc/mcp-security-gateway"
mkdir -p "$PACKAGE_DIR/etc/mcp-security-gateway/policies"
mkdir -p "$PACKAGE_DIR/lib/systemd/system"
mkdir -p "$PACKAGE_DIR/var/lib/mcp-security-gateway/workspace"
mkdir -p "$PACKAGE_DIR/var/log/mcp-security-gateway"
mkdir -p "$DIST_DIR"

# バイナリのコピー
if [ -f "$BUILD_DIR/mcp-gateway" ]; then
    echo "✅ バイナリが見つかりました"
    cp "$BUILD_DIR/mcp-gateway" "$PACKAGE_DIR/usr/bin/mcp-security-gateway"
    chmod +x "$PACKAGE_DIR/usr/bin/mcp-security-gateway"
else
    echo "❌ バイナリが見つかりません。先にビルドしてください"
    exit 1
fi

# 設定ファイルのコピー
cp -r config/* "$PACKAGE_DIR/etc/mcp-security-gateway/"
cp -r policies/* "$PACKAGE_DIR/etc/mcp-security-gateway/policies/"

# systemdサービスの作成
cat > "$PACKAGE_DIR/lib/systemd/system/mcp-security-gateway.service" << EOF
[Unit]
Description=MCP Security Gateway
After=network.target

[Service]
ExecStart=/usr/bin/mcp-security-gateway --config /etc/mcp-security-gateway/config.yaml
Restart=on-failure
User=mcp-gateway
Group=mcp-gateway
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# 依存パッケージ
DEBIAN_DEPENDS="bubblewrap (>= 0.4.0), libseccomp2 (>= 2.5.0), ca-certificates"
RPM_DEPENDS="bubblewrap >= 0.4.0, libseccomp >= 2.5.0, ca-certificates"

# DEB パッケージの作成
echo "🔨 debパッケージをビルド中..."
fpm -s dir -t deb \
    --name mcp-security-gateway \
    --version ${VERSION} \
    --architecture amd64 \
    --depends "bubblewrap >= 0.4.0" \
    --depends "libseccomp2 >= 2.5.0" \
    --depends "ca-certificates" \
    --maintainer "MCP Team <support@mcp-security.io>" \
    --description "MCP Security Gateway - AI/ML models and external systems secure communication adapter" \
    --license "Apache-2.0" \
    --url "https://github.com/fcchi/mcp-security-gateway" \
    --deb-systemd ${ROOT_DIR}/scripts/systemd/mcp-security-gateway.service \
    --deb-default ${ROOT_DIR}/scripts/default/mcp-security-gateway \
    --config-files /etc/mcp-security-gateway/config.yaml \
    --after-install ${ROOT_DIR}/scripts/postinst.sh \
    --before-remove ${ROOT_DIR}/scripts/prerm.sh \
    ${PACKAGE_DIR}/usr/bin/mcp-security-gateway=${PACKAGE_DIR}/usr/bin/ \
    ${PACKAGE_DIR}/etc/mcp-security-gateway/=${PACKAGE_DIR}/etc/mcp-security-gateway/ \
    ${PACKAGE_DIR}/etc/mcp-security-gateway/policies/=${PACKAGE_DIR}/etc/mcp-security-gateway/policies/

# RPM パッケージの作成
echo "🔨 rpmパッケージをビルド中..."
fpm -s dir -t rpm \
    --name mcp-security-gateway \
    --version ${VERSION} \
    --architecture x86_64 \
    --depends "bubblewrap >= 0.4.0" \
    --depends "libseccomp >= 2.5.0" \
    --depends "ca-certificates" \
    --maintainer "MCP Team <support@mcp-security.io>" \
    --description "MCP Security Gateway - AI/ML models and external systems secure communication adapter" \
    --license "Apache-2.0" \
    --url "https://github.com/fcchi/mcp-security-gateway" \
    --rpm-systemd ${ROOT_DIR}/scripts/systemd/mcp-security-gateway.service \
    --config-files /etc/mcp-security-gateway/config.yaml \
    --after-install ${ROOT_DIR}/scripts/postinst.sh \
    --before-remove ${ROOT_DIR}/scripts/prerm.sh \
    ${PACKAGE_DIR}/usr/bin/mcp-security-gateway=${PACKAGE_DIR}/usr/bin/ \
    ${PACKAGE_DIR}/etc/mcp-security-gateway/=${PACKAGE_DIR}/etc/mcp-security-gateway/ \
    ${PACKAGE_DIR}/etc/mcp-security-gateway/policies/=${PACKAGE_DIR}/etc/mcp-security-gateway/policies/

echo "✅ パッケージビルド完了"
echo "パッケージの場所:"
echo "  DEB: $DIST_DIR/mcp-security-gateway_${VERSION}_${ARCH}.deb"
echo "  RPM: $DIST_DIR/mcp-security-gateway-${VERSION}-1.${ARCH}.rpm"

# クリーンアップ
rm -rf "$PACKAGE_DIR"

echo "インストール方法:"
echo "  DEB: sudo apt install ./$DIST_DIR/mcp-security-gateway_${VERSION}_${ARCH}.deb"
echo "  RPM: sudo dnf install ./$DIST_DIR/mcp-security-gateway-${VERSION}-1.${ARCH}.rpm" 