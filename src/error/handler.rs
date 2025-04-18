use crate::error::ErrorType;
use crate::logging::handler::LogHandler;
use crate::logging::LogType;

pub struct ErrorHandler {
    log_handler: LogHandler,
}

impl ErrorHandler {
    pub fn new(log_handler: LogHandler) -> Self {
        Self { log_handler }
    }
    
    pub fn handle_error(&self, process_name: &str, error_type: ErrorType, message: &str) {
        let error_msg = format!("[{}] {}: {}", error_type_to_string(&error_type), process_name, message);
        self.log_handler.log(process_name, LogType::System, &error_msg);
    }
    
    pub fn classify_error(&self, exit_code: i32) -> ErrorType {
        match exit_code {
            0 => ErrorType::AbnormalExit, // 正常退出但未預期
            _ => ErrorType::StartFailed,  // 非零退出碼
        }
    }
}

fn error_type_to_string(error_type: &ErrorType) -> &'static str {
    match error_type {
        ErrorType::StartFailed => "START_FAILED",
        ErrorType::AbnormalExit => "ABNORMAL_EXIT",
        ErrorType::RestartLimited => "RESTART_LIMITED",
        ErrorType::ConfigInvalid => "CONFIG_INVALID",
    }
}
