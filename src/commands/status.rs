use crate::core::simple_index;
use anyhow::Result;
use clap::Args;
use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

/// CLI arguments for the `status` command.
#[derive(Args)]
pub struct StatusObject {
    /// Optional: custom path to .git (default current/.git)
    pub guts_dir: Option<PathBuf>,
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

/// Entry point for the `guts status` command
/// Version adapted for simple JSON index
pub fn run(args: &StatusObject) -> Result<String> {
    // Determine current directory to use
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

    // Check if we're in a git repository
    if !simple_index::is_git_repository()? {
        return Ok("fatal: not a git repository".to_string());
    }

    // Load our simple JSON index
    let index = simple_index::SimpleIndex::load()?;

    // List all files in the working directory (excluding .git)
    let work_files = list_working_dir_files(&current_dir)?;

    let mut output = String::new();
    output.push_str("On branch main\n");
    output.push_str("Your branch is up to date with 'origin/main'.\n");
    output.push_str("\n");

    // Create a set of working directory files for fast lookup
    let work_files_set: HashSet<_> = work_files.iter().collect();

    // Staged files (in the index)
    let staged_files: Vec<&String> = index.get_staged_files();

    // 1. Files staged for commit
    if !staged_files.is_empty() {
        output.push_str("Changes to be committed:\n");
        output.push_str("  (use \"git reset HEAD <file>...\" to unstage)\n");
        for file_path in &staged_files {
            output.push_str(&format!("        new file:   {}\n", file_path));
        }
        output.push_str("\n");
    }

    // 2. Untracked files (present in working dir but not in index)
    let mut untracked_files = Vec::new();
    for work_file in &work_files {
        let relative_path = get_relative_path(work_file, &current_dir)?;
        if !index.contains_file(&relative_path) {
            untracked_files.push(relative_path);
        }
    }

    if !untracked_files.is_empty() {
        output.push_str("Untracked files:\n");
        output.push_str("  (use \"git add <file>...\" to include in what will be committed)\n");
        for file in &untracked_files {
            output.push_str(&format!("        {}\n", file));
        }
        output.push_str("\n");
    }

    // 3. Deleted files (in index but not in working dir)
    for staged_file in &staged_files {
        let full_path = current_dir.join(staged_file);
        if !work_files_set.contains(&full_path) {
            output.push_str(&format!("        deleted:    {}\n", staged_file));
        }
    }

    // Final message if everything is clean
    if staged_files.is_empty() && untracked_files.is_empty() {
        output.push_str("nothing to commit, working tree clean\n");
    }

    Ok(output)
}

/// Recursively list all files in the working directory, excluding .git
fn list_working_dir_files(current_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let walker = WalkDir::new(current_dir).into_iter().filter_entry(|e| {
        // Exclude .git directory
        !e.path().components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s == ".git"
        })
    });

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }

    Ok(files)
}

/// Convert an absolute path to a relative path from the repo root
fn get_relative_path(file_path: &PathBuf, current_dir: &PathBuf) -> Result<String> {
    let relative = file_path
        .strip_prefix(current_dir)
        .map_err(|_| anyhow::anyhow!("file is not in the current directory"))?;
    Ok(relative.to_string_lossy().to_string())
}
