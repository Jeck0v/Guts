use anyhow::{Context, Result};
use std::fs;
use crate::internals::hash;

pub fn hash_blob_from_path(path: &str) -> Result<String> {
    let content = fs::read(path)
        .with_context(|| format!("Failed to read file '{}'", path))?;

    if content.is_empty() {
        anyhow::bail!("File '{}' is empty", path);
    }
    let blob_data = hash::prepare_blob(&content);
    let sha = hash::sha1_hex(&blob_data);
    Ok(sha)
}
