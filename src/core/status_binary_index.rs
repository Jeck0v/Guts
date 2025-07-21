use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::hash;

/// Represents a file entry from the Git index.
pub struct IndexEntry {
    pub path: PathBuf,     // Relative file path
    pub blob_hash: String, // SHA-1 hash of the file content
}

/// Recursively lists all files in the working directory, excluding .git folders.
pub fn list_working_dir_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        // Skip .git directory
        !e.path().components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s == ".git"
        })
    });

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            entries.push(entry.into_path());
        }
    }

    Ok(entries)
}

/// Parses the .git/index file and returns the list of tracked file entries.
pub fn parse_git_index(index_path: &Path) -> Result<Vec<IndexEntry>> {
    let index_path = index_path.join("index");
    let data = fs::read(&index_path).context("failed to read index")?;

    if &data[0..4] != b"DIRC" {
        return Err(anyhow::anyhow!("Invalid index file (Missing DIRC)"));
    }

    let num_entries = u32::from_be_bytes(data[8..12].try_into().unwrap());
    let mut entries = Vec::new();
    let mut pos = 12;

    for _ in 0..num_entries {
        // Minimum size of a header entry (not including file name): 62 bytes
        if pos + 62 > data.len() {
            return Err(anyhow::anyhow!("Index file truncated"));
        }

        // SHA1 is at offset 40–60 (20 bytes)
        let sha_start = pos + 40;
        let sha_end = sha_start + 20;
        let sha_bytes = &data[sha_start..sha_end];
        let blob_hash = hex::encode(sha_bytes);

        // Flags are 2 bytes just after SHA1
        let flags_start = sha_end;
        let flags_end = flags_start + 2;
        let _flags = u16::from_be_bytes(data[flags_start..flags_end].try_into().unwrap());

        // Path starts after flags
        let mut path_end = flags_end;
        while path_end < data.len() && data[path_end] != 0 {
            path_end += 1;
        }

        if path_end >= data.len() {
            return Err(anyhow::anyhow!("Path name not null-terminated"));
        }

        let path = String::from_utf8_lossy(&data[flags_end..path_end]).to_string();
        entries.push(IndexEntry {
            path: PathBuf::from(path),
            blob_hash,
        });

        // Go to the next entry: include null byte and padding
        path_end += 1;
        let entry_len = path_end - pos;
        let padding = (8 - (entry_len % 8)) % 8;
        pos = path_end + padding;
    }

    Ok(entries)
}

/// Reads the current HEAD commit hash from .git/HEAD.
/// If HEAD is a symbolic reference (e.g. `ref: refs/heads/main`), it resolves the actual hash.
pub fn read_head_commit(gut_dir: &Path) -> Result<String> {
    let head_path = gut_dir.join("HEAD");
    let head_content = fs::read_to_string(&head_path).context("cannot read HEAD")?;

    if let Some(ref_line) = head_content.strip_prefix("ref: ") {
        // HEAD is a symbolic reference
        let ref_path = gut_dir.join(ref_line.trim());
        let sha = fs::read_to_string(ref_path)?.trim().to_string();
        Ok(sha)
    } else {
        // Detached HEAD
        Ok(head_content.trim().to_string())
    }
}

/// Checks if a single file has been modified compared to the Git index.
/// A file is considered modified if its content hash does not match the index hash,
/// or if the file is missing (deleted).
pub fn is_modified_single(entry: &IndexEntry, project_root: &Path) -> Result<bool> {
    let file_path = project_root.join(&entry.path);

    if !file_path.exists() {
        // File was deleted
        return Ok(true);
    }

    let content =
        fs::read(&file_path).with_context(|| format!("Failed to read file {:?}", file_path))?;

    let computed_hash = hash::hash_blob(&content).context("Failed to compute blob hash")?;

    Ok(computed_hash != entry.blob_hash)
}

/// Returns a list of files that were modified or deleted from the index,
/// by comparing the working directory with the Git index entries.
pub fn is_modified(index_entries: &[IndexEntry]) -> Result<Vec<PathBuf>> {
    let project_root = std::env::current_dir()
        .context("Cannot get current directory")?;
    let mut modified_files = Vec::new();

    for entry in index_entries {
        if is_modified_single(entry, &project_root)? {
            modified_files.push(entry.path.clone());
        }
    }

    Ok(modified_files)
}

pub fn get_staged_changes(
    index_entries: &[IndexEntry],
    head_entries: &[IndexEntry],
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut head_map = std::collections::HashMap::new();
    for entry in head_entries {
        head_map.insert(&entry.path, &entry.blob_hash);
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    let index_paths: HashSet<_> = index_entries.iter().map(|e| &e.path).collect();

    for entry in index_entries {
        match head_map.get(&entry.path) {
            None => added.push(entry.path.clone()),
            Some(blob_hash) if *blob_hash != &entry.blob_hash => modified.push(entry.path.clone()),
            _ => {}
        }
    }

    for path in head_map.keys() {
        if !index_paths.contains(path) {
            deleted.push((*path).clone());
        }
    }

    (added, modified, deleted)
}
