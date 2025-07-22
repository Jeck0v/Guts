use anyhow::Result;
use std::path::Path;
use std::fs;

// Tries to resolve a reference name (like "main") to its corresponding commit SHA
pub fn resolve_ref(guts_dir: &Path, head_input: &str) -> Result<String> {
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