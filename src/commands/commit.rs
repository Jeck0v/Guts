use crate::commands::{commit_tree, write_tree};
use crate::core::simple_index;
use anyhow::Result;
use clap::Args;
use std::env;
use std::path::PathBuf;

#[derive(Args)]
pub struct CommitArgs {
    /// Commit message
    #[arg(short = 'm', long)]
    pub message: String,
    
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

pub fn run(args: &CommitArgs) -> Result<String> {
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("could not get the current dir"));

    // Change to the specified directory for this operation
    let original_dir = env::current_dir()?;
    env::set_current_dir(&current_dir)?;

    let result = run_commit(args);

    // Restore original directory
    env::set_current_dir(original_dir)?;

    result
}

fn run_commit(args: &CommitArgs) -> Result<String> {
    // Check if we're in a git repository
    if !simple_index::is_git_repository()? {
        return Err(anyhow::anyhow!("fatal: not a git repository"));
    }

    // Load the index to check if there are staged files
    let index = simple_index::SimpleIndex::load()?;
    if index.files.is_empty() {
        return Err(anyhow::anyhow!("nothing to commit, working tree clean"));
    }

    // 1. Create tree from staged files using write-tree
    let write_tree_args = write_tree::WriteTreeArgs { dir: None };
    let tree_hash = write_tree::run(&write_tree_args)?;

    // 2. Get the current HEAD commit (parent) if it exists
    let parent = match get_current_head()? {
        Some(p) => Some(vec![p]),
        None => None,
    };


    // 3. Create commit object using commit-tree
    let commit_tree_args = commit_tree::CommitObject {
        tree: tree_hash.clone(),
        parent: parent,
        message: args.message.clone(),
        author: "guts <guts@example.com>".to_string(),
        committer: "guts <guts@example.com>".to_string(),
        author_date: None,
        committer_date: None,
        dir: None,
    };
    let commit_hash = commit_tree::run(&commit_tree_args)?;

    // 4. Update HEAD to point to the new commit
    update_head(&commit_hash)?;

    // 5. Clear the index (staged files become committed)
    clear_index()?;

    Ok(format!("[{}] {}", &commit_hash[..7], args.message))
}

/// Get the current HEAD commit hash, or None if this is the first commit
fn get_current_head() -> Result<Option<String>> {
    let head_path = std::path::Path::new(".git/HEAD");
    
    if !head_path.exists() {
        return Ok(None);
    }

    let head_content = std::fs::read_to_string(head_path)?;
    let head_content = head_content.trim();

    // Check if HEAD points to a branch (ref: refs/heads/main)
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.strip_prefix("ref: ")
            .ok_or_else(|| anyhow::anyhow!("malformed HEAD reference: {}", head_content))?;
        let ref_file = std::path::Path::new(".git").join(ref_path);
        
        if ref_file.exists() {
            let commit_hash = std::fs::read_to_string(ref_file)?;
            Ok(Some(commit_hash.trim().to_string()))
        } else {
            // Branch exists but no commits yet
            Ok(None)
        }
    } else {
        // HEAD points directly to a commit (detached HEAD)
        Ok(Some(head_content.to_string()))
    }
}

/// Update HEAD to point to the new commit
fn update_head(commit_hash: &str) -> Result<()> {
    let head_path = std::path::Path::new(".git/HEAD");
    let head_content = std::fs::read_to_string(head_path)?;
    let head_content = head_content.trim();

    if head_content.starts_with("ref: ") {
        // HEAD points to a branch, update the branch ref
        let ref_path = head_content.strip_prefix("ref: ")
            .ok_or_else(|| anyhow::anyhow!("malformed HEAD reference: {}", head_content))?;
        let ref_file = std::path::Path::new(".git").join(ref_path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = ref_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(ref_file, format!("{}\n", commit_hash))?;
    } else {
        // Detached HEAD, update HEAD directly
        std::fs::write(head_path, format!("{}\n", commit_hash))?;
    }

    Ok(())
}

/// Clear the staging area after successful commit
fn clear_index() -> Result<()> {
    let mut index = simple_index::SimpleIndex::load()?;
    index.files.clear();
    index.save()?;
    Ok(())
}