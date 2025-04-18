use std::fs;

use crate::config::{Config, GlobalConfig, ProcessConfig};
use crate::error::{JanusError, Result};

#[derive(Debug)]
pub struct ConfigManager {
    config: Config,
}

impl ConfigManager {
    pub fn new(config_path: &str) -> Result<Self> {
        let config = Self::load_config(config_path)?;
        
        let manager = Self {
            config,
        };
        
        manager.validate()?;
        
        Ok(manager)
    }
    
    pub fn validate(&self) -> Result<()> {
        let mut names = std::collections::HashSet::new();
        
        for process in &self.config.process {
            if !names.insert(&process.name) {
                return Err(JanusError::Config(format!(
                    "Duplicate process name: {}",
                    process.name
                )));
            }
            
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
        let config_content = match fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                return Err(JanusError::Config(format!(
                    "Failed to read config file: {}",
                    e
                )))
            }
        };
        
        match toml::from_str::<Config>(&config_content) {
            Ok(config) => Ok(config),
            Err(e) => Err(JanusError::Config(format!("Failed to parse config file: {}", e))),
        }
    }
}
