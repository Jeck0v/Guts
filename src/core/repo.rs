use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Initialise a `.guts` Repository in the given Directory
/// Create:
/// - .guts/
/// - .guts/objects/
/// - .guts/refs/heads/
/// - .guts/HEAD
/// - .guts/config
pub fn init(path: &Path) -> Result<()> {
    let guts_dir = path.join(".guts");
    let objects_dir = guts_dir.join("objects");
    let refs_heads_dir = guts_dir.join("refs").join("heads");
    let head_file = guts_dir.join("HEAD");
    let config_file = guts_dir.join("config");

    fs::create_dir_all(&objects_dir).with_context(|| "failed to create objects directory")?;
    fs::create_dir_all(&refs_heads_dir).with_context(|| "failed to create refs/heads directory")?;

    fs::write(&head_file, b"ref: refs/heads/master\n")
        .with_context(|| "failed to write HEAD file")?;

    fs::write(&config_file, b"[core]\n\trepositoryformatversion = 0\n")
        .with_context(|| "failed to write config file")?;

    Ok(())
}
