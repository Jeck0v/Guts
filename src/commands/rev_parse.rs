use clap::Args;
use anyhow::{Context, Result};
use crate::core::read_head::read_head; 
use crate::core::resolve_parse::resolve_ref;

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
