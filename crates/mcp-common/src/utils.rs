use crate::error::{McpError, McpResult};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// 現在のUNIXタイムスタンプをミリ秒単位で取得
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

/// ISO 8601形式の現在時刻文字列を取得
pub fn current_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// ユニークなタスクIDを生成
pub fn generate_task_id() -> String {
    format!("task-{}", Uuid::new_v4().simple())
}

/// スライスを指定サイズのチャンクに分割するイテレータを返す
pub fn chunk_slice<T>(slice: &[T], chunk_size: usize) -> impl Iterator<Item = &[T]> {
    slice.chunks(chunk_size)
}

/// 文字列を省略（最大長を超える場合は末尾を "..." で置換）
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        // 極端に短い場合はドットのみ
        ".".repeat(max_len)
    } else {
        format!("{}...", &s[0..max_len.saturating_sub(3)])
    }
}

/// 安全なJSONパース（エラーをMcpError::InvalidRequestに変換）
pub fn parse_json<T>(json_str: &str) -> McpResult<T> 
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(json_str)
        .map_err(|e| McpError::InvalidRequest(format!("JSONのパースに失敗しました: {}", e)))
}

/// 環境変数の取得（存在しない場合はMcpError::Internalを返す）
pub fn get_env_var(name: &str) -> McpResult<String> {
    std::env::var(name)
        .map_err(|_| McpError::Internal(format!("環境変数 {} が設定されていません", name)))
}

/// 環境変数の取得（デフォルト値付き）
pub fn get_env_var_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("abc", 2), ".."); // 極端な短さのケース
    }

    #[test]
    fn test_parse_json() {
        #[derive(serde::Deserialize)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let json = r#"{"name": "test", "value": 42}"#;
        let result: McpResult<TestStruct> = parse_json(json);
        assert!(result.is_ok());
        
        let test_struct = result.unwrap();
        assert_eq!(test_struct.name, "test");
        assert_eq!(test_struct.value, 42);

        let invalid_json = r#"{"name": "test", "value": "not a number"}"#;
        let result: McpResult<TestStruct> = parse_json(invalid_json);
        assert!(result.is_err());
    }
} 