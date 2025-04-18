use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

use crate::error::{JanusError, Result};
use crate::logging::LogType;
use crate::process::{manager::ProcessManager, ProcessStatus};

pub struct ProcessRunner {
}

impl ProcessRunner {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn start_process(&self, manager: &mut ProcessManager, name: &str) -> Result<()> {
        let process = manager.get_process_mut(name).ok_or_else(|| {
            JanusError::Process(format!("Process not found: {}", name))
        })?;
        
        // 如果進程已經在運行，則不做任何事
        if process.status == ProcessStatus::Running && process.process.is_some() {
            return Ok(());
        }
        
        // 創建命令
        let mut command = Command::new(&process.command);
        command.args(&process.args);
        
        // 設置環境變量
        for (key, value) in &process.env {
            command.env(key, value);
        }
        
        // 設置工作目錄
        if let Some(dir) = &process.working_dir {
            command.current_dir(dir);
        }
        
        // 設置標準輸出和標準錯誤
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        
        // 啟動進程
        let log_handler = manager.get_log_handler().clone();
        let process_name = name.to_string();
        
        match command.spawn() {
            Ok(mut child) => {
                // 處理標準輸出
                if let Some(stdout) = child.stdout.take() {
                    let process_name_clone = process_name.clone();
                    let log_handler_clone = log_handler.clone();
                    
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                log_handler_clone.log(&process_name_clone, LogType::Stdout, &line);
                            }
                        }
                    });
                }
                
                // 處理標準錯誤
                if let Some(stderr) = child.stderr.take() {
                    let process_name_clone = process_name.clone();
                    let log_handler_clone = log_handler.clone();
                    
                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                log_handler_clone.log(&process_name_clone, LogType::Stderr, &line);
                            }
                        }
                    });
                }
                
                // 更新進程狀態
                let process = manager.get_process_mut(name).unwrap();
                process.process = Some(child);
                process.status = ProcessStatus::Running;
                process.start_time = Some(Instant::now());
                
                // 啟動監控線程
                self.monitor_process(manager, name);
                
                manager.get_log_handler().log(
                    name,
                    LogType::System,
                    &format!("Process started: {}", name),
                );
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to start process: {}", e);
                manager.get_log_handler().log(name, LogType::System, &error_msg);
                
                let process = manager.get_process_mut(name).unwrap();
                process.status = ProcessStatus::Failed;
                
                Err(JanusError::Process(error_msg))
            }
        }
    }
    
    pub fn monitor_process(&self, manager: &mut ProcessManager, name: &str) {
        // 簡化實現，僅使用 log_handler 來報告進程退出
        // 這避免了複雜的可變借用問題
        let log_handler = manager.get_log_handler().clone();
        let process_name = name.to_string();
        
        // 只提取我們需要的信息，不保留對 manager 的引用
        let child_opt = if let Some(process) = manager.get_process_mut(name) {
            process.process.take()
        } else {
            None
        };
        
        if let Some(mut child) = child_opt {
            // 標記進程正在運行 (需在此重新獲取 process)
            if let Some(process) = manager.get_process_mut(name) {
                process.status = ProcessStatus::Running;
            }
            
            // 將必要資訊移到線程中
            thread::spawn(move || {
                // 簡化：只監控進程退出，不嘗試自動重啟
                match child.wait() {
                    Ok(status) => {
                        let exit_code = status.code().unwrap_or(-1);
                        log_handler.log(
                            &process_name,
                            LogType::System,
                            &format!("Process exited with code: {}", exit_code)
                        );
                    }
                    Err(e) => {
                        log_handler.log(
                            &process_name,
                            LogType::System,
                            &format!("Error waiting for process: {}", e)
                        );
                    }
                }
            });
        }
    }
}
