use crate::core::simple_index;
use anyhow::{anyhow, Result};
use clap::Args;
use std::fs;
use std::path::PathBuf;

/// Arguments for the `guts rm` command
#[derive(Args)]
pub struct RmArgs {
    /// File(s) to remove from working directory and index
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
    /// Current directory for the operation (injected by TUI)
    #[arg(last = true)]
    pub dir: Option<PathBuf>,
}

/// Convert absolute path to relative path from repo root
fn get_relative_path(file_path: &PathBuf) -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let repo_root = simple_index::find_repo_root()?;
    
    let absolute_path = if file_path.is_absolute() {
        file_path.clone()
    } else {
        current_dir.join(file_path)
    };
    
    let relative = absolute_path.strip_prefix(&repo_root)
        .map_err(|_| anyhow!("file is not in the repository"))?;
    Ok(relative.to_string_lossy().to_string())
}

/// Remove a file from the index
fn remove_file_from_index(file_path: &PathBuf) -> Result<bool> {
    let mut index = simple_index::SimpleIndex::load()?;
    let relative_path = get_relative_path(file_path)?;
    
    if index.files.remove(&relative_path).is_some() {
        index.save()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Main function for the `guts rm` command
/// Removes files from working directory and index
pub fn run(args: &RmArgs) -> Result<String> {
    // Check if we're in a git repository
    if !simple_index::is_git_repository()? {
        return Err(anyhow!("fatal: not a git repository"));
    }

    let mut removed_files = Vec::new();
    let mut output = String::new();

    // Process each requested file
    for file_path in &args.files {
        // Basic checks
        if !file_path.exists() {
            return Err(anyhow!(
                "pathspec '{}' did not match any files",
                file_path.display()
            ));
        }

        if file_path.is_dir() {
            return Err(anyhow!(
                "fatal: not removing '{}' recursively without -r",
                file_path.display()
            ));
        }

        // Remove from index
        let was_in_index = remove_file_from_index(file_path)?;
        
        if !was_in_index {
            return Err(anyhow!(
                "fatal: pathspec '{}' did not match any files",
                file_path.display()
            ));
        }

        // Remove from working directory
        fs::remove_file(file_path)
            .map_err(|e| anyhow!("failed to remove '{}': {}", file_path.display(), e))?;

        removed_files.push(file_path.display().to_string());
    }

    // Confirmation message
    if removed_files.len() == 1 {
        output.push_str(&format!("rm '{}'", removed_files[0]));
    } else {
        for file in &removed_files {
            output.push_str(&format!("rm '{}'\n", file));
        }
        output.pop(); // Remove last newline
    }

    Ok(output)
}