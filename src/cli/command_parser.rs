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
    
    pub fn parse_and_execute(&self, args: Vec<String>) -> Result<()> {
        let matches = self.build_cli().get_matches_from(args);
        
        match matches.subcommand() {
            Some(("start", _)) => self.cmd_start_all(),
            Some(("stop", _)) => self.cmd_stop_all(),
            Some(("restart", _)) => self.cmd_restart_all(),
            Some(("status", _)) => self.cmd_status(),
            Some(("start-one", sub_m)) => self.cmd_start_one(sub_m),
            Some(("stop-one", sub_m)) => self.cmd_stop_one(sub_m),
            Some(("restart-one", sub_m)) => self.cmd_restart_one(sub_m),
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
    
    fn cmd_start_all(&self) -> Result<()> {
        println!("Starting all processes...");
        let mut manager = self.manager.lock().unwrap();
        manager.start_all()?;
        println!("All processes started");
        Ok(())
    }
    
    fn cmd_stop_all(&self) -> Result<()> {
        println!("Stopping all processes...");
        let mut manager = self.manager.lock().unwrap();
        manager.stop_all()?;
        println!("All processes stopped");
        Ok(())
    }
    
    fn cmd_restart_all(&self) -> Result<()> {
        println!("Restarting all processes...");
        let mut manager = self.manager.lock().unwrap();
        manager.stop_all()?;
        manager.start_all()?;
        println!("All processes restarted");
        Ok(())
    }
    
    fn cmd_status(&self) -> Result<()> {
        let manager = self.manager.lock().unwrap();
        let reporter = StatusReporter::new(&manager);
        reporter.report_all()?;
        Ok(())
    }
    
    fn cmd_start_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Starting process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        let runner = crate::process::runner::ProcessRunner::new();
        runner.start_process(&mut manager, name)?;
        
        println!("Process started: {}", name);
        Ok(())
    }
    
    fn cmd_stop_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Stopping process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        let process = manager.get_process_mut(name).ok_or_else(|| {
            JanusError::Process(format!("Process not found: {}", name))
        })?;
        
        if let Some(child) = &mut process.process {
            if let Err(e) = child.kill() {
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
    
    fn cmd_restart_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Restarting process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        manager.restart_process(name)?;
        
        println!("Process restarted: {}", name);
        Ok(())
    }
}
