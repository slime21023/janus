pub mod manager;
use std::collections::HashMap;
use std::time::Instant;
use tokio::process::Child;

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Failed,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "RUNNING"),
            ProcessStatus::Stopped => write!(f, "STOPPED"),
            ProcessStatus::Failed => write!(f, "FAILED"),
        }
    }
}

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
            process: None, 
            start_time: self.start_time.clone(),
        }
    }
}
