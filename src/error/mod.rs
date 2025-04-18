use std::io;
use std::result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum JanusError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Config error: {0}")]
    Config(String),
    
    #[error("Process error: {0}")]
    Process(String),
    
    #[error("Command error: {0}")]
    Command(String),
}

pub type Result<T> = result::Result<T, JanusError>;

// 重構：移除未使用的錯誤類型和處理器
// 後續可以根據需要重新添加更合適的錯誤類型

pub mod handler {
    // 這個模塊保留用於將來擴展，但當前簡化為空
}
