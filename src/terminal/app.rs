use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
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
    pub command_history: Vec<CommandResult>,  // stores results of executed commands
    pub history_index: usize,                 // tracks position in command history (unused here)
    pub input_history: Vec<String>,           // stores raw command inputs for navigation
    pub input_history_index: usize,           // tracks position in input history for Up/Down keys
    pub should_quit: bool,                    // flag to signal app termination
    pub current_dir: String,                  // current working directory for commands
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
            // initialize current_dir with actual current working directory or empty
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

    // handles user key input and updates the input buffer and cursor accordingly
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Ctrl+C triggers quit
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            // enter executes the current input command if not empty
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.execute_command()?;
                }
            }
            // backspace deletes character before cursor
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            // delete removes character at cursor
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                }
            }
            // move cursor left, bounded at 0
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            // move cursor right, bounded at input length
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }
            // navigate up in input history (previous commands)
            KeyCode::Up => {
                if !self.input_history.is_empty() && self.input_history_index > 0 {
                    self.input_history_index -= 1;
                    self.input = self.input_history[self.input_history_index].clone();
                    self.cursor_position = self.input.len();
                }
            }
            // navigate down in input history (next commands)
            KeyCode::Down => {
                if !self.input_history.is_empty() && self.input_history_index < self.input_history.len() - 1 {
                    self.input_history_index += 1;
                    self.input = self.input_history[self.input_history_index].clone();
                    self.cursor_position = self.input.len();
                } else if self.input_history_index == self.input_history.len() - 1 {
                    // Clear input if moved past last history entry
                    self.input_history_index = self.input_history.len();
                    self.input.clear();
                    self.cursor_position = 0;
                }
            }
            // mmove cursor to start of input
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            // move cursor to end of input
            KeyCode::End => {
                self.cursor_position = self.input.len();
            }
            // insert printable character at cursor position
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            _ => {}
        }
        Ok(())
    }

    // parses and executes the current input command string
    pub fn execute_command(&mut self) -> Result<()> {
        let command = self.input.trim().to_string();

        // save command to input history for navigation
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
                // clear screen history and input buffer
                self.command_history.clear();
                self.input.clear();
                self.cursor_position = 0;
                return Ok(());
            }
            "pwd" => {
                // print current directory
                CommandResult {
                    command: command.clone(),
                    output: self.current_dir.clone(),
                    error: None,
                }
            }
            cmd if cmd.starts_with("cd") => {
                // change directory handling, with fallback to HOME if no arg
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let target_dir = if parts.len() > 1 {
                    std::path::PathBuf::from(&self.current_dir).join(parts[1])
                } else {
                    std::env::var("HOME").unwrap_or_else(|_| self.current_dir.clone()).into()
                };

                // try to canonicalize path and update current_dir
                match target_dir.canonicalize() {
                    Ok(path) => {
                        self.current_dir = path.to_string_lossy().to_string();
                        CommandResult {
                            command: command.clone(),
                            output: format!("Changed directory to {}", self.current_dir),
                            error: None,
                        }
                    }
                    Err(e) => CommandResult {
                        command: command.clone(),
                        output: String::new(),
                        error: Some(format!("cd error: {}", e)),
                    },
                }
            }
            "ls" => {
                // list directory contents sorted by name
                match fs::read_dir(&self.current_dir) {
                    Ok(entries) => {
                        let mut lines = vec![];
                        for entry in entries.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                lines.push(name);
                            }
                        }
                        lines.sort();
                        CommandResult {
                            command: command.clone(),
                            output: lines.join("\n"),
                            error: None,
                        }
                    }
                    Err(e) => CommandResult {
                        command: command.clone(),
                        output: String::new(),
                        error: Some(format!("ls error: {}", e)),
                    },
                }
            }
            // handle commands starting with guts via guts-CLI
            cmd if cmd.starts_with("guts ") => self.execute_guts_command(&command)?,
            // fallback to running system command
            _ => self.execute_system_command(&command)?,
        };

        // add result to history and reset input
        self.command_history.push(result);
        self.input.clear();
        self.cursor_position = 0;
        Ok(())
    }

    // executes guts subcommands via CLI parsing and dispatch
    fn execute_guts_command(&mut self, command: &str) -> Result<CommandResult> {
        let args: Vec<&str> = command.split_whitespace().collect();

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
                    Commands::CommitTree(commit_args) => {
                        match guts::commands::commit_tree::run(&commit_args) {
                            Ok(_) => Ok(CommandResult {
                                command: command.to_string(),
                                output: "Commit created successfully".to_string(),
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    // TUI not supported here, return error (should not)
                    Commands::Tui => Ok(CommandResult {
                        command: command.to_string(),
                        output: String::new(),
                        error: Some("Cannot launch TUI".to_string()),
                    }),
                }
            }
            Err(e) => Ok(CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }

    // executes system shell commands via std::process::Command
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
            .current_dir(&self.current_dir)  // run in current directory context
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                // convert the raw stdout bytes to a UTF-8 string (lossy conversion to handle invalid UTF-8)
                // docs stdout: contains the standard output of the child process as a Vec<u8>
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                // convert the raw stderr bytes similarly
                // docs stderr: contains the standard error of the child process as a Vec<u8>
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                // return the CommandResult struct with captured output and possible error
                Ok(CommandResult {
                    command: command.to_string(),
                    output: stdout,
                    // ff stderr is empty, no error, else wrap stderr in Some()
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                })
            }
            Err(e) =>
            // in case the command execution failed (e.g. command not found),
            // return an error message wrapped in CommandResult.error
                Ok(CommandResult {
                    command: command.to_string(),
                    output: String::new(),
                    error: Some(format!("Failed to execute command: {}", e)),
                }),
        }
    }
}
