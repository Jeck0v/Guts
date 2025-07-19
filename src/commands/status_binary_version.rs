use std::collections::HashSet;
use std::path::PathBuf;
use anyhow::{Result};
use clap::Args;

use crate::core::status_binary_index;

/// CLI arguments for the `status` command.
#[derive(Args)]
pub struct StatusObject {
    // Optional custom path to the .git directory (defaults to current/.git)
    pub guts_dir: Option<PathBuf>,
}

/// Entry point for the `gut status` command.
pub fn run(args: &StatusObject) -> Result<String> {
    // Determine the path to the .git directory (or .guts if used)
    let guts_dir = args
        .guts_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir()
            .expect("failed to get the current directory")
            .join(".git"));

    // Validate the path exists
    if !guts_dir.exists() {
        return Err(anyhow::anyhow!(
            "Path {:?} does not exist",
            guts_dir.display()
        ));
    }

    // Get all files in the working directory (excluding .git/.guts)
    let work_files = status_binary_index::list_working_dir_files(
        &std::env::current_dir().expect("failed to get the current directory")
    )?;

    // Parse the .git/index to get the list of tracked files
    let index_entries = status_binary_index::parse_git_index(&guts_dir)?;

    // Compare the working directory files to the index to find modified/deleted files
    let modified_files = status_binary_index::is_modified(&index_entries, &guts_dir)?;

    // Create a set of working directory paths for fast lookup
    let work_files_set: HashSet<_> = work_files.iter().collect();

    let mut output = String::new();
    output.push_str("On branch main\n");
    output.push_str("Your branch is up to date with 'origin/main'.\n");
    output.push_str("\n");

    // Detect untracked files (present in working dir but not in index)
    // if !work_files.is_empty() {
    //     output.push_str("Untracked files:\n");
    //     output.push_str("  (use \"git add <file>...\" to include in what will be committed)\n");
    //     for file in &work_files {
    //         if !index_entries.iter().any(|entry| entry.path == *file) {
    //             output.push_str(&format!("        {}\n", file.display()));
    //         }
    //     }
    //     output.push_str("\n");
    // }

    
    // Show files that have been modified since last index
    if !modified_files.is_empty() {
        output.push_str("Changes not staged for commit:\n");
        output.push_str("  (use \"git add <file>...\" to update what will be committed)\n");
        for file in &modified_files {
            output.push_str(&format!("        modified:   {}\n", file.display()));
        }
        output.push_str("\n");
    }

    // Show files that are in the index but missing in the working directory (deleted)
    for entry in &index_entries {
        if !work_files_set.contains(&entry.path) {
            output.push_str(&format!("Deleted : {}\n", entry.path.display()));
        }
    }

    Ok(output)
}