use std::path::Path;
use std::fs;
use clap::Args;
use anyhow::{Context, Result};
use crate::core::read_head::read_head; 

// CLI arguments for the `rev-parse` command
#[derive(Args)]
pub struct RevParse {
    // The reference to resolve (e.g., "HEAD", "main", a SHA hash)
    pub head: String,
}

// Checks whether the input string looks like a full SHA-1 hash (40 hex digits)
fn looks_like_sha(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

// Tries to resolve a reference name (like "main") to its corresponding commit SHA
fn resolve_ref(guts_dir: &Path, head_input: &str) -> Result<String> {
    let paths_to_try = [
        // Try resolving as a local branch: .git/refs/heads/<name>
        guts_dir.join("refs").join("heads").join(head_input),
        // Try resolving as a tag: .git/refs/tags/<name>
        guts_dir.join("refs").join("tags").join(head_input),
        // Fallback: directly try the raw path inside .git/
        guts_dir.join(head_input),
    ];

    // Check each path, return the SHA if found
    for path in paths_to_try {
        if path.exists() {
            let sha = fs::read_to_string(path)?.trim().to_string();
            return Ok(sha);
        }
    }

    // If none match, return an error
    Err(anyhow::anyhow!("Reference '{}' not found", head_input))
}

// Main entry point for `gut rev-parse` command
pub fn run(head_input: &RevParse) -> Result<String> {
    // Determine the path to the .git directory
    let current_dir = std::env::current_dir().context("Cannot get current directory")?;
    let gits_dir = current_dir.join(".git"); 

    match head_input.head.as_str() {
        // If the user requested "HEAD", resolve it with read_head()
        "HEAD" => {
            let sha = read_head(&gits_dir, &head_input.head)?; 
            Ok(sha)
        }

        // If it looks like a valid SHA, return it directly
        s if looks_like_sha(s) => {
            Ok(s.to_string())
        }

        // Otherwise, try to resolve the ref (e.g., a branch name)
        other => resolve_ref(&gits_dir, other)
    }
}
