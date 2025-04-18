pub mod manager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalConfig {
    pub working_dir: Option<String>,
    pub log_level: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            working_dir: None,
            log_level: Some("info".to_string()),
            env: Some(HashMap::new()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessConfig {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub auto_restart: Option<bool>,
    pub restart_limit: Option<u32>,
    pub restart_delay: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,
    pub process: Vec<ProcessConfig>,
}
