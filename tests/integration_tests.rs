#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;
    
    #[test]
    fn test_config_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let config_content = r#"
        [global]
        log_level = "debug"
        
        [[process]]
        name = "test-process"
        command = "echo"
        args = ["Hello, World!"]
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let config_manager = janus::config::manager::ConfigManager::new(
            config_path.to_str().unwrap(),
        )
        .unwrap();
        
        assert_eq!(
            config_manager.get_global_config().log_level.as_deref().unwrap(),
            "debug"
        );
        
        let processes = config_manager.get_process_configs();
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].name, "test-process");
        assert_eq!(processes[0].command, "echo");
        assert_eq!(processes[0].args.as_ref().unwrap(), &vec!["Hello, World!".to_string()]);
    }
    
    #[test]
    fn test_duplicate_process_names() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid_config.toml");
        
        let config_content = r#"
        [[process]]
        name = "test-process"
        command = "echo"
        args = ["First"]
        
        [[process]]
        name = "test-process"
        command = "echo"
        args = ["Second"]
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let result = janus::config::manager::ConfigManager::new(
            config_path.to_str().unwrap(),
        );
        
        assert!(result.is_err());
    }
    
    // 注意：以下測試需要實際運行進程，可能需要在 CI 環境中特別處理
    #[test]
    #[ignore]
    fn test_process_lifecycle() {
        // 創建臨時配置文件
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("lifecycle_config.toml");
        
        // 使用 sleep 命令作為測試進程
        let config_content = r#"
        [[process]]
        name = "sleep-process"
        command = "sleep"
        args = ["10"]
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // 啟動進程
        let status = Command::new("cargo")
            .args(&["run", "--", "--config", config_path.to_str().unwrap(), "start"])
            .status()
            .unwrap();
        
        assert!(status.success());
        
        // 檢查進程狀態
        let output = Command::new("cargo")
            .args(&["run", "--", "--config", config_path.to_str().unwrap(), "status"])
            .output()
            .unwrap();
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(output_str.contains("RUNNING"));
        
        // 停止進程
        let status = Command::new("cargo")
            .args(&["run", "--", "--config", config_path.to_str().unwrap(), "stop"])
            .status()
            .unwrap();
        
        assert!(status.success());
        
        // 再次檢查狀態
        let output = Command::new("cargo")
            .args(&["run", "--", "--config", config_path.to_str().unwrap(), "status"])
            .output()
            .unwrap();
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(output_str.contains("STOPPED"));
    }
}
