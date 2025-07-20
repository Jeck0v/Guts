// Module for a simple Git index in JSON format
// Educational alternative to Git's complex binary index

use crate::core::{blob, hash};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Simple structure for Git index
/// Stores only "staged" files with their SHA-1 hash
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SimpleIndex {
    /// Map: relative file path -> SHA-1 hash of content
    pub files: HashMap<String, String>,
}

impl SimpleIndex {
    /// Load index from .git/simple_index.json
    /// If file doesn't exist, return empty index
    pub fn load() -> Result<Self> {
        let index_path = get_simple_index_path()?;

        if !index_path.exists() {
            return Ok(SimpleIndex::default());
        }

        let content = fs::read_to_string(&index_path)
            .with_context(|| format!("unable to read {:?}", index_path))?;

        let index: SimpleIndex =
            serde_json::from_str(&content).with_context(|| "invalid JSON in index")?;

        Ok(index)
    }

    /// Save index to .git/simple_index.json
    pub fn save(&self) -> Result<()> {
        let index_path = get_simple_index_path()?;

        let content =
            serde_json::to_string_pretty(self).with_context(|| "unable to serialize index")?;

        fs::write(&index_path, content)
            .with_context(|| format!("unable to write {:?}", index_path))?;

        Ok(())
    }

    /// Add a file to the index (= "stage" it for next commit)
    pub fn add_file(&mut self, file_path: &Path) -> Result<()> {
        // Convert to absolute path if necessary
        let absolute_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(file_path)
        };

        // Read file content
        let content = fs::read(&absolute_path)
            .with_context(|| format!("unable to read {:?}", absolute_path))?;

        // Create Git blob and calculate its SHA-1 hash
        let blob = blob::Blob::new(content);
        let file_hash = hash::write_object(&blob)?;

        // Convert to relative path from repo root
        let relative_path = get_relative_path(&absolute_path)?;

        // Add to our map
        self.files.insert(relative_path, file_hash);

        Ok(())
    }

    /// Check if a file is in the index (staged)
    pub fn contains_file(&self, file_path: &str) -> bool {
        self.files.contains_key(file_path)
    }

    /// Return list of staged files
    pub fn get_staged_files(&self) -> Vec<&String> {
        self.files.keys().collect()
    }
}

/// Find Git repository root (directory containing .git/)
pub fn find_repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().with_context(|| "unable to get current directory")?;

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err(anyhow!("not a git repository")),
        }
    }
}

/// Return path to .git/simple_index.json
fn get_simple_index_path() -> Result<PathBuf> {
    let repo_root = find_repo_root()?;
    Ok(repo_root.join(".git").join("simple_index.json"))
}

/// Convert absolute path to relative path from repo root
fn get_relative_path(file_path: &Path) -> Result<String> {
    let repo_root = find_repo_root()?;
    let relative = file_path
        .strip_prefix(&repo_root)
        .with_context(|| "file is not in the repository")?;
    Ok(relative.to_string_lossy().to_string())
}

/// Check if we're in a Git repository
pub fn is_git_repository() -> Result<bool> {
    match find_repo_root() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Public function to add a file to the index
/// This is the function that the `guts add` command will call
pub fn add_file_to_index(file_path: &Path) -> Result<()> {
    let mut index = SimpleIndex::load()?;
    index.add_file(file_path)?;
    index.save()?;
    Ok(())
}
