use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// Reads the content of the HEAD file (or any given ref) and resolves it to a SHA-1 hash.
/// 
/// If the reference is symbolic (e.g. "ref: refs/heads/main"), it follows the reference.
/// Otherwise, it assumes the content is already a SHA and returns it as-is.
pub fn read_head(guts_dir: &Path, head_input: &str) -> Result<String> {
    // Construct the full path to the HEAD file (or another ref passed as input)
    let ref_path = guts_dir.join(head_input);
    
    // Read the content of the ref file (e.g. ".git/HEAD")
    let content = fs::read_to_string(&ref_path)
        .with_context(|| format!("Failed to read reference: {}", head_input))?;

    // If the file contains a symbolic reference like "ref: refs/heads/main"
    if let Some(symbolic) = content.strip_prefix("ref: ") {
        // Construct the path to the actual ref (e.g. ".git/refs/heads/main")
        let real_ref_path = guts_dir.join(symbolic.trim());

        // Read the content of the resolved ref file (which should be the SHA)
        let sha = fs::read_to_string(&real_ref_path)
            .with_context(|| format!("Failed to read resolved ref: {}", symbolic.trim()))?;
        
        // Return the trimmed SHA
        Ok(sha.trim().to_string())
    } else {
        // If the ref is not symbolic, assume it's a SHA and return it directly
        Ok(content.trim().to_string())
    }
}
