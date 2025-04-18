pub mod manager;
pub mod runner;

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
