use crate::core::{ignore::IgnoreMatcher, simple_index};
use anyhow::{anyhow, Result};
use clap::Args;
use std::fs;
use std::path::{PathBuf};

/// Arguments for the `guts add` command
#[derive(Args)]
pub struct AddArgs {
    /// File(s) to add to the staging area
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Current directory for the operation (injected by TUI)
    #[arg(last = true)]
    pub dir: Option<PathBuf>,
}

/// Recursively collect all files from a directory (excludes .git)
fn collect_files_recursively(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if dir.is_file() {
        files.push(dir.clone());
        return Ok(files);
    }

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Ignore .git directory
        if path.file_name().and_then(|s| s.to_str()) == Some(".git") {
            continue;
        }

        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            let mut sub_files = collect_files_recursively(&path)?;
            files.append(&mut sub_files);
        }
    }

    Ok(files)
}

/// Main function for the `guts add` command
/// Adds files to the staging area (index)
pub fn run(args: &AddArgs) -> Result<String> {
    // Determine current directory to use
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

    // Check if we're in a git repository
    if !simple_index::is_git_repository_from(Some(&current_dir))? {
        return Err(anyhow!("fatal: not a git repository"));
    }

    let mut added_files = Vec::new();
    let mut output = String::new();

    // Load .gutsignore matcher
    let matcher = IgnoreMatcher::from_gutsignore(&current_dir)
        .unwrap_or_else(|_| IgnoreMatcher::empty());

    // Process each requested file
    for file_path in &args.files {
        // Support for "." - add all files from current directory
        if file_path.to_string_lossy() == "." {
            let files = collect_files_recursively(&current_dir)?;
            for file in files {
                if matcher.is_ignored(&file, &current_dir) {
                    continue;
                }
                simple_index::add_file_to_index_from(&file, Some(&current_dir))?;
                added_files.push(file.display().to_string());
            }
            continue;
        }

        // Basic checks
        if !file_path.exists() {
            return Err(anyhow!(
                "pathspec '{}' did not match any files",
                file_path.display()
            ));
        }

        if file_path.is_dir() {
            // If it's a directory, add all files recursively
            let files = collect_files_recursively(file_path)?;
            for file in files {
                if matcher.is_ignored(&file, &current_dir) {
                    continue;
                }
                simple_index::add_file_to_index_from(&file, Some(&current_dir))?;
                added_files.push(file.display().to_string());
            }
        } else {
            // Skip if ignored
            if matcher.is_ignored(file_path, &current_dir) {
                continue;
            }
            // Add the file to the JSON index
            simple_index::add_file_to_index_from(file_path, Some(&current_dir))?;
            added_files.push(file_path.display().to_string());
        }
    }

    // Confirmation message
    if added_files.len() == 1 {
        output.push_str(&format!("Added: {}", added_files[0]));
    } else {
        output.push_str(&format!("Added {} files:", added_files.len()));
        for file in &added_files {
            output.push_str(&format!("\n  - {}", file));
        }
    }

    Ok(output)
}
