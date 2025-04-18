use signal_hook::{consts::{SIGINT, SIGTERM}, iterator::Signals};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::error::{JanusError, Result};
use crate::process::manager::ProcessManager;

pub struct SignalHandler {
    manager: Arc<Mutex<ProcessManager>>,
}

impl SignalHandler {
    pub fn new(manager: Arc<Mutex<ProcessManager>>) -> Self {
        Self { manager }
    }
    
    pub fn register_signals(&self) -> Result<()> {
        let signals = Signals::new(&[SIGINT, SIGTERM])
            .map_err(|e| JanusError::Signal(format!("Failed to register signals: {}", e)))?;
        
        let manager = self.manager.clone();
        
        thread::spawn(move || {
            for signal in signals.forever() {
                match signal {
                    SIGINT | SIGTERM => {
                        println!("Received termination signal, shutting down...");
                        if let Ok(mut manager) = manager.lock() {
                            let _ = manager.stop_all();
                        }
                        std::process::exit(0);
                    }
                    _ => unreachable!(),
                }
            }
        });
        
        Ok(())
    }
}
