package mcp.network

import future.keywords.if
import future.keywords.in

# デフォルトのルール: 拒否
default allow = false

# 許可されたホスト
allowed_hosts := {
    "api.example.com",
    "cdn.example.com",
    "data.example.com"
}

# 許可されたポート
allowed_ports := {
    80,
    443,
    8080
}

# 許可されたプロトコル
allowed_protocols := {
    "tcp",
    "https"
}

# ネットワークアクセスを許可するルール
allow if {
    host_is_allowed
    port_is_allowed
    protocol_is_allowed
}

# ホストが許可リストにあるかチェック
host_is_allowed if {
    input.network.host in allowed_hosts
}

# ポートが許可リストにあるかチェック
port_is_allowed if {
    input.network.port in allowed_ports
}

# プロトコルが許可リストにあるかチェック
protocol_is_allowed if {
    input.network.protocol in allowed_protocols
}

# 拒否理由
deny_reasons contains reason if {
    not host_is_allowed
    reason := sprintf("ホスト '%s' へのアクセスは許可されていません", [input.network.host])
}

deny_reasons contains reason if {
    not port_is_allowed
    reason := sprintf("ポート %d へのアクセスは許可されていません", [input.network.port])
}

deny_reasons contains reason if {
    not protocol_is_allowed
    reason := sprintf("プロトコル '%s' の使用は許可されていません", [input.network.protocol])
}

# 警告メッセージ
warnings contains message if {
    host_is_allowed
    port_is_allowed
    protocol_is_allowed
    message := "ネットワークリクエストは監査されます"
} 