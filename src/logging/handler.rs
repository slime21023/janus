use chrono::Local;
use colored::*;
use std::io::Write;

use crate::logging::{LogEntry, LogType};

#[derive(Clone)]
pub struct LogHandler {}

impl LogHandler {
    pub fn new(_log_level: &str) -> Self {
        // 保留參數以保持 API 兼容性，但不存儲它
        Self {}
    }
    
    pub fn log(&self, process_name: &str, log_type: LogType, content: &str) {
        let entry = LogEntry {
            timestamp: Local::now(),
            process_name: process_name.to_string(),
            log_type: log_type.clone(),
            content: content.to_string(),
        };
        
        let formatted = self.format_log_entry(&entry);
        match log_type {
            LogType::Stderr => {
                let _ = std::io::stderr().write_all(formatted.as_bytes());
            }
            _ => {
                let _ = std::io::stdout().write_all(formatted.as_bytes());
            }
        }
    }
    
    pub fn format_log_entry(&self, entry: &LogEntry) -> String {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let prefix = match entry.log_type {
            LogType::Stdout => format!("[{}] [{}]", timestamp.blue(), entry.process_name.green()),
            LogType::Stderr => format!("[{}] [{}]", timestamp.blue(), entry.process_name.red()),
            LogType::System => format!("[{}] [{}]", timestamp.blue(), "SYSTEM".yellow()),
        };
        
        format!("{} {}\n", prefix, entry.content)
    }
}
