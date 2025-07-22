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
/// Handles subdirectories by creating recursive tree structure
fn build_tree_from_index(index: &simple_index::SimpleIndex) -> Result<Tree> {
    // Build the root tree with all files from index
    build_tree_recursive(&index.files, "")
}

/// Recursively build a tree for a given directory path
/// 
/// Simple algorithm:
/// 1. Filter files that belong to current directory level
/// 2. For direct files: create blob entries  
/// 3. For subdirectories: collect files, recurse, create tree entries
fn build_tree_recursive(
    all_files: &std::collections::HashMap<String, String>, 
    prefix: &str
) -> Result<Tree> {
    use std::collections::HashMap;
    
    let mut entries = Vec::new();
    let mut subdirs: HashMap<String, Vec<(String, String)>> = HashMap::new();
    
    // Process each file to see if it belongs in this directory level
    for (file_path, file_hash) in all_files {
        // Skip files not in our prefix
        let relative_path = if prefix.is_empty() {
            file_path.as_str()
        } else if file_path.starts_with(prefix) && file_path.len() > prefix.len() && file_path.chars().nth(prefix.len()) == Some('/') {
            &file_path[prefix.len() + 1..] // +1 to skip the '/'
        } else {
            continue; // Not in this directory
        };
        
        if let Some(slash_pos) = relative_path.find('/') {
            // File is in a subdirectory
            let subdir_name = &relative_path[..slash_pos];
            subdirs.entry(subdir_name.to_string())
                   .or_default()
                   .push((file_path.clone(), file_hash.clone()));
        } else {
            // File is directly in this directory
            let hash_bin = hex::decode(file_hash)
                .map_err(|_| anyhow::anyhow!("invalid SHA-1 hash: {}", file_hash))?;
            let mut hash = [0u8; 20];
            hash.copy_from_slice(&hash_bin);
            
            entries.push(TreeEntry {
                mode: "100644".to_string(),
                name: relative_path.to_string(),
                hash,
            });
        }
    }
    
    // Create subtrees for each subdirectory
    for (subdir_name, _) in subdirs {
        let subdir_prefix = if prefix.is_empty() {
            subdir_name.clone()
        } else {
            format!("{}/{}", prefix, subdir_name)
        };
        
        let subtree = build_tree_recursive(all_files, &subdir_prefix)?;
        let subtree_hash = hash::write_object(&subtree)?;
        let hash_bin = hex::decode(&subtree_hash)?;
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&hash_bin);
        
        entries.push(TreeEntry {
            mode: "40000".to_string(), // Directory mode (Git uses 40000, not 040000)
            name: subdir_name,
            hash,
        });
    }
    
    // Sort entries by name (required by Git)
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(Tree { entries })
}
