#[cfg(test)]
mod tests {
    use crate::executor::CommandExecutor;
    use crate::models::SandboxConfig;
    use std::collections::HashMap;

    // CommandExecutor::new のテスト
    #[test]
    fn test_new() {
        // デフォルトの設定で作成
        let _executor = CommandExecutor::new();
    }

    // CommandExecutor::with_config のテスト
    #[test]
    fn test_with_config() {
        // サンドボックス設定でインスタンス作成
        let sandbox_config = SandboxConfig::default();
        let _executor = CommandExecutor::with_config(60, sandbox_config.clone());
    }

    // 有効なコマンドの実行テスト
    #[tokio::test]
    async fn test_execute_valid_command() {
        let executor = CommandExecutor::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "echo".to_string(), "hello".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("echo", vec!["hello".to_string()]);
        
        let env = HashMap::new();
        let cwd = None;
        let timeout = Some(10);
        
        let result = executor.execute(command, args, env, cwd, timeout).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
        assert!(output.stdout.trim() == "hello");
        assert!(output.stderr.is_empty());
    }
    
    // 存在しないコマンドの実行テスト
    #[tokio::test]
    async fn test_execute_invalid_command() {
        let executor = CommandExecutor::new();
        let command = "command_that_does_not_exist";
        let args = vec![];
        let env = HashMap::new();
        let cwd = None;
        let timeout = Some(10);
        
        let result = executor.execute(command, args, env, cwd, timeout).await;
        assert!(result.is_err());
    }
    
    // 環境変数を使用するコマンドのテスト
    #[tokio::test]
    async fn test_execute_with_env_vars() {
        let executor = CommandExecutor::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "echo".to_string(), "%TEST_VAR%".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("sh", vec!["-c".to_string(), "echo $TEST_VAR".to_string()]);
        
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());
        
        let cwd = None;
        let timeout = Some(10);
        
        let result = executor.execute(command, args, env, cwd, timeout).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
        assert!(output.stdout.trim() == "test_value");
    }
    
    // 作業ディレクトリを指定するテスト
    #[tokio::test]
    async fn test_execute_with_cwd() {
        let executor = CommandExecutor::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "cd".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("pwd", vec![]);
        
        let env = HashMap::new();
        
        #[cfg(target_os = "windows")]
        let cwd = Some("C:\\".to_string());
        
        #[cfg(not(target_os = "windows"))]
        let cwd = Some("/tmp".to_string());
        
        let timeout = Some(10);
        
        let result = executor.execute(command, args, env, cwd.clone(), timeout).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
        
        #[cfg(target_os = "windows")]
        assert!(output.stdout.trim().contains("C:\\"));
        
        #[cfg(not(target_os = "windows"))]
        assert!(output.stdout.trim() == "/tmp");
    }
    
    // with_sandbox_configメソッドのテスト
    #[test]
    fn test_with_sandbox_config() {
        // 既存のインスタンスから新しいサンドボックス設定で新しいインスタンスを作成
        let executor = CommandExecutor::new();
        let config = SandboxConfig::default();
        let _new_executor = executor.with_sandbox_config(config.clone());
    }
    
    // with_timeoutメソッドのテスト
    #[test]
    fn test_with_timeout() {
        // 既存のインスタンスから新しいタイムアウト設定で新しいインスタンスを作成
        let executor = CommandExecutor::new();
        let _new_executor = executor.with_timeout(120);
    }
}