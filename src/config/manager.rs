use std::fs;
use std::path::Path;

use crate::config::{Config, GlobalConfig, ProcessConfig};
use crate::error::{JanusError, Result};

#[derive(Debug)]
pub struct ConfigManager {
    config_path: String,
    config: Config,
}

impl ConfigManager {
    pub fn new(config_path: &str) -> Result<Self> {
        let config = Self::load_config(config_path)?;
        
        let manager = Self {
            config_path: config_path.to_string(),
            config,
        };
        
        manager.validate()?;
        
        Ok(manager)
    }
    
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_config(&self.config_path)?;
        self.validate()?;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<()> {
        // 檢查進程名稱唯一性
        let mut names = std::collections::HashSet::new();
        
        for process in &self.config.process {
            if !names.insert(&process.name) {
                return Err(JanusError::Config(format!(
                    "Duplicate process name: {}",
                    process.name
                )));
            }
            
            // 檢查命令非空
            if process.command.trim().is_empty() {
                return Err(JanusError::Config(format!(
                    "Empty command for process: {}",
                    process.name
                )));
            }
        }
        
        Ok(())
    }
    
    pub fn get_process_configs(&self) -> &[ProcessConfig] {
        &self.config.process
    }
    
    pub fn get_global_config(&self) -> &GlobalConfig {
        &self.config.global
    }
    
    fn load_config(config_path: &str) -> Result<Config> {
        let config_str = fs::read_to_string(config_path).map_err(|e| {
            JanusError::Config(format!("Failed to read config file: {}", e))
        })?;
        
        let config: Config = toml::from_str(&config_str).map_err(|e| {
            JanusError::Config(format!("Failed to parse TOML: {}", e))
        })?;
        
        Ok(config)
    }
}
