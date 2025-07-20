use crate::core::simple_index;
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
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
/// Version that compares 3 states: committed (HEAD), staged (index), and working directory
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

    // Load the 3 states
    let committed_files = simple_index::get_committed_files()?; // HEAD commit
    let index = simple_index::SimpleIndex::load()?;             // Staged files
    let work_files = list_working_dir_files(&current_dir)?;     // Working directory

    let mut output = String::new();
    output.push_str("On branch main\n");
    
    if committed_files.is_empty() {
        output.push_str("\nNo commits yet\n");
    } else {
        output.push_str("Your branch is up to date with 'origin/main'.\n");
    }
    output.push_str("\n");

    // Convert working files to relative paths for comparison
    let mut work_files_map = HashMap::new();
    for work_file in &work_files {
        let relative_path = get_relative_path(work_file, &current_dir)?;
        work_files_map.insert(relative_path, work_file.clone());
    }

    // Staged files (in the index)
    let staged_files = &index.files;

    // Collect different types of changes
    let mut staged_changes = Vec::new();
    let mut unstaged_changes = Vec::new();
    let mut untracked_files = Vec::new();

    // Check all files in working directory
    for (work_path, work_file_path) in &work_files_map {
        let committed_hash = committed_files.get(work_path as &str);
        let staged_hash = staged_files.get(work_path as &str);

        match (committed_hash, staged_hash) {
            (None, None) => {
                // File not committed and not staged -> untracked
                untracked_files.push(work_path.clone());
            }
            (None, Some(_)) => {
                // File not committed but staged -> new file staged
                staged_changes.push((work_path.clone(), "new file"));
            }
            (Some(committed_hash), Some(staged_hash)) => {
                // File committed and staged
                if committed_hash != staged_hash {
                    // Staged version different from committed -> modified staged
                    staged_changes.push((work_path.clone(), "modified"));
                }
                // If they're the same, no change to report for staged
            }
            (Some(committed_hash), None) => {
                // File committed but not staged -> check if working file changed
                let work_hash = calculate_file_hash(work_file_path)?;
                if &work_hash != committed_hash {
                    // Working file different from committed -> modified unstaged
                    unstaged_changes.push((work_path.clone(), "modified"));
                }
                // If they're the same, file is clean
            }
        }
    }

    // Check for deleted files (committed or staged but not in working directory)
    for file_path in committed_files.keys() {
        if !work_files_map.contains_key(file_path) {
            if staged_files.contains_key(file_path) {
                // File deleted but still in index
                staged_changes.push((file_path.clone(), "deleted"));
            } else {
                // File deleted and not in index
                unstaged_changes.push((file_path.clone(), "deleted"));
            }
        }
    }

    // Check for staged files that don't exist in working directory
    for file_path in staged_files.keys() {
        if !work_files_map.contains_key(file_path) && !committed_files.contains_key(file_path) {
            staged_changes.push((file_path.clone(), "deleted"));
        }
    }

    // 1. Staged changes (changes to be committed)
    if !staged_changes.is_empty() {
        output.push_str("Changes to be committed:\n");
        output.push_str("  (use \"git reset HEAD <file>...\" to unstage)\n");
        for (file_path, change_type) in &staged_changes {
            output.push_str(&format!("        {}:   {}\n", change_type, file_path));
        }
        output.push_str("\n");
    }

    // 2. Unstaged changes (changes not staged for commit)
    if !unstaged_changes.is_empty() {
        output.push_str("Changes not staged for commit:\n");
        output.push_str("  (use \"git add <file>...\" to update what will be committed)\n");
        output.push_str("  (use \"git checkout -- <file>...\" to discard changes in working directory)\n");
        for (file_path, change_type) in &unstaged_changes {
            output.push_str(&format!("        {}:   {}\n", change_type, file_path));
        }
        output.push_str("\n");
    }

    // 3. Untracked files
    if !untracked_files.is_empty() {
        output.push_str("Untracked files:\n");
        output.push_str("  (use \"git add <file>...\" to include in what will be committed)\n");
        for file in &untracked_files {
            output.push_str(&format!("        {}\n", file));
        }
        output.push_str("\n");
    }

    // Final message if everything is clean
    if staged_changes.is_empty() && unstaged_changes.is_empty() && untracked_files.is_empty() {
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

/// Calculate the Git blob hash for a file (same as what `guts add` would generate)
fn calculate_file_hash(file_path: &PathBuf) -> Result<String> {
    use crate::core::{blob, hash};
    use std::fs;
    
    let content = fs::read(file_path)?;
    let blob = blob::Blob::new(content);
    hash::write_object(&blob)
}
