#[cfg(test)]
mod tests {
    use crate::models::{ExecutionRequest, SandboxConfig};
    use crate::runner::SandboxRunner;
    use std::collections::HashMap;
    use std::path::PathBuf;

    // SandboxRunner::new のテスト
    #[test]
    fn test_runner_new() {
        // インスタンスの作成のみを確認
        let _runner = SandboxRunner::new();
        assert!(true);
    }

    // 基本的なコマンド実行のテスト
    #[tokio::test]
    async fn test_run_basic_command() {
        let runner = SandboxRunner::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "echo".to_string(), "hello".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("echo", vec!["hello".to_string()]);
        
        let env = HashMap::new();
        let cwd = None;
        let timeout = 10;
        let sandbox_config = SandboxConfig::default();
        
        let request = ExecutionRequest {
            command: command.to_string(),
            args,
            env,
            cwd,
            timeout,
            sandbox_config,
        };
        
        let result = runner.run(request).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
        assert!(output.stdout.trim() == "hello");
        assert!(output.stderr.is_empty());
    }
    
    // タイムアウトするコマンドの実行テスト
    #[tokio::test]
    async fn test_run_timeout_command() {
        let runner = SandboxRunner::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("timeout", vec!["10".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("sleep", vec!["10".to_string()]);
        
        let env = HashMap::new();
        let cwd = None;
        let timeout = 1; // 1秒でタイムアウト
        let sandbox_config = SandboxConfig::default();
        
        let request = ExecutionRequest {
            command: command.to_string(),
            args,
            env,
            cwd,
            timeout,
            sandbox_config,
        };
        
        let result = runner.run(request).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("タイムアウト"));
    }
    
    // サンドボックス無効モードでのコマンド実行テスト
    #[tokio::test]
    async fn test_run_without_sandbox() {
        let runner = SandboxRunner::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "echo".to_string(), "hello".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("echo", vec!["hello".to_string()]);
        
        let env = HashMap::new();
        let cwd = None;
        let timeout = 10;
        let mut sandbox_config = SandboxConfig::default();
        sandbox_config.enabled = false;
        
        let request = ExecutionRequest {
            command: command.to_string(),
            args,
            env,
            cwd,
            timeout,
            sandbox_config,
        };
        
        let result = runner.run(request).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
    }
    
    // 環境変数を使用するコマンドのテスト
    #[tokio::test]
    async fn test_run_with_env_vars() {
        let runner = SandboxRunner::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "echo".to_string(), "%TEST_VAR%".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("sh", vec!["-c".to_string(), "echo $TEST_VAR".to_string()]);
        
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());
        
        let cwd = None;
        let timeout = 10;
        let sandbox_config = SandboxConfig::default();
        
        let request = ExecutionRequest {
            command: command.to_string(),
            args,
            env,
            cwd,
            timeout,
            sandbox_config,
        };
        
        let result = runner.run(request).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
        assert!(output.stdout.trim() == "test_value");
    }
    
    // 作業ディレクトリを指定するテスト
    #[tokio::test]
    async fn test_run_with_cwd() {
        let runner = SandboxRunner::new();
        
        #[cfg(target_os = "windows")]
        let (command, args) = ("cmd", vec!["/C".to_string(), "cd".to_string()]);
        
        #[cfg(not(target_os = "windows"))]
        let (command, args) = ("pwd", vec![]);
        
        let env = HashMap::new();
        
        #[cfg(target_os = "windows")]
        let cwd = Some(PathBuf::from("C:\\"));
        
        #[cfg(not(target_os = "windows"))]
        let cwd = Some(PathBuf::from("/tmp"));
        
        let timeout = 10;
        let sandbox_config = SandboxConfig::default();
        
        let request = ExecutionRequest {
            command: command.to_string(),
            args,
            env,
            cwd,
            timeout,
            sandbox_config,
        };
        
        let result = runner.run(request).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.exit_code.unwrap() == 0);
        
        #[cfg(target_os = "windows")]
        assert!(output.stdout.trim().contains("C:\\"));
        
        #[cfg(not(target_os = "windows"))]
        assert!(output.stdout.trim() == "/tmp");
    }
} 