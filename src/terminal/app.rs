use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::process::{Command, Stdio};
use clap::Parser;
use guts::cli::{Cli, Commands};

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: String,
    pub output: String,
    pub error: Option<String>,
}

pub struct App {
    pub input: String,
    pub cursor_position: usize,
    pub command_history: Vec<CommandResult>,
    pub history_index: usize,
    pub input_history: Vec<String>,
    pub input_history_index: usize,
    pub should_quit: bool,
    pub current_dir: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            command_history: Vec::new(),
            history_index: 0,
            input_history: Vec::new(),
            input_history_index: 0,
            should_quit: false,
            current_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.execute_command()?;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Up => {
                if !self.input_history.is_empty() && self.input_history_index > 0 {
                    self.input_history_index -= 1;
                    self.input = self.input_history[self.input_history_index].clone();
                    self.cursor_position = self.input.len();
                }
            }
            KeyCode::Down => {
                if !self.input_history.is_empty() && self.input_history_index < self.input_history.len() - 1 {
                    self.input_history_index += 1;
                    self.input = self.input_history[self.input_history_index].clone();
                    self.cursor_position = self.input.len();
                } else if self.input_history_index == self.input_history.len() - 1 {
                    self.input_history_index = self.input_history.len();
                    self.input.clear();
                    self.cursor_position = 0;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn execute_command(&mut self) -> Result<()> {
        let command = self.input.trim().to_string();

        // add to input history
        if !command.is_empty() {
            self.input_history.push(command.clone());
            self.input_history_index = self.input_history.len();
        }

        let result = match command.as_str() {
            "exit" | "quit" => {
                self.should_quit = true;
                return Ok(());
            }
            "clear" => {
                self.command_history.clear();
                self.input.clear();
                self.cursor_position = 0;
                return Ok(());
            }
            "pwd" => {
                CommandResult {
                    command: command.clone(),
                    output: self.current_dir.clone(),
                    error: None,
                }
            }
            //  /!\ Need to Fix ls and cd /!\
            _ => {
                // handle guts commands
                if command.starts_with("guts ") {
                    self.execute_guts_command(&command)?
                } else {
                    // handle system commands
                    self.execute_system_command(&command)?
                }
            }
        };

        self.command_history.push(result);
        self.input.clear();
        self.cursor_position = 0;
        Ok(())
    }

    fn execute_guts_command(&mut self, command: &str) -> Result<CommandResult> {
        // parsing the command using clap
        let args: Vec<&str> = command.split_whitespace().collect();

        // try to parse the command with clap
        match Cli::try_parse_from(args) {
            Ok(cli) => {
                match cli.command {
                    Commands::Init(init_args) => {
                        match guts::commands::init::run(&init_args) {
                            Ok(_) => Ok(CommandResult {
                                command: command.to_string(),
                                output: "Repository initialized successfully".to_string(),
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::HashObject(hash_args) => {
                        match guts::commands::hash_object::run(&hash_args) {
                            Ok(_) => Ok(CommandResult {
                                command: command.to_string(),
                                output: "Object hashed successfully".to_string(),
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::CatFile(cat_args) => {
                        match guts::commands::cat_file::run(&cat_args) {
                            Ok(_) => Ok(CommandResult {
                                command: command.to_string(),
                                output: "File content displayed".to_string(),
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::WriteTree(tree_args) => {
                        match guts::commands::write_tree::run(&tree_args) {
                            Ok(_) => Ok(CommandResult {
                                command: command.to_string(),
                                output: "Tree written successfully".to_string(),
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::Tui => {
                        // this shouldn't happen in the TUI
                        Ok(CommandResult {
                            command: command.to_string(),
                            output: String::new(),
                            error: Some("Cannot launch TUI".to_string()),
                        })
                    }
                }
            }
            Err(e) => {
                // if clap parsing fails, show the error
                Ok(CommandResult {
                    command: command.to_string(),
                    output: String::new(),
                    error: Some(e.to_string()),
                })
            }
        }
    }

    fn execute_system_command(&mut self, command: &str) -> Result<CommandResult> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some("Empty command".to_string()),
            });
        }

        let output = Command::new(parts[0])
            .args(&parts[1..])
            .current_dir(&self.current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                // /!\ Need to Fix it /!\ (Soon, i am cooking)
                // handle cd command specially
                if parts[0] == "cd" && output.status.success() {
                    if parts.len() > 1 {
                        let new_dir = std::path::PathBuf::from(&self.current_dir).join(parts[1]);
                        if let Ok(canonical) = new_dir.canonicalize() {
                            self.current_dir = canonical.to_string_lossy().to_string();
                        }
                    } else {
                        if let Ok(home) = std::env::var("HOME") {
                            self.current_dir = home;
                        }
                    }
                }

                Ok(CommandResult {
                    command: command.to_string(),
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                })
            }
            Err(e) => Ok(CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}