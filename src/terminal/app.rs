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
    pub command_history: Vec<CommandResult>,
    pub history_index: usize,
    pub input_history: Vec<String>,
    pub input_history_index: usize,
    pub should_quit: bool,
    pub current_dir: String,
    pub scroll_offset: usize,  // scroll position in history
    pub max_visible_lines: usize,  // max number of lines visible
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
            scroll_offset: 0,
            max_visible_lines: 10,  // default value
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
// ======================= Line & Scroll =======================
    // calc line hysto
    pub fn total_history_lines(&self) -> usize {
        if self.command_history.is_empty() {
            return 4; // Lignes de bienvenue
        }

        let mut total = 0;
        for result in &self.command_history {
            total += 1;
            if !result.output.is_empty() {
                total += result.output.lines().count();
            }
            if let Some(error) = &result.error {
                total += error.lines().count();
            }
            total += 1;
        }
        total
    }

    pub fn scroll_down(&mut self) {
        let total_lines = self.total_history_lines();
        if total_lines > self.max_visible_lines {
            let max_scroll = total_lines - self.max_visible_lines;
            if self.scroll_offset < max_scroll {
                self.scroll_offset += 1;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        let total_lines = self.total_history_lines();
        if total_lines > self.max_visible_lines {
            self.scroll_offset = total_lines - self.max_visible_lines;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn update_visible_lines(&mut self, height: usize) {
        self.max_visible_lines = if height > 8 { height - 6 } else { 2 };
    }
// ======================= EVENT KEY =======================
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
                // Ctrl+Up, scroll up
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.scroll_up();
                } else {
                    if !self.input_history.is_empty() && self.input_history_index > 0 {
                        self.input_history_index -= 1;
                        self.input = self.input_history[self.input_history_index].clone();
                        self.cursor_position = self.input.len();
                    }
                }
            }
            KeyCode::Down => {
                //  Ctrl+Down, scroll down
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.scroll_down();
                } else {
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
            }
            //  fast scroll
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_down();
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
// ======================= EXECUTE COMMANDS =======================
    pub fn execute_command(&mut self) -> Result<()> {
        let command = self.input.trim().to_string();

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
                self.scroll_offset = 0;  // reset scroll position
                return Ok(());
            }
            "pwd" => {
                CommandResult {
                    command: command.clone(),
                    output: self.current_dir.clone(),
                    error: None,
                }
            }
            cmd if cmd.starts_with("cd") => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let target_dir = if parts.len() > 1 {
                    std::path::PathBuf::from(&self.current_dir).join(parts[1])
                } else {
                    std::env::var("HOME").unwrap_or_else(|_| self.current_dir.clone()).into()
                };

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
            cmd if cmd.starts_with("guts ") => self.execute_guts_command(&command)?,
            _ => self.execute_system_command(&command)?,
        };

        self.command_history.push(result);
        self.input.clear();
        self.cursor_position = 0;

        self.scroll_to_bottom(); // Auto-scroll after new command

        Ok(())
    }
// ======================= ONLY GUTS COMMANDS =======================
// Handles `guts` subcommands
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
// ======================= System COMMANDS =======================
// Executes shell/system-level commands
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

                Ok(CommandResult {
                    command: command.to_string(),
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                })
            }
            Err(e) => Ok(CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some(format!("Failed to execute command: {}", e)),
            }),
        }
    }
}