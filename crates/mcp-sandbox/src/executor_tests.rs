#[cfg(test)]
mod tests {
    use crate::executor::CommandExecutor;
    use crate::models::SandboxConfig;
    use std::collections::HashMap;

    // Test for CommandExecutor::new
    #[test]
    fn test_new() {
        // Create with default settings
        let _executor = CommandExecutor::new();
    }

    // Test for CommandExecutor::with_config
    #[test]
    fn test_with_config() {
        // Create instance with sandbox configuration
        let sandbox_config = SandboxConfig::default();
        let _executor = CommandExecutor::with_config(60, sandbox_config.clone());
    }

    // Test for executing a valid command
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
    
    // Test for executing a non-existent command
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
    
    // Test for command execution with environment variables
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
    
    // Test for command execution with working directory
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
    
    // Test for with_sandbox_config method
    #[test]
    fn test_with_sandbox_config() {
        // Create a new instance with new sandbox settings from an existing instance
        let executor = CommandExecutor::new();
        let config = SandboxConfig::default();
        let _new_executor = executor.with_sandbox_config(config.clone());
    }
    
    // Test for with_timeout method
    #[test]
    fn test_with_timeout() {
        // Create a new instance with new timeout setting from an existing instance
        let executor = CommandExecutor::new();
        let _new_executor = executor.with_timeout(120);
    }
}