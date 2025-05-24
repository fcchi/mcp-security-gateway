package mcp.file

import future.keywords.if
import future.keywords.in

# デフォルトのルール: 拒否
default allow = false

# 読み取り可能なパスパターン
readable_paths := [
    "/workspace/",
    "/tmp/",
    "/data/public/"
]

# 書き込み可能なパスパターン
writable_paths := [
    "/workspace/",
    "/tmp/"
]

# 実行可能なパスパターン
executable_paths := [
    "/workspace/bin/",
    "/usr/bin/",
    "/bin/"
]

# アクセス禁止パスパターン
denied_paths := [
    "/etc/",
    "/var/",
    "/root/",
    "/home/"
]

# ファイルアクセスを許可するルール
allow if {
    not path_is_denied
    mode_is_allowed
}

# パスのパターンマッチをチェック
path_matches(path, pattern) if {
    startswith(path, pattern)
}

# パスが拒否リストにあるかチェック
path_is_denied if {
    some pattern in denied_paths
    path_matches(input.file.path, pattern)
}

# モードがパスに対して許可されているかチェック
mode_is_allowed if {
    input.file.mode == "read"
    some pattern in readable_paths
    path_matches(input.file.path, pattern)
}

mode_is_allowed if {
    input.file.mode == "write"
    some pattern in writable_paths
    path_matches(input.file.path, pattern)
}

mode_is_allowed if {
    input.file.mode == "execute"
    some pattern in executable_paths
    path_matches(input.file.path, pattern)
}

# 拒否理由
deny_reasons contains reason if {
    path_is_denied
    reason := sprintf("パス '%s' へのアクセスは禁止されています", [input.file.path])
}

deny_reasons contains reason if {
    not mode_is_allowed
    reason := sprintf("パス '%s' への '%s' アクセスは許可されていません", [input.file.path, input.file.mode])
}

# 警告メッセージ
warnings contains message if {
    mode_is_allowed
    input.file.mode == "write"
    message := "ファイル書き込み操作は監査されます"
} 