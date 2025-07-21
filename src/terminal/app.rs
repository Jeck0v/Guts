use anyhow::Result;
use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use guts::cli::{Cli, Commands};
use std::fs;
use std::process::{Command, Stdio};

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
    pub input_history: Vec<String>,
    pub input_history_index: usize,
    pub should_quit: bool,
    pub current_dir: String,
    pub scroll_offset: usize,           // scroll position in history
    pub max_visible_lines: usize,       // max number of lines visible
    pub autocomplete_list: Vec<String>, // auto complete
    pub show_autocomplete: bool,
    pub autocomplete_index: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            command_history: Vec::new(),
            input_history: Vec::new(),
            input_history_index: 0,
            should_quit: false,
            current_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            scroll_offset: 0,
            max_visible_lines: 10, // default value
            autocomplete_list: Vec::new(),
            show_autocomplete: false,
            autocomplete_index: 0,
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
            return 4;
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

    // ================= Auto complete: helpers =================
    fn update_autocomplete(&mut self) {
        use std::collections::HashSet;

        self.autocomplete_list.clear();
        self.show_autocomplete = false;

        if self.input.is_empty() {
            return;
        }

        let mut suggestions = HashSet::new();

        for history in &self.input_history {
            if history.starts_with(&self.input) {
                suggestions.insert(history.clone());
            }
        }

        // basic command
        let basic_cmds = vec![
            "cd",
            "ls",
            "pwd",
            "clear",
            "exit",
            "quit",
            "guts",
            "guts init",
            "guts hash-object",
            "guts cat-file",
            "guts write-tree",
            "guts commit-tree",
            "guts rm",
            "guts add",
            "guts status",
            "guts commit",
            "guts log",
            "guts show-ref",
        ];
        for cmd in basic_cmds {
            if cmd.starts_with(&self.input) {
                suggestions.insert(cmd.to_string());
            }
        }

        let mut sorted: Vec<String> = suggestions.into_iter().collect();
        sorted.sort();

        if !sorted.is_empty() {
            self.autocomplete_list = sorted;
            self.show_autocomplete = true;
            self.autocomplete_index = 0;
        }
    }

    fn apply_autocomplete(&mut self) {
        if self.show_autocomplete && !self.autocomplete_list.is_empty() {
            if let Some(suggestion) = self.autocomplete_list.get(self.autocomplete_index) {
                self.input = suggestion.clone();
                self.cursor_position = self.input.len();
                self.show_autocomplete = false;
            }
        }
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
                    if !self.input_history.is_empty()
                        && self.input_history_index < self.input_history.len() - 1
                    {
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
                self.update_autocomplete();
            }
            KeyCode::Tab => {
                if self.show_autocomplete {
                    self.apply_autocomplete();
                }
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
                self.scroll_offset = 0; // reset scroll position
                return Ok(());
            }
            "pwd" => CommandResult {
                command: command.clone(),
                output: self.current_dir.clone(),
                error: None,
            },
            cmd if cmd.starts_with("cd") => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let target_dir = if parts.len() > 1 {
                    std::path::PathBuf::from(&self.current_dir).join(parts[1])
                } else {
                    std::env::var("HOME")
                        .unwrap_or_else(|_| self.current_dir.clone())
                        .into()
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
            "ls" => match fs::read_dir(&self.current_dir) {
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
            },
            cmd if cmd.starts_with("guts ") => self.execute_guts_command(&command)?,
            _ => self.execute_system_command(&command)?,
        };

        self.command_history.push(result);
        self.input.clear();
        self.cursor_position = 0;

        self.scroll_to_bottom(); // Auto-scroll after new command
        Ok(())
    }
    // ======================= Handles only guts subcommands =======================
    fn execute_guts_command(&mut self, command: &str) -> Result<CommandResult> {
        let args: Vec<&str> = command.split_whitespace().collect();

        match Cli::try_parse_from(args) {
            Ok(cli) => {
                match cli.command {
                    Commands::Init(mut init_args) => {
                        // Use TUI current directory if no directory specified
                        if init_args.dir.is_none() {
                            init_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        }
                        match guts::commands::init::run(&init_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::HashObject(mut hash_args) => {
                        // Inject current TUI directory
                        hash_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::hash_object::run(&hash_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::CatFile(mut cat_args) => {
                        // Inject current TUI directory
                        cat_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::cat_file::run(&cat_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::WriteTree(mut tree_args) => {
                        // Inject current TUI directory
                        tree_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::write_tree::run(&tree_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::CommitTree(mut commit_args) => {
                        // Inject current TUI directory
                        commit_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::commit_tree::run(&commit_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::Status(mut status_args) => {
                        // Inject current TUI directory
                        status_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::status::run(&status_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::Add(mut add_args) => {
                        // Inject current TUI directory
                        add_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::add::run(&add_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::Rm(mut rm_args) => {
                        // Inject current TUI directory
                        rm_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::rm::run(&rm_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::Commit(mut commit_args) => {
                        // Inject current TUI directory
                        commit_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::commit::run(&commit_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::RevParse(rev_parse_args) => {
                        match guts::commands::rev_parse::run(&rev_parse_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::Log(mut log_args) => {
                        // Inject current TUI directory
                        log_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::log::run(&log_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
                                error: None,
                            }),
                            Err(e) => Ok(CommandResult {
                                command: command.to_string(),
                                output: String::new(),
                                error: Some(e.to_string()),
                            }),
                        }
                    }
                    Commands::ShowRef(mut show_ref_args) => {
                        // Inject current TUI directory
                        show_ref_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::show_ref::run(&show_ref_args) {
                            Ok(out) => Ok(CommandResult {
                                command: command.to_string(),
                                output: out,
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
                    error: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
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
