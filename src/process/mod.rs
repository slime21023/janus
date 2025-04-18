pub mod manager;
use std::collections::HashMap;
use std::time::Instant;
use tokio::process::Child;

#[derive(Clone, PartialEq, Debug)]
pub enum ProcessStatus {
    Stopped,
    Running,
    Failed,
}

// ManagedProcess 不能自動派生 Clone，因為 tokio::process::Child 不實現 Clone
pub struct ManagedProcess {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub env: HashMap<String, String>,
    pub auto_restart: bool,
    pub restart_count: u32,
    pub restart_limit: Option<u32>,
    pub restart_delay: u64,
    pub status: ProcessStatus,
    pub process: Option<Child>,
    pub start_time: Option<Instant>,
}

// 手動實現 Clone，避免克隆 tokio::process::Child
impl Clone for ManagedProcess {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            working_dir: self.working_dir.clone(),
            env: self.env.clone(),
            auto_restart: self.auto_restart,
            restart_count: self.restart_count,
            restart_limit: self.restart_limit,
            restart_delay: self.restart_delay,
            status: self.status.clone(),
            process: None, // 不克隆進程句柄
            start_time: self.start_time, // Instant 已實現 Copy，無需克隆
        }
    }
}
