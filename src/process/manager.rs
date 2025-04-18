use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::config::{manager::ConfigManager, GlobalConfig, ProcessConfig};
use crate::error::{ErrorType, JanusError, Result};
use crate::logging::handler::LogHandler;
use crate::logging::LogType;
use crate::process::{ProcessStatus, runner::ProcessRunner};
use crate::error::handler::ErrorHandler;

pub struct ManagedProcess {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub env: HashMap<String, String>,
    pub auto_restart: bool,
    pub restart_limit: Option<u32>,
    pub restart_delay: u64,
    pub process: Option<Child>,
    pub status: ProcessStatus,
    pub restart_count: u32,
    pub start_time: Option<Instant>,
}

impl ManagedProcess {
    pub fn from_config(
        process_config: &ProcessConfig,
        global_config: &GlobalConfig,
    ) -> Self {
        // 合併環境變量
        let mut env = HashMap::new();
        if let Some(global_env) = &global_config.env {
            env.extend(global_env.clone());
        }
        if let Some(process_env) = &process_config.env {
            env.extend(process_env.clone());
        }

        // 確定工作目錄
        let working_dir = process_config
            .working_dir
            .clone()
            .or_else(|| global_config.working_dir.clone());

        Self {
            name: process_config.name.clone(),
            command: process_config.command.clone(),
            args: process_config.args.clone().unwrap_or_default(),
            working_dir,
            env,
            auto_restart: process_config.auto_restart.unwrap_or(false),
            restart_limit: process_config.restart_limit,
            restart_delay: process_config.restart_delay.unwrap_or(1),
            process: None,
            status: ProcessStatus::Stopped,
            restart_count: 0,
            start_time: None,
        }
    }
}

pub struct ProcessManager {
    processes: HashMap<String, ManagedProcess>,
    config_manager: ConfigManager,
    log_handler: LogHandler,
    error_handler: ErrorHandler,
}

impl ProcessManager {
    pub fn new(config_manager: ConfigManager, log_handler: LogHandler) -> Self {
        let error_handler = ErrorHandler::new(log_handler.clone());
        
        let mut manager = Self {
            processes: HashMap::new(),
            config_manager,
            log_handler,
            error_handler,
        };
        
        // 從配置初始化進程
        manager.init_processes();
        
        manager
    }
    
    fn init_processes(&mut self) {
        let global_config = self.config_manager.get_global_config();
        
        for process_config in self.config_manager.get_process_configs() {
            let managed_process = ManagedProcess::from_config(process_config, global_config);
            self.processes.insert(managed_process.name.clone(), managed_process);
        }
    }
    
    pub fn start_all(&mut self) -> Result<()> {
        let process_runner = ProcessRunner::new(self);
        
        for name in self.processes.keys().cloned().collect::<Vec<_>>() {
            if let Err(e) = process_runner.start_process(&name) {
                self.log_handler.log(
                    &name,
                    LogType::System,
                    &format!("Failed to start process: {}", e),
                );
            }
        }
        
        Ok(())
    }
    
    pub fn stop_all(&mut self) -> Result<()> {
        for (name, process) in &mut self.processes {
            if let Some(child) = &mut process.process {
                self.log_handler.log(name, LogType::System, "Stopping process");
                
                if let Err(e) = child.kill() {
                    self.log_handler.log(
                        name,
                        LogType::System,
                        &format!("Failed to kill process: {}", e),
                    );
                }
                
                process.status = ProcessStatus::Stopped;
                process.process = None;
            }
        }
        
        Ok(())
    }
    
    pub fn restart_process(&mut self, name: &str) -> Result<()> {
        let process_runner = ProcessRunner::new(self);
        
        // 先停止
        if let Some(process) = self.processes.get_mut(name) {
            if let Some(child) = &mut process.process {
                self.log_handler.log(name, LogType::System, "Stopping process for restart");
                
                if let Err(e) = child.kill() {
                    self.log_handler.log(
                        name,
                        LogType::System,
                        &format!("Failed to kill process: {}", e),
                    );
                }
                
                process.status = ProcessStatus::Stopped;
                process.process = None;
            }
        } else {
            return Err(JanusError::Process(format!("Process not found: {}", name)));
        }
        
        // 再啟動
        process_runner.start_process(name)?;
        
        Ok(())
    }
    
    pub fn get_process_status(&self, name: &str) -> Option<ProcessStatus> {
        self.processes.get(name).map(|p| p.status.clone())
    }
    
    pub fn get_all_processes(&self) -> &HashMap<String, ManagedProcess> {
        &self.processes
    }
    
    pub fn get_process_mut(&mut self, name: &str) -> Option<&mut ManagedProcess> {
        self.processes.get_mut(name)
    }
    
    pub fn get_log_handler(&self) -> &LogHandler {
        &self.log_handler
    }
    
    pub fn get_error_handler(&self) -> &ErrorHandler {
        &self.error_handler
    }
}
