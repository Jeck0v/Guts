use std::fs;
use std::path::Path;

use anyhow::{Result, Context};

use crate::core::object::{Tree, TreeEntry};
use crate::core::{blob, hash};

/// Recursively builds a Git tree object from a directory on the filesystem.
/// 
/// # Arguments
/// * `dir` - Path to the directory to build the tree from.
///
/// # Returns
/// * `Result<Tree>` - A Git tree object representing the directory contents, or an error.
///
/// This function reads the directory entries, skips the `.guts` folder,
/// hashes all files as blobs, and collects their info as tree entries.
pub fn build_tree(dir: &Path) -> Result<Tree> {
    let mut entries = Vec::new(); // Container for the tree entries (files)

    // Iterate over directory entries, return error if directory can't be read
    for entry in fs::read_dir(dir)? {
        let entry = entry?;                    // Unwrap the DirEntry
        let path = entry.path();               // Get full path of the entry
        let name = entry.file_name()
            .into_string()
            .expect("File name is not valid UTF-8"); // Convert OsString to String

        if name == ".guts" {
            // Skip the internal .guts directory (where your git objects may be stored)
            continue;
        }

        if path.is_file() {
            // For files only (ignore directories for now)

            // Read the file content as bytes
            let data = fs::read(&path)
                .with_context(|| format!("failed to read file {:?}", path))?;

            // Create a Blob Git object from the file content
            let blob = blob::Blob::new(data);

            // Write the blob object and get its SHA1 hash in hex format
            let oid_hex = hash::write_object(&blob)?;

            // Decode the hex SHA1 hash into raw bytes (20 bytes for SHA1)
            let hash_bin = hex::decode(oid_hex)
                .expect("valid SHA1 hex");

            // Create fixed-size array to store the 20-byte hash
            let mut hash = [0u8; 20];
            hash.copy_from_slice(&hash_bin);

            // Create a tree entry for this file
            entries.push(TreeEntry {
                mode: "100644".to_string(), // File mode for a normal file
                name,
                hash,
            });
        }
    }

    // Return a Tree Git object containing all collected entries
    Ok(Tree { entries })
}
