use clap::{Command, Arg, ArgMatches};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{JanusError, Result};
use crate::process::manager::ProcessManager;
use crate::process::ProcessStatus;

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
            Some(("status", _)) => self.cmd_status().await,
            Some(("start-one", sub_m)) => self.cmd_start_one(sub_m).await,
            Some(("stop-one", sub_m)) => self.cmd_stop_one(sub_m).await,
            Some(("restart-one", sub_m)) => self.cmd_restart_one(sub_m).await,
            _ => Err(JanusError::Command("Unknown command".to_string())),
        }
    }
    
    pub fn build_cli(&self) -> Command {
        Command::new("janus")
            .version("0.1.0")
            .author("Janus Team")
            .about("A lightweight process manager for container environments")
            .long_about(
                "Janus is a lightweight process manager designed specifically for container environments. \
                It provides simple yet powerful commands to manage multiple processes, \
                with features like auto-restart, status monitoring, and structured logging."
            )
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
                    .long_help(
                        "Specify a custom configuration file path instead of using the default 'janus.toml'. \
                        The configuration file defines processes to manage, their startup parameters, \
                        working directories, environment variables, and restart policies."
                    )
            )
            .subcommand(
                Command::new("start")
                    .about("Start all processes")
                    .long_about(
                        "Start all processes defined in the configuration file. \
                        Processes that are already running will be skipped. \
                        Any startup errors will be reported, but won't prevent other processes from starting."
                    )
                    .display_order(1)
            )
            .subcommand(
                Command::new("stop")
                    .about("Stop all processes")
                    .long_about(
                        "Stop all currently running processes managed by Janus. \
                        This sends a termination signal to each process and waits for them to exit gracefully. \
                        For containers, this is often the command to use before shutting down."
                    )
                    .display_order(2)
            )
            .subcommand(
                Command::new("restart")
                    .about("Restart all processes")
                    .long_about(
                        "Restart all processes by stopping them if they're running, then starting them again. \
                        This is useful when you need to reload all processes, such as after a configuration change."
                    )
                    .display_order(3)
            )
            .subcommand(
                Command::new("status")
                    .about("Show status of all processes")
                    .long_about(
                        "Display detailed status information for all processes, including their running state, \
                        uptime (for running processes), command, arguments, environment variables, \
                        and restart configuration."
                    )
                    .display_order(4)
            )
            .subcommand(
                Command::new("start-one")
                    .about("Start a specific process")
                    .long_about(
                        "Start a single process by name. If the process is already running, it will be skipped. \
                        This command is useful when you want to start processes selectively."
                    )
                    .arg(
                        Arg::new("name")
                            .help("Name of the process to start")
                            .long_help(
                                "The process name as defined in the configuration file. \
                                This must match exactly the name in the [process.NAME] section."
                            )
                            .required(true)
                            .index(1),
                    )
                    .display_order(5)
                    .after_help("Example: janus start-one web-server")
            )
            .subcommand(
                Command::new("stop-one")
                    .about("Stop a specific process")
                    .long_about(
                        "Stop a single process by name. If the process is not running, a message will be displayed. \
                        This command sends a termination signal and waits for the process to exit gracefully."
                    )
                    .arg(
                        Arg::new("name")
                            .help("Name of the process to stop")
                            .long_help(
                                "The process name as defined in the configuration file. \
                                This must match exactly the name in the [process.NAME] section."
                            )
                            .required(true)
                            .index(1),
                    )
                    .display_order(6)
                    .after_help("Example: janus stop-one database")
            )
            .subcommand(
                Command::new("restart-one")
                    .about("Restart a specific process")
                    .long_about(
                        "Restart a single process by name, stopping it first if it's running, then starting it again. \
                        This is useful for reloading a specific process after configuration changes."
                    )
                    .arg(
                        Arg::new("name")
                            .help("Name of the process to restart")
                            .long_help(
                                "The process name as defined in the configuration file. \
                                This must match exactly the name in the [process.NAME] section."
                            )
                            .required(true)
                            .index(1),
                    )
                    .display_order(7)
                    .after_help("Example: janus restart-one api-service")
            )
            .after_help(
                "CONFIGURATION FILE FORMAT:\n\
                The configuration file uses TOML format with the following structure:\n\n\
                [global]\n\
                log_level = \"info\"  # Optional, default is \"info\"\n\
                working_dir = \"/app\"  # Optional, default working directory\n\
                env = { KEY = \"value\" }  # Optional, global environment variables\n\n\
                [process.web-server]\n\
                command = \"node\"\n\
                args = [\"server.js\"]\n\
                working_dir = \"/app/web\"  # Overrides global working_dir\n\
                env = { PORT = \"8080\" }  # Merged with global env\n\
                auto_restart = true  # Optional, default is false\n\
                restart_limit = 5  # Optional, maximum number of restarts\n\
                restart_delay = 2  # Optional, seconds to wait before restart\n\n\
                [process.worker]\n\
                command = \"python\"\n\
                args = [\"worker.py\"]\n\
                auto_restart = true\n\n\
                For more information and examples, visit: https://github.com/example/janus"
            )
    }
    
    async fn cmd_start_all(&self) -> Result<()> {
        println!("Starting all processes...");
        
        let process_names = {
            let manager = self.manager.lock().await;
            manager.get_all_processes()
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        };
        
        for name in process_names {
            let result = {
                let mut manager = self.manager.lock().await;
                manager.start_process(&name).await
            };
            
            if let Err(e) = result {
                eprintln!("Failed to start {}: {}", name, e);
            }
        }
        
        println!("All processes started");
        Ok(())
    }
    
    async fn cmd_stop_all(&self) -> Result<()> {
        println!("Stopping all processes...");
        
        {
            let mut manager = self.manager.lock().await;
            manager.stop_all().await?;
        }
        
        println!("All processes stopped");
        Ok(())
    }
    
    async fn cmd_restart_all(&self) -> Result<()> {
        println!("Restarting all processes...");
        
        {
            let mut manager = self.manager.lock().await;
            manager.stop_all().await?;
        }
        
        let process_names = {
            let manager = self.manager.lock().await;
            manager.get_all_processes()
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        };
        
        for name in process_names {
            let result = {
                let mut manager = self.manager.lock().await;
                manager.start_process(&name).await
            };
            
            if let Err(e) = result {
                eprintln!("Failed to restart {}: {}", name, e);
            }
        }
        
        println!("All processes restarted");
        Ok(())
    }
    
    async fn cmd_status(&self) -> Result<()> {
        let manager = self.manager.lock().await;
        let reporter = StatusReporter::new(&manager);
        reporter.report_all()?;
        Ok(())
    }
    
    async fn cmd_start_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Starting process: {}", name);
        
        {
            let mut manager = self.manager.lock().await;
            manager.start_process(name).await?;
        }
        
        println!("Process started: {}", name);
        Ok(())
    }
    
    async fn cmd_stop_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Stopping process: {}", name);
        
        let process_exists_and_running = {
            let manager = self.manager.lock().await;
            manager.get_all_processes()
                .get(name)
                .map(|p| p.status == ProcessStatus::Running && p.process.is_some())
                .unwrap_or(false)
        };
        
        if process_exists_and_running {
            let result = {
                let mut manager = self.manager.lock().await;
                if let Some(process) = manager.get_process_mut(name) {
                    if let Some(child) = &mut process.process {
                        match child.kill().await {
                            Ok(_) => {
                                process.status = ProcessStatus::Stopped;
                                process.process = None;
                                Ok(())
                            }
                            Err(e) => Err(JanusError::Process(format!("Failed to kill process: {}", e)))
                        }
                    } else {
                        Ok(()) // Process is not running
                    }
                } else {
                    Err(JanusError::Process(format!("Process not found: {}", name)))
                }
            };
            
            match result {
                Ok(_) => println!("Process stopped: {}", name),
                Err(e) => return Err(e),
            }
        } else {
            println!("Process is not running: {}", name);
        }
        
        Ok(())
    }
    
    async fn cmd_restart_one(&self, matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("name").unwrap();
        println!("Restarting process: {}", name);
        
        {
            let mut manager = self.manager.lock().await;
            manager.restart_process(name).await?;
        }
        
        println!("Process restarted: {}", name);
        Ok(())
    }
}
