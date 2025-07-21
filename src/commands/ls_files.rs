use crate::core::simple_index;
use anyhow::Result;
use clap::Args;

/// Arguments for the `guts ls-files` command
#[derive(Args)]
pub struct LsFilesArgs {
    // Placeholder for future options if needed
}

/// List all files in the index
pub fn run(_args: &LsFilesArgs) -> Result<String> {
    let index = simple_index::SimpleIndex::load()?;
    
    let staged_files = index.get_staged_files();
    
    if staged_files.is_empty() {
        return Ok(String::new());
    }
    
    // Sort the files for consistent output
    let mut sorted_files: Vec<&String> = staged_files;
    sorted_files.sort();
    
    // Join all files with newlines
    let output = sorted_files
        .iter()
        .map(|f| f.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    
    Ok(output)
}
