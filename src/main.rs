mod cli;
mod config;
mod error;
mod logging;
mod process;
mod signal;

use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use cli::command_parser::CommandParser;
use config::manager::ConfigManager;
use error::Result;
use logging::handler::LogHandler;
use process::manager::ProcessManager;
use signal::handler::SignalHandler;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // 獲取命令行參數
    let args: Vec<String> = env::args().collect();
    
    // 檢查是否是幫助或版本命令
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h" || args[1] == "--version" || args[1] == "-V") {
        // 直接創建命令解析器並顯示幫助信息
        let empty_manager = Arc::new(Mutex::new(ProcessManager::new_empty()));
        let command_parser = CommandParser::new(empty_manager);
        
        if args[1] == "--help" || args[1] == "-h" {
            // 手動顯示幫助信息
            let mut cli = command_parser.build_cli();
            println!("{}", cli.render_help());
        } else {
            // 顯示版本信息
            let cli = command_parser.build_cli();
            println!("{} {}", cli.get_name(), cli.get_version().unwrap_or("unknown"));
        }
        return Ok(());
    }
    
    // 默認配置文件路徑
    let default_config = "janus.toml";
    
    // 解析配置文件路徑
    let config_path = if args.len() > 2 && args[1] == "--config" {
        &args[2]
    } else {
        default_config
    };
    
    // 初始化配置管理器
    let config_manager = ConfigManager::new(config_path)?;
    
    // 獲取日誌級別
    let log_level = config_manager
        .get_global_config()
        .log_level
        .as_deref()
        .unwrap_or("info");
    
    // 初始化日誌處理器
    let log_handler = LogHandler::new(log_level);
    
    // 初始化進程管理器
    let process_manager = ProcessManager::new(config_manager, log_handler);
    
    // 使用 Arc<Mutex<>> 包裝進程管理器以便在多個線程間共享
    let manager = Arc::new(Mutex::new(process_manager));
    
    // 初始化信號處理器
    let signal_handler = SignalHandler::new(manager.clone());
    signal_handler.register_signals().await?;
    
    // 初始化命令解析器
    let command_parser = CommandParser::new(manager);
    
    // 解析並執行命令
    command_parser.parse_and_execute(args).await?;
    
    Ok(())
}
