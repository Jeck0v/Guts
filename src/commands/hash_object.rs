use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use crate::core::{blob, hash};

#[derive(Args)]
pub struct HashObjectArgs {
    /// Path to the file to hash
    pub file: PathBuf,
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

pub fn run(args: &HashObjectArgs) -> Result<String> {
    let path = &args.file;

    if !path.exists() {
        return Err(anyhow!("file {:?} does not exist", path));
    }

    if path.is_dir() {
        return Err(anyhow!("path {:?} is a directory", path));
    }

    let data = std::fs::read(path)
        .with_context(|| format!("failed to read file {:?}", path))?;

    let blob = blob::Blob::new(data);
    let oid = hash::write_object(&blob)?;

    Ok(oid)
}
