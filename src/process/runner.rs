use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{JanusError, Result};
use crate::logging::LogType;
use crate::process::{manager::ProcessManager, ProcessStatus};

pub struct ProcessRunner<'a> {
    manager: &'a mut ProcessManager,
}

impl<'a> ProcessRunner<'a> {
    pub fn new(manager: &'a mut ProcessManager) -> Self {
        Self { manager }
    }
    
    pub fn start_process(&mut self, name: &str) -> Result<()> {
        let process = self.manager.get_process_mut(name).ok_or_else(|| {
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
        let log_handler = self.manager.get_log_handler().clone();
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
                let process = self.manager.get_process_mut(name).unwrap();
                process.process = Some(child);
                process.status = ProcessStatus::Running;
                process.start_time = Some(Instant::now());
                
                // 啟動監控線程
                self.monitor_process(name);
                
                self.manager.get_log_handler().log(
                    name,
                    LogType::System,
                    &format!("Process started: {}", name),
                );
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to start process: {}", e);
                self.manager.get_log_handler().log(name, LogType::System, &error_msg);
                
                let process = self.manager.get_process_mut(name).unwrap();
                process.status = ProcessStatus::Failed;
                
                Err(JanusError::Process(error_msg))
            }
        }
    }
    
    pub fn monitor_process(&self, name: &str) {
        let process_name = name.to_string();
        let manager = self.manager as *mut ProcessManager;
        
        thread::spawn(move || {
            // 這裡使用不安全的指針來避免生命週期問題
            // 在實際產品中應該使用更安全的方式，如 Arc<Mutex<ProcessManager>>
            let manager = unsafe { &mut *manager };
            
            loop {
                // 獲取進程
                let process = match manager.get_process_mut(&process_name) {
                    Some(p) => p,
                    None => break,
                };
                
                // 檢查進程是否還在運行
                if let Some(child) = &mut process.process {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // 進程已退出
                            let exit_code = status.code().unwrap_or(-1);
                            let error_type = manager.get_error_handler().classify_error(exit_code);
                            
                            manager.get_error_handler().handle_error(
                                &process_name,
                                error_type,
                                &format!("Process exited with code: {}", exit_code),
                            );
                            
                            process.status = ProcessStatus::Stopped;
                            process.process = None;
                            
                            // 如果配置了自動重啟，則重啟進程
                            if process.auto_restart {
                                // 檢查重啟次數限制
                                if let Some(limit) = process.restart_limit {
                                    if process.restart_count >= limit {
                                        manager.get_error_handler().handle_error(
                                            &process_name,
                                            crate::error::ErrorType::RestartLimited,
                                            &format!("Restart limit reached: {}", limit),
                                        );
                                        break;
                                    }
                                }
                                
                                // 增加重啟計數
                                process.restart_count += 1;
                                
                                // 等待指定的延遲時間
                                let delay = process.restart_delay;
                                drop(process); // 釋放可變引用
                                
                                thread::sleep(Duration::from_secs(delay));
                                
                                // 重啟進程
                                let mut runner = ProcessRunner::new(manager);
                                if let Err(e) = runner.start_process(&process_name) {
                                    manager.get_log_handler().log(
                                        &process_name,
                                        LogType::System,
                                        &format!("Failed to restart process: {}", e),
                                    );
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        Ok(None) => {
                            // 進程還在運行
                            thread::sleep(Duration::from_millis(500));
                        }
                        Err(e) => {
                            // 檢查進程狀態時出錯
                            manager.get_log_handler().log(
                                &process_name,
                                LogType::System,
                                &format!("Error checking process status: {}", e),
                            );
                            
                            process.status = ProcessStatus::Failed;
                            process.process = None;
                            break;
                        }
                    }
                } else {
                    // 進程已經不存在
                    break;
                }
            }
        });
    }
}
