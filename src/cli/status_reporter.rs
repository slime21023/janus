use colored::*;
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::process::{manager::ProcessManager, ProcessStatus};

pub struct StatusReporter<'a> {
    manager: &'a ProcessManager,
}

impl<'a> StatusReporter<'a> {
    pub fn new(manager: &'a ProcessManager) -> Self {
        Self { manager }
    }
    
    pub fn report_all(&self) -> Result<()> {
        println!("{}", "Process Status".bold().underline());
        println!("{:<20} {:<10} {:<15} {:<10}", "NAME", "STATUS", "UPTIME", "RESTARTS");
        println!("{}", "-".repeat(55));
        
        for (name, process) in self.manager.get_all_processes() {
            let status_str = match process.status {
                ProcessStatus::Running => process.status.to_string().green(),
                ProcessStatus::Stopped => process.status.to_string().yellow(),
                ProcessStatus::Failed => process.status.to_string().red(),
            };
            
            let uptime = match (process.status.clone(), process.start_time) {
                (ProcessStatus::Running, Some(start_time)) => {
                    format_duration(start_time.elapsed())
                }
                _ => "-".to_string(),
            };
            
            println!(
                "{:<20} {:<10} {:<15} {:<10}",
                name,
                status_str,
                uptime,
                process.restart_count
            );
        }
        
        Ok(())
    }
    
    pub fn report_process(&self, name: &str) -> Result<()> {
        let process = self.manager.get_all_processes().get(name).ok_or_else(|| {
            crate::error::JanusError::Process(format!("Process not found: {}", name))
        })?;
        
        println!("{}", format!("Process: {}", name).bold().underline());
        println!("Command: {} {}", process.command, process.args.join(" "));
        
        let status_str = match process.status {
            ProcessStatus::Running => process.status.to_string().green(),
            ProcessStatus::Stopped => process.status.to_string().yellow(),
            ProcessStatus::Failed => process.status.to_string().red(),
        };
        
        println!("Status: {}", status_str);
        
        if let Some(start_time) = process.start_time {
            if process.status == ProcessStatus::Running {
                println!("Uptime: {}", format_duration(start_time.elapsed()));
            }
        }
        
        println!("Auto restart: {}", process.auto_restart);
        println!("Restart count: {}", process.restart_count);
        
        if let Some(limit) = process.restart_limit {
            println!("Restart limit: {}", limit);
        } else {
            println!("Restart limit: unlimited");
        }
        
        println!("Restart delay: {} seconds", process.restart_delay);
        
        if !process.env.is_empty() {
            println!("\nEnvironment variables:");
            for (key, value) in &process.env {
                println!("  {}={}", key, value);
            }
        }
        
        Ok(())
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
