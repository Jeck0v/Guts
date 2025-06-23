use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Args;

use crate::core::{blob, hash};

/// Arguments for the `hash-object` command.
/// Expects a single file path.
#[derive(Args)]
pub struct HashObjectArgs {
    /// Path to the file to hash
    pub file: PathBuf,
}

/// Runs the `hash-object` command:
/// - Reads the file at the given path
/// - Wraps its content in a Git-like blob object
/// - Computes its SHA-1 hash
/// - Stores the blob in `.guts/objects/` if not already present
/// - Prints the object ID (SHA-1 hash) to stdout
pub fn run(args: &HashObjectArgs) -> Result<()> {
    let path = &args.file;

    // Check that the path exists
    if !path.exists() {
        return Err(anyhow!("file {:?} does not exist", path));
    }

    // Reject directories — only files are valid
    if path.is_dir() {
        return Err(anyhow!("path {:?} is a directory", path));
    }

    let data = std::fs::read(path)
        .with_context(|| format!("failed to read file {:?}", path))?;

    let blob = blob::Blob::new(data);

    // Compute and write the object to `.guts/objects/` if needed
    let oid = hash::write_object(&blob)?;

    // Output the hash to stdout
    println!("{}", oid);

    Ok(())
}
