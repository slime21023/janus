use clap::{Command, Arg, ArgMatches};
use std::sync::{Arc, Mutex};

use crate::error::{JanusError, Result};
use crate::process::manager::ProcessManager;

use super::status_reporter::StatusReporter;

pub struct CommandParser {
    manager: Arc<Mutex<ProcessManager>>,
}

impl CommandParser {
    pub fn new(manager: Arc<Mutex<ProcessManager>>) -> Self {
        Self { manager }
    }
    
    pub async fn parse_and_execute(&self, args: Vec<String>) -> Result<()> {
        let matches = self.build_cli().get_matches_from(args);
        
        match matches.subcommand() {
            Some(("start", _)) => self.cmd_start_all().await,
            Some(("stop", _)) => self.cmd_stop_all().await,
            Some(("restart", _)) => self.cmd_restart_all().await,
            Some(("status", _)) => self.cmd_status(),
            Some(("start-one", sub_m)) => self.cmd_start_one(sub_m).await,
            Some(("stop-one", sub_m)) => self.cmd_stop_one(sub_m).await,
            Some(("restart-one", sub_m)) => self.cmd_restart_one(sub_m).await,
            _ => Err(JanusError::Command("Unknown command".to_string())),
        }
    }
    
    fn build_cli(&self) -> Command {
        Command::new("janus")
            .version("0.1.0")
            .author("Your Name <your.email@example.com>")
            .about("A lightweight process manager for container environments")
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
            )
            .subcommand(Command::new("start").about("Start all processes"))
            .subcommand(Command::new("stop").about("Stop all processes"))
            .subcommand(Command::new("restart").about("Restart all processes"))
            .subcommand(Command::new("status").about("Show status of all processes"))
            .subcommand(
                Command::new("start-one")
                    .about("Start a specific process")
                    .arg(
                        Arg::new("name")
                            .help("Process name")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("stop-one")
                    .about("Stop a specific process")
                    .arg(
                        Arg::new("name")
                            .help("Process name")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                Command::new("restart-one")
                    .about("Restart a specific process")
                    .arg(
                        Arg::new("name")
                            .help("Process name")
                            .required(true)
                            .index(1),
                    ),
            )
    }
    
    async fn cmd_start_all(&self) -> Result<()> {
        println!("Starting all processes...");
        let mut manager = self.manager.lock().unwrap();
        
        // 獲取所有進程名稱
        let process_names: Vec<String> = manager.get_all_processes()
            .keys()
            .cloned()
            .collect();
        
        // 依次啟動所有進程
        for name in process_names {
            if let Err(e) = manager.start_process(&name).await {
                eprintln!("Failed to start {}: {}", name, e);
            }
        }
        
        println!("All processes started");
        Ok(())
    }
    
    async fn cmd_stop_all(&self) -> Result<()> {
        println!("Stopping all processes...");
        let mut manager = self.manager.lock().unwrap();
        manager.stop_all().await?;
        println!("All processes stopped");
        Ok(())
    }
    
    async fn cmd_restart_all(&self) -> Result<()> {
        println!("Restarting all processes...");
        let mut manager = self.manager.lock().unwrap();
        manager.stop_all().await?;
        
        // 獲取所有進程名稱
        let process_names: Vec<String> = manager.get_all_processes()
            .keys()
            .cloned()
            .collect();
        
        // 依次啟動所有進程
        for name in process_names {
            if let Err(e) = manager.start_process(&name).await {
                eprintln!("Failed to restart {}: {}", name, e);
            }
        }
        
        println!("All processes restarted");
        Ok(())
    }
    
    fn cmd_status(&self) -> Result<()> {
        let manager = self.manager.lock().unwrap();
        let reporter = StatusReporter::new(&manager);
        reporter.report_all()?;
        Ok(())
    }
    
    async fn cmd_start_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Starting process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        manager.start_process(name).await?;
        
        println!("Process started: {}", name);
        Ok(())
    }
    
    async fn cmd_stop_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Stopping process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        let process = manager.get_process_mut(name).ok_or_else(|| {
            JanusError::Process(format!("Process not found: {}", name))
        })?;
        
        if let Some(child) = &mut process.process {
            // 使用 tokio 的 Child::kill() 並等待進程退出
            if let Err(e) = child.kill().await {
                return Err(JanusError::Process(format!("Failed to kill process: {}", e)));
            }
            process.status = crate::process::ProcessStatus::Stopped;
            process.process = None;
            println!("Process stopped: {}", name);
            Ok(())
        } else {
            println!("Process is not running: {}", name);
            Ok(())
        }
    }
    
    async fn cmd_restart_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Restarting process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        manager.restart_process(name).await?;
        
        println!("Process restarted: {}", name);
        Ok(())
    }
}
