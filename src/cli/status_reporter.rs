use std::time::{Duration, Instant};

use crate::process::{ProcessStatus, manager::ProcessManager};

pub struct StatusReporter<'a> {
    process_manager: &'a ProcessManager,
}

impl<'a> StatusReporter<'a> {
    pub fn new(process_manager: &'a ProcessManager) -> Self {
        Self { process_manager }
    }
    
    pub fn report_all(&self) -> crate::error::Result<()> {
        let processes = self.process_manager.get_all_processes();
        
        if processes.is_empty() {
            println!("No processes configured");
            return Ok(());
        }
        
        println!("Process Status Report:");
        println!("=====================");
        
        for (name, process) in processes {
            self.report_status(name, process);
            println!("---------------------");
        }
        
        Ok(())
    }
    
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let days = total_secs / (24 * 60 * 60);
        let hours = (total_secs % (24 * 60 * 60)) / (60 * 60);
        let minutes = (total_secs % (60 * 60)) / 60;
        let seconds = total_secs % 60;
        
        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
    
    fn report_status(&self, name: &str, process: &crate::process::ManagedProcess) {
        println!("Process: {}", name);
        println!("Status: {:?}", process.status);
        
        // 顯示運行時間（如果進程正在運行）
        if process.status == ProcessStatus::Running {
            if let Some(start_time) = process.start_time {
                let uptime = start_time.elapsed();
                println!("Uptime: {}", Self::format_duration(uptime));
            }
        }
        
        // 顯示命令和參數
        println!("Command: {}", process.command);
        if !process.args.is_empty() {
            println!("Args: {:?}", process.args);
        }
        
        // 顯示工作目錄
        if let Some(dir) = &process.working_dir {
            println!("Working directory: {}", dir);
        }
        
        // 顯示環境變量
        if !process.env.is_empty() {
            println!("Environment variables: {} defined", process.env.len());
        }
        
        // 顯示重啟配置
        println!("Auto-restart: {}", process.auto_restart);
        println!("Restart count: {}", process.restart_count);
        
        if let Some(limit) = process.restart_limit {
            println!("Restart limit: {}", limit);
        } else {
            println!("Restart limit: unlimited");
        }
        
        println!("Restart delay: {} seconds", process.restart_delay);
    }
}
