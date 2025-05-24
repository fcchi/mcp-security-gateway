package mcp.command

import future.keywords.if
import future.keywords.in

# デフォルトのルール: 拒否
default allow = false

# 許可されたコマンドのリスト
allowed_commands := {
    "ls",
    "echo",
    "cat",
    "grep",
    "find",
    "python",
    "python3",
    "node",
    "npm"
}

# 危険と見なされるコマンド
dangerous_commands := {
    "rm",
    "dd",
    "wget",
    "curl",
    "chmod",
    "chown",
    "sudo",
    "su"
}

# コマンド実行を許可するルール
allow if {
    not is_dangerous_command
    input.user.roles[_] == "admin" # 管理者権限を持つユーザーは実行可能
}

allow if {
    not is_dangerous_command
    is_allowed_command
}

# 許可されたコマンドかどうかをチェック
is_allowed_command if {
    input.command.name in allowed_commands
}

# 危険なコマンドかどうかをチェック
is_dangerous_command if {
    input.command.name in dangerous_commands
}

# 拒否理由
deny_reasons contains reason if {
    is_dangerous_command
    reason := sprintf("コマンド '%s' は危険なため禁止されています", [input.command.name])
}

deny_reasons contains reason if {
    not is_allowed_command
    not input.user.roles[_] == "admin"
    reason := sprintf("コマンド '%s' は許可リストにありません", [input.command.name])
}

# 警告メッセージ
warnings contains message if {
    input.user.roles[_] == "admin"
    is_allowed_command
    message := "管理者として実行中。全ての操作が監査されます。"
} 