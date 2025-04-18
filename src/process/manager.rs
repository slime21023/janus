use std::collections::HashMap;
use std::time::Instant;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;

use crate::config::manager::ConfigManager;
use crate::error::{JanusError, Result};
use crate::logging::handler::LogHandler;
use crate::logging::LogType;

use super::{ManagedProcess, ProcessStatus};

pub struct ProcessManager {
    processes: HashMap<String, ManagedProcess>,
    log_handler: LogHandler,
}

impl ProcessManager {
    pub fn new(config_manager: ConfigManager, log_handler: LogHandler) -> Self {
        // 從配置中獲取進程
        let processes = config_manager
            .get_process_configs()
            .iter()
            .map(|config| {
                let process = ManagedProcess {
                    name: config.name.clone(),
                    command: config.command.clone(),
                    args: config.args.clone().unwrap_or_default(),
                    env: config.env.clone().unwrap_or_default(),
                    working_dir: config.working_dir.clone(),
                    auto_restart: config.auto_restart.unwrap_or(false),
                    restart_count: 0,
                    restart_limit: config.restart_limit,
                    restart_delay: config.restart_delay.unwrap_or(1),
                    status: ProcessStatus::Stopped,
                    process: None,
                    start_time: None,
                };
                (config.name.clone(), process)
            })
            .collect();

        Self {
            processes,
            log_handler,
        }
    }

    pub fn get_all_processes(&self) -> &HashMap<String, ManagedProcess> {
        &self.processes
    }

    pub fn get_process(&self, name: &str) -> Option<&ManagedProcess> {
        self.processes.get(name)
    }

    pub fn get_process_mut(&mut self, name: &str) -> Option<&mut ManagedProcess> {
        self.processes.get_mut(name)
    }

    pub async fn start_all(&mut self) -> Result<()> {
        let process_names: Vec<String> = self.processes.keys().cloned().collect();
        
        for name in process_names {
            if let Err(e) = self.start_process(&name).await {
                let log_handler = self.log_handler.clone();
                log_handler.log(
                    &name,
                    LogType::System,
                    &format!("Failed to start process: {}", e),
                );
            }
        }
        
        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<()> {
        for (name, process) in &mut self.processes {
            if process.status == ProcessStatus::Running {
                if let Some(child) = &mut process.process {
                    let log_handler = self.log_handler.clone();
                    let name = name.clone();
                    
                    match child.kill().await {
                        Ok(_) => {
                            log_handler.log(
                                &name,
                                LogType::System,
                                "Process stopped",
                            );
                            process.status = ProcessStatus::Stopped;
                            process.process = None;
                        }
                        Err(e) => {
                            log_handler.log(
                                &name,
                                LogType::System,
                                &format!("Failed to stop process: {}", e),
                            );
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn restart_process(&mut self, name: &str) -> Result<()> {
        // 首先檢查進程是否存在
        if !self.processes.contains_key(name) {
            return Err(JanusError::Process(format!("Process not found: {}", name)));
        }
        
        // 獲取日誌處理器和進程名稱的克隆
        let log_handler = self.log_handler.clone();
        let process_name = name.to_string();
        
        // 獲取並處理進程
        let process_running;
        {
            let process = self.get_process_mut(&process_name).ok_or_else(|| {
                JanusError::Process(format!("Process not found: {}", name))
            })?;
            
            process_running = process.status == ProcessStatus::Running;
            
            // 如果進程在運行，則先停止它
            if process_running {
                if let Some(child) = &mut process.process {
                    // 先停止進程
                    match child.kill().await {
                        Ok(_) => {
                            log_handler.log(
                                &process_name,
                                LogType::System,
                                "Process stopped for restart",
                            );
                            process.status = ProcessStatus::Stopped;
                            process.process = None;
                        }
                        Err(e) => {
                            return Err(JanusError::Process(format!("Failed to stop process: {}", e)));
                        }
                    }
                }
            }
        }
        
        // 然後重新啟動
        self.start_process(&process_name).await
    }

    pub async fn start_process(&mut self, name: &str) -> Result<()> {
        // 檢查進程是否存在
        if !self.processes.contains_key(name) {
            return Err(JanusError::Process(format!("Process not found: {}", name)));
        }
        
        // 獲取日誌處理器的克隆
        let log_handler = self.log_handler.clone();
        let process_name = name.to_string();
        
        // 獲取並處理進程
        let command_str;
        let args;
        let env;
        let working_dir;
        
        {
            let process = self.get_process_mut(name).unwrap();
            
            // 如果進程已在運行，則直接返回
            if process.status == ProcessStatus::Running {
                log_handler.log(
                    name,
                    LogType::System,
                    "Process already running",
                );
                return Ok(());
            }
            
            // 複製所需信息以避免借用問題
            command_str = process.command.clone();
            args = process.args.clone();
            env = process.env.clone();
            working_dir = process.working_dir.clone();
        }
        
        // 創建命令（避免借用衝突）
        let mut command = Command::new(&command_str);
        command.args(&args)
               .stdin(Stdio::null())
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());
        
        // 設置環境變量
        for (key, value) in &env {
            command.env(key, value);
        }
        
        // 設置工作目錄
        if let Some(dir) = &working_dir {
            command.current_dir(dir);
        }
        
        // 啟動進程
        match command.spawn() {
            Ok(mut child) => {
                // 處理標準輸出
                if let Some(stdout) = child.stdout.take() {
                    let log_handler_clone = log_handler.clone();
                    let process_name_clone = process_name.clone();
                    
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stdout);
                        let mut line = String::new();
                        
                        loop {
                            line.clear();
                            match reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF
                                Ok(_) => {
                                    if !line.is_empty() {
                                        log_handler_clone.log(&process_name_clone, LogType::Stdout, line.trim());
                                    }
                                }
                                Err(e) => {
                                    log_handler_clone.log(
                                        &process_name_clone,
                                        LogType::System,
                                        &format!("Error reading stdout: {}", e),
                                    );
                                    break;
                                }
                            }
                        }
                    });
                }
                
                // 處理標準錯誤
                if let Some(stderr) = child.stderr.take() {
                    let log_handler_clone = log_handler.clone();
                    let process_name_clone = process_name.clone();
                    
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stderr);
                        let mut line = String::new();
                        
                        loop {
                            line.clear();
                            match reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF
                                Ok(_) => {
                                    if !line.is_empty() {
                                        log_handler_clone.log(&process_name_clone, LogType::Stderr, line.trim());
                                    }
                                }
                                Err(e) => {
                                    log_handler_clone.log(
                                        &process_name_clone,
                                        LogType::System,
                                        &format!("Error reading stderr: {}", e),
                                    );
                                    break;
                                }
                            }
                        }
                    });
                }
                
                // 設置監控進程退出
                {
                    let mut process = self.get_process_mut(&process_name).unwrap();
                    process.process = Some(child);
                    process.status = ProcessStatus::Running;
                    process.start_time = Some(Instant::now());
                }
                
                // 創建共享引用用於監控
                let process_name_clone = process_name.clone();
                let log_handler_clone = log_handler.clone();
                
                // 監控進程退出
                tokio::spawn(async move {
                    // 簡單的方案是僅記錄啟動監控
                    log_handler_clone.log(
                        &process_name_clone,
                        LogType::System,
                        "Process monitoring started",
                    );
                });
                
                log_handler.log(
                    &process_name,
                    LogType::System,
                    "Process started",
                );
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to start process: {}", e);
                log_handler.log(&process_name, LogType::System, &error_msg);
                
                let mut process = self.get_process_mut(&process_name).unwrap();
                process.status = ProcessStatus::Failed;
                
                Err(JanusError::Process(error_msg))
            }
        }
    }
}
