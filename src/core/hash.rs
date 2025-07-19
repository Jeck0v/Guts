use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use sha1::{Digest, Sha1};

use crate::core::object::GitObject;

/// Calculates the SHA-1 and writes the object to .git/objects/<prefix>/<hash>
pub fn write_object(obj: &impl GitObject) -> Result<String> {
    let serialized = obj.serialize();

    let mut hasher = Sha1::new();
    hasher.update(&serialized);
    let hash = hasher.finalize();
    let hex = hex::encode(&hash);

    let (dir_name, file_name) = hex.split_at(2);
    let path = PathBuf::from(".git/objects").join(dir_name).join(file_name);

    if path.exists() {
        return Ok(hex);
    }

    fs::create_dir_all(path.parent().unwrap())
        .with_context(|| "failed to create object directory")?;
    fs::write(&path, serialized)
        .with_context(|| format!("failed to write object to {:?}", path))?;

    Ok(hex)
}

pub fn hash_blob(data: &[u8]) -> Result<String> {
    let header = format!("blob {}\0", data.len());
    let mut hasher = Sha1::new();

    hasher.update(header.as_bytes());
    hasher.update(data);

    let result = hasher.finalize();

    Ok(hex::encode(result))
}