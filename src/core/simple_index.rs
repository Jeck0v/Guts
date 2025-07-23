// Module for a simple Git index in JSON format
// Educational alternative to Git's complex binary index

use crate::core::{blob, cat, hash};
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

/// Get the files committed in the current HEAD
/// Returns a HashMap: relative file path -> SHA-1 hash
pub fn get_committed_files() -> Result<HashMap<String, String>> {
    let repo_root = find_repo_root()?;
    let git_dir = repo_root.join(".git");
    
    // Read HEAD to get current commit
    let head_path = git_dir.join("HEAD");
    if !head_path.exists() {
        // No commits yet
        return Ok(HashMap::new());
    }
    
    let head_content = fs::read_to_string(&head_path)?;
    let head_content = head_content.trim();
    
    // Get the commit hash
    let commit_hash = if head_content.starts_with("ref: ") {
        // HEAD points to a branch
        let ref_path = head_content.strip_prefix("ref: ").unwrap();
        let ref_file = git_dir.join(ref_path);
        
        if !ref_file.exists() {
            // Branch exists but no commits yet
            return Ok(HashMap::new());
        }
        
        fs::read_to_string(ref_file)?.trim().to_string()
    } else {
        // Detached HEAD, direct commit hash
        head_content.to_string()
    };
    
    // Read the commit object to get the tree hash
    let commit_obj_path = cat::get_object_path(&git_dir, &commit_hash);
    if !commit_obj_path.exists() {
        return Ok(HashMap::new());
    }
    
    let commit_data = fs::read(&commit_obj_path)?;
    let decompressed = decompress_object(&commit_data)?;
    let parsed = cat::parse_object(&decompressed)?;
    
    let tree_hash = match parsed {
        cat::ParsedObject::Commit(commit) => commit.tree,
        _ => return Err(anyhow!("HEAD does not point to a commit object")),
    };
    
    // Read the tree object to get the files
    get_files_from_tree(&git_dir, &tree_hash, "")
}

/// Recursively get all files from a tree object
/// Returns a HashMap: relative file path -> SHA-1 hash
fn get_files_from_tree(git_dir: &Path, tree_hash: &str, prefix: &str) -> Result<HashMap<String, String>> {
    let mut files = HashMap::new();
    
    let tree_obj_path = cat::get_object_path(git_dir, tree_hash);
    if !tree_obj_path.exists() {
        return Ok(files);
    }
    
    let tree_data = fs::read(&tree_obj_path)?;
    let decompressed = decompress_object(&tree_data)?;
    let parsed = cat::parse_object(&decompressed)?;
    
    let entries = match parsed {
        cat::ParsedObject::Tree(entries) => entries,
        _ => return Err(anyhow!("Object is not a tree")),
    };
    
    for entry in entries {
        let file_path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };
        
        if entry.mode == "100644" {
            // Regular file
            let hash_hex = hex::encode(entry.hash);
            files.insert(file_path, hash_hex);
        } else if entry.mode == "40000" {
            // Directory - recursively get files from subtree
            let subtree_hash = hex::encode(entry.hash);
            let subfiles = get_files_from_tree(git_dir, &subtree_hash, &file_path)?;
            files.extend(subfiles);
        }
    }
    
    Ok(files)
}

/// Decompress Git object data (Git uses zlib compression)
/// But our simple implementation stores objects uncompressed, so try both
fn decompress_object(data: &[u8]) -> Result<Vec<u8>> {
    // First try to decompress as zlib (standard Git format)
    use std::io::Read;
    
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(_) => {
            // If decompression fails, assume data is already uncompressed
            // (our simple implementation stores objects uncompressed)
            Ok(data.to_vec())
        }
    }
}

/// Find Git repository root from a specific directory
pub fn find_repo_root_from(start_dir: Option<&PathBuf>) -> Result<PathBuf> {
    let mut current = match start_dir {
        Some(dir) => dir.clone(),
        None => std::env::current_dir().with_context(|| "unable to get current directory")?,
    };

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

/// Check if we're in a Git repository from a specific directory
pub fn is_git_repository_from(start_dir: Option<&PathBuf>) -> Result<bool> {
    match find_repo_root_from(start_dir) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Add a file to the index from a specific directory context
pub fn add_file_to_index_from(file_path: &Path, start_dir: Option<&PathBuf>) -> Result<()> {
    // Set current directory context if provided
    let original_dir = std::env::current_dir()?;
    
    if let Some(dir) = start_dir {
        std::env::set_current_dir(dir)?;
    }
    
    // Use existing add_file_to_index function
    let result = add_file_to_index(file_path);
    
    // Restore original directory
    std::env::set_current_dir(&original_dir)?;
    
    result
}
