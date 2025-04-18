use clap::{App, Arg, ArgMatches, SubCommand};
use std::sync::{Arc, Mutex};

use crate::config::manager::ConfigManager;
use crate::error::{JanusError, Result};
use crate::logging::handler::LogHandler;
use crate::process::manager::ProcessManager;
use crate::signal::handler::SignalHandler;

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
            ("start", _) => self.cmd_start_all(),
            ("stop", _) => self.cmd_stop_all(),
            ("restart", _) => self.cmd_restart_all(),
            ("status", _) => self.cmd_status(),
            ("start-one", Some(sub_m)) => self.cmd_start_one(sub_m),
            ("stop-one", Some(sub_m)) => self.cmd_stop_one(sub_m),
            ("restart-one", Some(sub_m)) => self.cmd_restart_one(sub_m),
            _ => Err(JanusError::Command("Unknown command".to_string())),
        }
    }
    
    fn build_cli(&self) -> App<'static, 'static> {
        App::new("janus")
            .version("0.1.0")
            .author("Your Name <your.email@example.com>")
            .about("A lightweight process manager for container environments")
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
                    .takes_value(true),
            )
            .subcommand(SubCommand::with_name("start").about("Start all processes"))
            .subcommand(SubCommand::with_name("stop").about("Stop all processes"))
            .subcommand(SubCommand::with_name("restart").about("Restart all processes"))
            .subcommand(SubCommand::with_name("status").about("Show status of all processes"))
            .subcommand(
                SubCommand::with_name("start-one")
                    .about("Start a specific process")
                    .arg(
                        Arg::with_name("name")
                            .help("Process name")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                SubCommand::with_name("stop-one")
                    .about("Stop a specific process")
                    .arg(
                        Arg::with_name("name")
                            .help("Process name")
                            .required(true)
                            .index(1),
                    ),
            )
            .subcommand(
                SubCommand::with_name("restart-one")
                    .about("Restart a specific process")
                    .arg(
                        Arg::with_name("name")
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
        let name = matches.value_of("name").unwrap();
        println!("Starting process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        let mut runner = crate::process::runner::ProcessRunner::new(&mut manager);
        runner.start_process(name)?;
        
        println!("Process started: {}", name);
        Ok(())
    }
    
    fn cmd_stop_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.value_of("name").unwrap();
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
        let name = matches.value_of("name").unwrap();
        println!("Restarting process: {}", name);
        
        let mut manager = self.manager.lock().unwrap();
        manager.restart_process(name)?;
        
        println!("Process restarted: {}", name);
        Ok(())
    }
}

