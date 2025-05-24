package mcp

import data.mcp.command
import data.mcp.file
import data.mcp.network
import future.keywords.if

# デフォルトの決定値
default allow = false
default deny_reasons = []
default warnings = []

# メインの許可ルール
allow if {
    # 実行するタスクタイプに基づいて適切なポリシーを適用
    task_type := get_task_type
    
    # コマンド実行ポリシー
    task_type == "command"
    command.allow
}

allow if {
    # ファイルアクセスポリシー
    task_type := get_task_type
    task_type == "file"
    file.allow
}

allow if {
    # ネットワークアクセスポリシー
    task_type := get_task_type
    task_type == "network"
    network.allow
}

# タスクタイプを判断
get_task_type = "command" if {
    input.command.name != ""
}

get_task_type = "file" if {
    input.file != null
    input.file.path != ""
}

get_task_type = "network" if {
    input.network != null
    input.network.host != ""
}

# 拒否理由の集約
deny_reasons = reasons if {
    task_type := get_task_type
    
    task_type == "command"
    reasons := command.deny_reasons
} else = reasons if {
    task_type := get_task_type
    
    task_type == "file"
    reasons := file.deny_reasons
} else = reasons if {
    task_type := get_task_type
    
    task_type == "network"
    reasons := network.deny_reasons
} else = ["不明なタスクタイプ"]

# 警告メッセージの集約
warnings = msgs if {
    task_type := get_task_type
    
    task_type == "command"
    msgs := command.warnings
} else = msgs if {
    task_type := get_task_type
    
    task_type == "file"
    msgs := file.warnings
} else = msgs if {
    task_type := get_task_type
    
    task_type == "network"
    msgs := network.warnings
} else = [] 