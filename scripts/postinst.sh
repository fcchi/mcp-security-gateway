#!/bin/bash
set -e

# ユーザーとグループの作成
if ! getent group mcp-gateway > /dev/null; then
    groupadd -r mcp-gateway
fi

if ! getent passwd mcp-gateway > /dev/null; then
    useradd -r -g mcp-gateway -d /var/lib/mcp-security-gateway -s /bin/false -c "MCP Security Gateway" mcp-gateway
fi

# ディレクトリの所有権設定
chown -R mcp-gateway:mcp-gateway /var/lib/mcp-security-gateway
chown -R mcp-gateway:mcp-gateway /var/log/mcp-security-gateway
chown -R root:mcp-gateway /etc/mcp-security-gateway
chmod 750 /etc/mcp-security-gateway
chmod 640 /etc/mcp-security-gateway/config.yaml

# systemdの場合はサービスを有効化
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload >/dev/null 2>&1 || true
    systemctl enable mcp-security-gateway.service >/dev/null 2>&1 || true
    echo "MCPセキュリティゲートウェイをsystemdサービスとして有効化しました"
    echo "サービスを開始するには以下のコマンドを実行してください："
    echo "  sudo systemctl start mcp-security-gateway"
fi

echo "インストールが完了しました！" 