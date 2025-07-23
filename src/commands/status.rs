use crate::core::{ignore::IgnoreMatcher, simple_index, read_head};
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::path::{PathBuf};
use walkdir::WalkDir;

/// CLI arguments for the `status` command.
#[derive(Args)]
pub struct StatusObject {
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

/// Entry point for the `guts status` command
pub fn run(args: &StatusObject) -> Result<String> {
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

    if !simple_index::is_git_repository_from(Some(&current_dir))? {
        return Ok("fatal: not a git repository".to_string());
    }

    let matcher = IgnoreMatcher::from_gutsignore(&current_dir)
        .unwrap_or_else(|_| IgnoreMatcher::empty());

    let committed_files = simple_index::get_committed_files_from(Some(&current_dir))?;
    let index = simple_index::SimpleIndex::load_from(Some(&current_dir))?;
    let work_files = list_working_dir_files(&current_dir, &matcher)?;

    let current_branch = read_head::get_current_branch()
        .unwrap_or_else(|_| "main".to_string());
    
    let mut output = String::new();
    output.push_str(&format!("On branch {}\n", current_branch));

    if committed_files.is_empty() {
        output.push_str("\nNo commits yet\n");
    }
    output.push_str("\n");

    let mut work_files_map = HashMap::new();
    for work_file in &work_files {
        let relative_path = get_relative_path(work_file, &current_dir)?;
        work_files_map.insert(relative_path, work_file.clone());
    }

    let staged_files = &index.files;
    let mut staged_changes = Vec::new();
    let mut unstaged_changes = Vec::new();
    let mut untracked_files = Vec::new();

    for (work_path, work_file_path) in &work_files_map {
        let committed_hash = committed_files.get(work_path as &str);
        let staged_hash = staged_files.get(work_path as &str);

        match (committed_hash, staged_hash) {
            (None, None) => {
                untracked_files.push(work_path.clone());
            }
            (None, Some(_)) => {
                staged_changes.push((work_path.clone(), "new file"));
            }
            (Some(committed_hash), Some(staged_hash)) => {
                if committed_hash != staged_hash {
                    staged_changes.push((work_path.clone(), "modified"));
                }
            }
            (Some(committed_hash), None) => {
                let work_hash = calculate_file_hash(work_file_path)?;
                if &work_hash != committed_hash {
                    unstaged_changes.push((work_path.clone(), "modified"));
                }
            }
        }
    }

    for file_path in committed_files.keys() {
        if !work_files_map.contains_key(file_path) {
            if staged_files.contains_key(file_path) {
                staged_changes.push((file_path.clone(), "deleted"));
            } else {
                unstaged_changes.push((file_path.clone(), "deleted"));
            }
        }
    }

    for file_path in staged_files.keys() {
        if !work_files_map.contains_key(file_path) && !committed_files.contains_key(file_path) {
            staged_changes.push((file_path.clone(), "deleted"));
        }
    }

    if !staged_changes.is_empty() {
        output.push_str("Changes to be committed:\n");
        output.push_str("  (use \"git reset HEAD <file>...\" to unstage)\n");
        for (file_path, change_type) in &staged_changes {
            output.push_str(&format!("        {}:   {}\n", change_type, file_path));
        }
        output.push_str("\n");
    }

    if !unstaged_changes.is_empty() {
        output.push_str("Changes not staged for commit:\n");
        output.push_str("  (use \"git add <file>...\" to update what will be committed)\n");
        output.push_str("  (use \"git checkout -- <file>...\" to discard changes in working directory)\n");
        for (file_path, change_type) in &unstaged_changes {
            output.push_str(&format!("        {}:   {}\n", change_type, file_path));
        }
        output.push_str("\n");
    }

    if !untracked_files.is_empty() {
        output.push_str("Untracked files:\n");
        output.push_str("  (use \"git add <file>...\" to include in what will be committed)\n");
        for file in &untracked_files {
            output.push_str(&format!("        {}\n", file));
        }
        output.push_str("\n");
    }

    if staged_changes.is_empty() && unstaged_changes.is_empty() && untracked_files.is_empty() {
        output.push_str("nothing to commit, working tree clean\n");
    }

    Ok(output)
}

/// List all working directory files, excluding ignored and .git files
fn list_working_dir_files(current_dir: &PathBuf, matcher: &IgnoreMatcher) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let walker = WalkDir::new(current_dir).into_iter().filter_entry(|e| {
        let path = e.path();

        // Skip .git and anything ignored
        if path.components().any(|c| c.as_os_str() == ".git") {
            return false;
        }

        !matcher.is_ignored(path, &current_dir)
    });

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() && !matcher.is_ignored(entry.path(), &current_dir) {
            files.push(entry.into_path());
        }
    }

    Ok(files)
}

fn get_relative_path(file_path: &PathBuf, current_dir: &PathBuf) -> Result<String> {
    // Find repo root from current directory context
    let repo_root = simple_index::find_repo_root_from(Some(current_dir))?;
    let relative = file_path
        .strip_prefix(&repo_root)
        .map_err(|_| anyhow::anyhow!("file is not in the repository"))?;
    Ok(relative.to_string_lossy().to_string())
}

fn calculate_file_hash(file_path: &PathBuf) -> Result<String> {
    use crate::core::{blob, hash};
    use std::fs;

    let content = fs::read(file_path)?;
    let blob = blob::Blob::new(content);
    hash::write_object(&blob)
}
