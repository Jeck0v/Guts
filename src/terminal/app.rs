use anyhow::Result;
use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use guts::cli::{Cli, Commands};
use std::process::{Command, Stdio};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;


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
    pub force_redraw: bool,
    pub last_executed_command: Option<String>

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
            force_redraw: false,
            last_executed_command: None
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
            "cd", "ls", "pwd", "clear", "exit", "quit", "nano", "vim", "vi",
            "guts", "guts init", "guts hash-object", "guts cat-file", "guts write-tree",
            "guts commit-tree", "guts ls-tree", "guts rm", "guts add", "guts status",
            "guts commit", "guts log", "guts ls-files", "guts show-ref",
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
                    self.update_autocomplete();
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                    self.update_autocomplete();
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
                } else {
                    self.update_autocomplete();
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ======================= Helper method =======================
    fn finalize_command(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.scroll_to_bottom();
    }

    // ======================= EXECUTE COMMANDS =======================
    pub fn execute_command(&mut self) -> Result<()> {
        let command = self.input.trim().to_string();
        self.last_executed_command = Some(command.clone());


        if !command.is_empty() {
            self.input_history.push(command.clone());
            self.input_history_index = self.input_history.len();
        }

        // interne command
        if command == "exit" || command == "quit" {
            self.should_quit = true;
            return Ok(());
        }

        if command == "clear" {
            self.command_history.clear();
            self.finalize_command();
            self.scroll_offset = 0;
            return Ok(());
        }

        if command.starts_with("cd") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            let target_dir = if parts.len() > 1 {
                std::path::PathBuf::from(&self.current_dir).join(parts[1])
            } else {
                std::env::var("HOME")
                    .unwrap_or_else(|_| self.current_dir.clone())
                    .into()
            };

            let result = match target_dir.canonicalize() {
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
            };

            let result = self.handle_cd_command(&command);
            self.command_history.push(result);
            self.finalize_command();
            return Ok(());
        }

        if command.starts_with("guts ") {
            let result = self.execute_guts_command(&command)?;
            self.command_history.push(result);
            self.finalize_command();
            return Ok(());
        }

        // editor nano/vim/vi
        if command.starts_with("nano") || command.starts_with("vim") || command.starts_with("vi") {
            return Ok(());
        }

        // Sinon, commande système via shell
        let _cleaned_dir = if self.current_dir.starts_with(r"\\?\") {
        // sys command
        let result = self.execute_shell_command(&command);
        self.command_history.push(result);
        self.finalize_command();

        Ok(())
    }

    // ======================= CD Command Handler =======================
    fn handle_cd_command(&mut self, command: &str) -> CommandResult {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let target_dir = if parts.len() > 1 {
            std::path::PathBuf::from(&self.current_dir).join(parts[1])
        } else {
            std::env::var("HOME").unwrap_or_else(|_| self.current_dir.clone()).into()
        };

        match target_dir.canonicalize() {
            Ok(path) => {
                self.current_dir = path.to_string_lossy().to_string();
                CommandResult {
                    command: command.to_string(),
                    output: format!("Changed directory to {}", self.current_dir),
                    error: None,
                }
            }
            Err(e) => CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some(format!("cd error: {}", e)),
            },
        }
    }

    // ======================= Editor Handler =======================

    pub fn handle_editor_command(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        command: &str,
    ) -> Result<()> {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
        use std::io::{self, Write};
        use std::path::PathBuf;
        use std::process::Command;

        // out of the terminal
        terminal.clear()?; // clear tui
        drop(terminal);
        disable_raw_mode()?; // out raw mode

        // clear terminal
        print!("\x1B[2J\x1B[H\x1B[?25h"); // Clear + move cursor + show cursor
        io::stdout().flush().unwrap();

        // command parse
        let parts: Vec<&str> = command.split_whitespace().collect();
        let editor = parts[0];
        let args = &parts[1..];

        // fix bug onedrive
        let mut safe_dir = PathBuf::from(&self.current_dir);
        if safe_dir.to_string_lossy().to_lowercase().contains("onedrive") {
            if let Some(doc_dir) = dirs::document_dir() {
                safe_dir = doc_dir;
            } else {
                safe_dir = std::env::temp_dir();
            }
        }

        // launch editor
        let status = if cfg!(target_os = "windows") {
            let full_command = format!("{} {}", editor, args.join(" "));
            Command::new("cmd")
                .args(&["/C", &full_command])
                .current_dir(&safe_dir)
                .status()
        } else {
            let mut cmd = Command::new(editor);
            cmd.args(args).current_dir(&safe_dir);
            cmd.status()
        };

        let result = match status {
            Ok(exit_status) => {
                let message = if exit_status.success() {
                    format!("Editor {} exited successfully", editor)
                } else {
                    format!(
                        "Editor {} exited with code: {}",
                        editor,
                        exit_status.code().unwrap_or(-1)
                    )
                };
                CommandResult {
                    command: command.to_string(),
                    output: message,
                    error: None,
                }
            }
            Err(e) => CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some(format!("Failed to launch {}: {}", editor, e)),
            },
        };

        // add historic
        self.command_history.push(result);
        self.finalize_command();

        Ok(())
    }

    // ======================= Shell Command Handler =======================
    fn execute_shell_command(&self, command: &str) -> CommandResult {
        let cleaned_dir = if self.current_dir.starts_with(r"\\?\") {
            self.current_dir.trim_start_matches(r"\\?\\").to_string()
        } else {
            self.current_dir.clone()
        };

        #[cfg(target_os = "windows")]
        let shell_result = Command::new("powershell")
            .arg("-Command")
            .arg(command)
            .current_dir(&cleaned_dir)
            .output();

        #[cfg(not(target_os = "windows"))]
        let shell_result = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.current_dir)
            .output();

        match shell_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let combined_output = if !stderr.is_empty() {
                    format!("{}\n{}", stdout, stderr)
                } else {
                    stdout
                };

                CommandResult {
                    command: command.to_string(),
                    output: combined_output.trim().to_string(),
                    error: None,
                }
            }
            Err(e) => CommandResult {
                command: command.to_string(),
                output: String::new(),
                error: Some(format!("Execution failed: {}", e)),
            },
        }
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
                                error: Some(e.to_string()),
                                output: String::new(),
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
                    Commands::LsTree(mut ls_tree_args) => {
                        ls_tree_args.dir = Some(std::path::PathBuf::from(&self.current_dir));
                        match guts::commands::ls_tree::run(&ls_tree_args) {
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
                    Commands::LsFiles(ls_files_args) => {
                        match guts::commands::ls_files::run(&ls_files_args) {
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
                        error: Some("Cannot launch TUI from within TUI".to_string()),
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
