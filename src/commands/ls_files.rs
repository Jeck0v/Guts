use crate::core::simple_index;
use anyhow::Result;
use clap::Args;
use std::collections::HashSet;

/// Arguments for the `guts ls-files` command
#[derive(Args)]
pub struct LsFilesArgs {
    // Placeholder for future options if needed
}

/// List all files in the index
pub fn run(_args: &LsFilesArgs) -> Result<String> {
    // Get all tracked files (both from current index and from last commit)
    let mut tracked_files = HashSet::new();
    
    // Get currently staged files
    let index = simple_index::SimpleIndex::load()?;
    for file_path in index.get_staged_files() {
        tracked_files.insert(file_path.clone());
    }
    
    // Get files from the last commit
    match simple_index::get_committed_files() {
        Ok(committed_files) => {
            for file_path in committed_files.keys() {
                tracked_files.insert(file_path.clone());
            }
        },
        Err(_) => {
            // No commits yet, only show staged files
        }
    }
    
    if tracked_files.is_empty() {
        return Ok(String::new());
    }
    
    // Sort the files for consistent output
    let mut sorted_files: Vec<String> = tracked_files.into_iter().collect();
    sorted_files.sort();
    
    // Join all files with newlines
    let output = sorted_files.join("\n");
    
    Ok(output)
}
