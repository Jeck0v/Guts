use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

use crate::core::object::GitObject;

pub fn write_object(obj: &impl GitObject) -> Result<String> {
    // 1. Serialize the object (with header + content)
    let serialized = obj.serialize();

    // 2. Hash it using SHA-1
    let mut hasher = Sha1::new();
    hasher.update(&serialized);
    let hash = hasher.finalize();
    let hex = hex::encode(&hash);

    // 3. Prepare storage path .git/objects/xx/yyyy...
    let (dir_name, file_name) = hex.split_at(2);
    let path = PathBuf::from(".git/objects").join(dir_name).join(file_name);

    if path.exists() {
        return Ok(hex); // Object already exists
    }

    // 4. Compress the serialized content using zlib
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    std::io::copy(&mut &serialized[..], &mut encoder)?;
    let compressed = encoder.finish()?;

    // 5. Write to disk
    let parent_dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid object path: no parent directory"))?;
    fs::create_dir_all(parent_dir)
        .with_context(|| "failed to create object directory")?;
    fs::write(&path, compressed)
        .with_context(|| format!("failed to write object to {:?}", path))?;

    Ok(hex)
}

/// Computes the SHA-1 hash of a blob with Git-style header.
/// This is used to compare working directory files to their index versions.
pub fn hash_blob(data: &[u8]) -> Result<String> {
    let header = format!("blob {}\0", data.len());
    let mut hasher = Sha1::new();

    hasher.update(header.as_bytes());
    hasher.update(data);

    let result = hasher.finalize();
    Ok(hex::encode(result))
}
