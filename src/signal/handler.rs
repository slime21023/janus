#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use std::sync::{Arc, Mutex};

use crate::error::Result;
use crate::process::manager::ProcessManager;

pub struct SignalHandler {
    manager: Arc<Mutex<ProcessManager>>,
}

impl SignalHandler {
    pub fn new(manager: Arc<Mutex<ProcessManager>>) -> Self {
        Self { manager }
    }
    
    pub async fn register_signals(&self) -> Result<()> {
        #[cfg(unix)]
        {
            let mut sigint = signal(SignalKind::interrupt())?;
            let mut sigterm = signal(SignalKind::terminate())?;
            
            let manager = self.manager.clone();
            
            tokio::spawn(async move {
                tokio::select! {
                    _ = sigint.recv() => {
                        println!("Received SIGINT, shutting down...");
                    }
                    _ = sigterm.recv() => {
                        println!("Received SIGTERM, shutting down...");
                    }
                }
                
                if let Ok(mut manager) = manager.lock() {
                    let _ = manager.stop_all();
                }
                std::process::exit(0);
            });
        }
        
        // Windows 平台簡化處理
        #[cfg(windows)]
        {
            let manager = self.manager.clone();
            
            // 在 Windows 上使用 tokio 的 ctrl_c 處理程序
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                println!("Received Ctrl+C, shutting down...");
                
                if let Ok(mut manager) = manager.lock() {
                    let _ = manager.stop_all();
                }
                std::process::exit(0);
            });
            
            println!("Ctrl+C handler registered");
        }
        
        Ok(())
    }
}
