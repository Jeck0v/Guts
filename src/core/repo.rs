use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Initialise a `.git` Repository in the given Directory
/// Create:
/// - .git/
/// - .git/objects/
/// - .git/refs/heads/
/// - .git/HEAD
/// - .git/config
pub fn init(path: &Path) -> Result<()> {
    let guts_dir = path.join(".git");
    let objects_dir = guts_dir.join("objects");
    let refs_heads_dir = guts_dir.join("refs").join("heads");
    let head_file = guts_dir.join("HEAD");
    let config_file = guts_dir.join("config");

    fs::create_dir_all(&objects_dir).with_context(|| "failed to create objects directory")?;
    fs::create_dir_all(&refs_heads_dir).with_context(|| "failed to create refs/heads directory")?;

    fs::write(&head_file, b"ref: refs/heads/main\n")
        .with_context(|| "failed to write HEAD file")?;

    fs::write(&config_file, b"[core]\n\trepositoryformatversion = 0\n")
        .with_context(|| "failed to write config file")?;

    Ok(())
}
