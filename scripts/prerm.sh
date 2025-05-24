#!/bin/bash
set -e

# systemdサービスの停止と無効化
if [ -d /run/systemd/system ]; then
    systemctl --no-reload disable mcp-security-gateway.service >/dev/null 2>&1 || true
    systemctl stop mcp-security-gateway.service >/dev/null 2>&1 || true
fi

echo "サービスを停止しました" 