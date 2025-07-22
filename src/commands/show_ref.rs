use anyhow::Result;
use clap::Args;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// CLI arguments for the `show-ref` command.
#[derive(Args)]
pub struct ShowRefArgs {
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

/// Entry point for the `guts show-ref` command
/// Lists all refs and their hashes
pub fn run(args: &ShowRefArgs) -> Result<String> {
    // Determine current directory to use
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

    // Find .git directory
    let git_dir = current_dir.join(".git");

    if !git_dir.exists() {
        return Ok("fatal: not a git repository".to_string());
    }

    let refs_dir = git_dir.join("refs");
    if !refs_dir.exists() {
        return Ok("".to_string()); // No refs yet
    }

    let mut output = String::new();
    let mut refs = HashSet::new();

    // Walk through all refs directories (heads, remotes, tags)
    let walker = WalkDir::new(&refs_dir).into_iter().filter_entry(|e| {
        e.file_type().is_file() || e.file_type().is_dir()
    });

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            let ref_path = entry.path();
            let content = fs::read_to_string(ref_path)?;
            let content = content.trim();

            // Get relative path from refs/
            let relative_path = ref_path
                .strip_prefix(&refs_dir)
                .map_err(|_| anyhow::anyhow!("Failed to get relative path"))?;

            let ref_name = format!("refs/{}", relative_path.to_string_lossy());

            // Handle symbolic refs (like remotes/origin/HEAD)
            if content.starts_with("ref: ") {
                let target_ref = content.strip_prefix("ref: ").unwrap();
                let target_file = git_dir.join(target_ref);
                if target_file.exists() {
                    if let Ok(target_hash) = fs::read_to_string(target_file) {
                        refs.insert((target_hash.trim().to_string(), ref_name));
                    }
                }
            } else {
                // Direct hash reference
                refs.insert((content.to_string(), ref_name));
            }
        }
    }

    // Don't include HEAD separately as it usually points to another ref
    // and would be duplicated

    // Convert HashSet to Vec and sort by name for consistent output
    let mut refs_vec: Vec<(String, String)> = refs.into_iter().collect();
    refs_vec.sort_by(|a, b| a.1.cmp(&b.1));

    // Format output: hash ref_name
    for (hash, ref_name) in refs_vec {
        output.push_str(&format!("{} {}\n", hash, ref_name));
    }

    Ok(output)
}