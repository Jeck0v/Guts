use crate::core::object::{Tree, TreeEntry};
use crate::core::{hash, simple_index};
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct WriteTreeArgs {
    pub dir: Option<PathBuf>,
}

/// New version of write-tree that uses the simple JSON index
/// Instead of reading the filesystem, reads the index to create the tree
pub fn run(_args: &WriteTreeArgs) -> Result<String> {
    // Check if we're in a git repository
    if !simple_index::is_git_repository()? {
        return Err(anyhow::anyhow!("fatal: not a git repository"));
    }

    // Load the JSON index
    let index = simple_index::SimpleIndex::load()?;

    // Create the tree from the index (not the filesystem)
    let tree = build_tree_from_index(&index)?;

    // Write the tree object and return its hash
    let oid = hash::write_object(&tree)?;

    Ok(oid)
}

/// Build a Git tree object from the JSON index
/// Simpler and more correct than scanning the filesystem
fn build_tree_from_index(index: &simple_index::SimpleIndex) -> Result<Tree> {
    let mut entries = Vec::new();

    // For each file in the index
    for (file_path, file_hash) in &index.files {
        // Decode the SHA-1 hex hash to bytes
        let hash_bin = hex::decode(file_hash)
            .map_err(|_| anyhow::anyhow!("invalid SHA-1 hash: {}", file_hash))?;

        // Create the 20-byte hash array
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&hash_bin);

        // Extract just the file name (not the full path)
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid file path: {}", file_path))?
            .to_string_lossy()
            .to_string();

        // Create the tree entry
        entries.push(TreeEntry {
            mode: "100644".to_string(), // Mode for normal file
            name: file_name,
            hash,
        });
    }

    // Sort entries by name (required by Git)
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Tree { entries })
}
