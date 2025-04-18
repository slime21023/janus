use thiserror::Error;

#[derive(Error, Debug)]
pub enum JanusError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Process error: {0}")]
    Process(String),
    
    #[error("Signal handling error: {0}")]
    Signal(String),
    
    #[error("Command error: {0}")]
    Command(String),
}

pub type Result<T> = std::result::Result<T, JanusError>;

// 錯誤類型枚舉
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    StartFailed,
    AbnormalExit,
    RestartLimited,
    ConfigInvalid,
}
