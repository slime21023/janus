pub mod handler;

#[derive(Debug, Clone, PartialEq)]
pub enum LogType {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub process_name: String,
    pub log_type: LogType,
    pub content: String,
}
