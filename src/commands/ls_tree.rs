use crate::core::cat;
use anyhow::{anyhow, Result};
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LsTreeArgs {
    /// Tree SHA to list contents of
    pub tree_sha: String,
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

pub fn run(args: &LsTreeArgs) -> Result<String> {
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("could not get the current dir"));

    let git_dir = current_dir.join(".git");

    if !git_dir.exists() {
        return Err(anyhow!("fatal: not a git repository"));
    }

    // Get the object path
    let object_path = cat::get_object_path(&git_dir, &args.tree_sha);

    if !object_path.exists() {
        return Err(anyhow!("fatal: not a valid object name {}", args.tree_sha));
    }

    // Read and parse the object
    let object_data = std::fs::read(&object_path)?;
    let parsed_object = cat::parse_object(&object_data)?;

    match parsed_object {
        cat::ParsedObject::Tree(entries) => {
            let mut output = Vec::new();

            for entry in entries {
                // Convert 20-byte hash to hex string
                let hash_hex = hex::encode(&entry.hash);
                
                // Format: <mode> <type> <hash><TAB><name>
                // We need to determine the object type (blob/tree) from the mode
                let object_type = if entry.mode.starts_with("040") {
                    "tree"
                } else {
                    "blob"
                };

                output.push(format!("{} {} {}\t{}", entry.mode, object_type, hash_hex, entry.name));
            }

            Ok(output.join("\n"))
        }
        _ => Err(anyhow!("fatal: not a tree object")),
    }
}
