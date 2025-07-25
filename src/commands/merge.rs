use anyhow::{bail, Context, Result};
use clap::Args;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::checkout::{
    clean_working_directory, extract_tree_sha, parse_tree_object, read_and_parse_git_object,
};
use crate::core::parse_tree::{parse_tree, TreeEntry};
use crate::core::object::{Commit, Tree, TreeEntry as ObjectTreeEntry};
use crate::core::hash::write_object;

/// Command line arguments for the merge operation
#[derive(Args)]
pub struct MergeArgs {
    /// Name of the branch to merge into the current branch
    pub name: String,
    /// Optional directory path where the git repository is located
    #[arg(last = true)]
    pub dir: Option<PathBuf>,
}

/// Context structure that holds all the necessary information for a merge operation
/// This encapsulates the repository state and branch references
struct MergeContext {
    /// Path to the .git directory
    git_dir: PathBuf,
    /// Path to the current working directory
    current_dir: PathBuf,
    /// Current branch reference (e.g., "refs/heads/main")
    head_ref: String,
    /// SHA of the current commit (HEAD)
    current_commit: String,
    /// SHA of the commit from the branch being merged
    other_commit: String,
}

impl MergeContext {
    /// Creates a new MergeContext by reading the current repository state
    /// 
    /// # Arguments
    /// * `args` - Command line arguments containing branch name and optional directory
    /// 
    /// # Returns
    /// * `Result<Self>` - A new MergeContext or an error if the repository state is invalid
    fn new(args: &MergeArgs) -> Result<Self> {
        // Use provided directory or current working directory
        let current_dir = args.dir.clone().unwrap_or_else(|| std::env::current_dir().unwrap());
        let git_dir = current_dir.join(".git");

        // Read which branch HEAD points to
        let head_ref = Self::read_head_ref(&git_dir)?;
        // Get the commit SHAs for both branches
        let (current_commit, other_commit) = Self::read_commit_shas(&git_dir, &head_ref, &args.name)?;

        Ok(MergeContext {
            git_dir,
            current_dir,
            head_ref,
            current_commit,
            other_commit,
        })
    }

    /// Reads the HEAD reference to determine which branch is currently checked out
    /// 
    /// # Arguments  
    /// * `git_dir` - Path to the .git directory
    /// 
    /// # Returns
    /// * `Result<String>` - The branch reference (e.g., "refs/heads/main") or error if detached HEAD
    fn read_head_ref(git_dir: &Path) -> Result<String> {
        let head_path = git_dir.join("HEAD");
        let head_content = fs::read_to_string(&head_path)
            .with_context(|| format!("Failed to read HEAD from {}", head_path.display()))?;
        
        // HEAD should contain "ref: refs/heads/branch_name"
        head_content
            .strip_prefix("ref: ")
            .map(|s| s.trim().to_string())
            .ok_or_else(|| anyhow::anyhow!("Detached HEAD"))
    }

    /// Reads the commit SHAs for both the current branch and the branch to be merged
    /// 
    /// # Arguments
    /// * `git_dir` - Path to the .git directory
    /// * `head_ref` - Current branch reference
    /// * `branch_name` - Name of the branch to merge
    /// 
    /// # Returns
    /// * `Result<(String, String)>` - Tuple of (current_commit_sha, other_commit_sha)
    fn read_commit_shas(git_dir: &Path, head_ref: &str, branch_name: &str) -> Result<(String, String)> {
        let current_commit_path = git_dir.join(head_ref);
        let other_commit_path = git_dir.join("refs").join("heads").join(branch_name);

        // Verify both branch references exist
        if !current_commit_path.exists() {
            bail!("Current branch ref not found: {}", current_commit_path.display());
        }
        if !other_commit_path.exists() {
            bail!("Branch to merge not found: {}", other_commit_path.display());
        }

        // Read the commit SHAs from the reference files
        let current_commit = fs::read_to_string(&current_commit_path)?.trim().to_string();
        let other_commit = fs::read_to_string(&other_commit_path)?.trim().to_string();

        Ok((current_commit, other_commit))
    }
}

/// Structure to hold the three tree SHAs needed for a 3-way merge
/// In Git, a merge requires comparing:
/// - base_tree: The common ancestor (merge base)
/// - head_tree: The current branch's tree
/// - other_tree: The branch being merged's tree
struct MergeTrees {
    /// Tree SHA of the merge base commit
    base_tree: String,
    /// Tree SHA of the current branch (HEAD)
    head_tree: String,
    /// Tree SHA of the branch being merged
    other_tree: String,
}

impl MergeTrees {
    /// Loads the tree SHAs from the three commits involved in the merge
    /// 
    /// # Arguments
    /// * `git_dir` - Path to the .git directory
    /// * `merge_base` - SHA of the common ancestor commit
    /// * `current_commit` - SHA of the current branch's commit
    /// * `other_commit` - SHA of the branch being merged's commit
    /// 
    /// # Returns
    /// * `Result<Self>` - MergeTrees structure with all tree SHAs
    fn load_from_commits(git_dir: &Path, merge_base: &str, current_commit: &str, other_commit: &str) -> Result<Self> {
        // Extract tree SHA from each commit object
        let base_tree = extract_tree_sha(&read_commit_content_as_string(git_dir, merge_base)?)?;
        let head_tree = extract_tree_sha(&read_commit_content_as_string(git_dir, current_commit)?)?;
        let other_tree = extract_tree_sha(&read_commit_content_as_string(git_dir, other_commit)?)?;

        Ok(MergeTrees {
            base_tree,
            head_tree,
            other_tree,
        })
    }
}

/// Main entry point for the merge command
/// Orchestrates the entire merge process from finding the merge base to creating the merge commit
/// 
/// # Arguments
/// * `args` - Command line arguments containing branch name and optional directory
/// 
/// # Returns
/// * `Result<String>` - Empty string on success, or error if merge fails
pub fn run(args: &MergeArgs) -> Result<String> {
    // Initialize merge context with repository state
    let ctx = MergeContext::new(args)?;
    
    // Find the common ancestor (merge base) of the two branches
    let merge_base = find_merge_base(&ctx.git_dir, &ctx.current_commit, &ctx.other_commit)?
        .context("No common ancestor found")?;
    
    // Load the tree objects for the 3-way merge
    let trees = MergeTrees::load_from_commits(&ctx.git_dir, &merge_base, &ctx.current_commit, &ctx.other_commit)?;
    
    // Perform the actual merge of the trees
    let merged_tree_sha = merge_trees(&ctx.git_dir, &trees.base_tree, &trees.head_tree, &trees.other_tree)?;

    // Apply the merged tree to the working directory
    apply_merge_to_working_dir(&ctx, &merged_tree_sha)?;
    
    // Create the merge commit with two parents
    let new_commit_sha = create_merge_commit(&ctx, &merged_tree_sha, &args.name)?;
    
    // Update the current branch to point to the new merge commit
    update_head_ref(&ctx, &new_commit_sha)?;

    println!("Merged '{}' into '{}'. New commit: {}", args.name, ctx.head_ref, new_commit_sha);
    Ok(String::new())
}

/// Applies the merged tree to the working directory
/// This involves cleaning the current working directory and checking out the merged tree
/// 
/// # Arguments
/// * `ctx` - Merge context containing repository paths
/// * `merged_tree_sha` - SHA of the merged tree to apply
/// 
/// # Returns
/// * `Result<()>` - Success or error
fn apply_merge_to_working_dir(ctx: &MergeContext, merged_tree_sha: &str) -> Result<()> {
    // Clean the working directory and checkout the merged tree
    clean_working_directory(&ctx.current_dir, &ctx.git_dir, merged_tree_sha)?;
    let tree_content = read_and_parse_git_object(&ctx.git_dir, merged_tree_sha)?;
    parse_tree_object(&ctx.git_dir, &tree_content, ctx.current_dir.clone())?;
    Ok(())
}

/// Creates a merge commit with two parents
/// A merge commit is special because it has two parent commits instead of one
/// 
/// # Arguments
/// * `ctx` - Merge context containing commit SHAs and branch info
/// * `merged_tree_sha` - SHA of the merged tree
/// * `branch_name` - Name of the branch being merged (for commit message)
/// 
/// # Returns
/// * `Result<String>` - SHA of the newly created merge commit
fn create_merge_commit(ctx: &MergeContext, merged_tree_sha: &str, branch_name: &str) -> Result<String> {
    let now = chrono::Utc::now().timestamp();
    
    let commit = Commit {
        tree: merged_tree_sha.to_string(),
        // Two parents: current commit and the commit being merged
        parent: Some(vec![ctx.current_commit.clone(), ctx.other_commit.clone()]),
        author: "Your Name <you@example.com>".into(),
        committer: "Your Name <you@example.com>".into(),
        author_date: now,
        committer_date: now,
        message: format!("Merge branch '{}' into {}", branch_name, ctx.head_ref),
    };

    let new_commit_sha = write_object(&commit)?;
    Ok(new_commit_sha)
}

/// Updates the HEAD reference to point to the new merge commit
/// This effectively moves the current branch forward to include the merge
/// 
/// # Arguments
/// * `ctx` - Merge context containing the HEAD reference path
/// * `new_commit_sha` - SHA of the new merge commit
/// 
/// # Returns
/// * `Result<()>` - Success or error
fn update_head_ref(ctx: &MergeContext, new_commit_sha: &str) -> Result<()> {
    fs::write(ctx.git_dir.join(&ctx.head_ref), new_commit_sha)?;
    Ok(())
}

/// Enumeration of possible merge decisions for a file
/// This represents the outcome of comparing a file across the three trees
#[derive(Debug)]
enum MergeDecision {
    /// Take the version from the current branch (HEAD)
    TakeHead,
    /// Take the version from the branch being merged
    TakeOther,
    /// There's a conflict that requires manual resolution
    Conflict,
}

/// Decides what action to take for a file during merge based on 3-way comparison
/// This implements the core Git merge logic for individual files
/// 
/// # Arguments
/// * `base` - File entry from the merge base (common ancestor)
/// * `head` - File entry from the current branch (HEAD)
/// * `other` - File entry from the branch being merged
/// 
/// # Returns
/// * `MergeDecision` - The decision for how to handle this file
fn decide_merge_action(base: Option<&TreeEntry>, head: Option<&TreeEntry>, other: Option<&TreeEntry>) -> MergeDecision {
    match (base, head, other) {
        // Both branches have the same content - no conflict
        (Some(_), Some(h), Some(o)) if h.sha == o.sha => MergeDecision::TakeHead,
        // Current branch unchanged, other branch modified - take other
        (Some(b), Some(h), Some(o)) if b.sha == h.sha => MergeDecision::TakeOther,
        // Other branch unchanged, current branch modified - take head
        (Some(b), Some(h), Some(o)) if b.sha == o.sha => MergeDecision::TakeHead,
        // New file added in both branches with same content - no conflict
        (None, Some(h), Some(o)) if h.sha == o.sha => MergeDecision::TakeHead,
        // File only exists in current branch - keep it
        (_, Some(_), None) => MergeDecision::TakeHead,
        // File only exists in other branch - take it
        (_, None, Some(_)) => MergeDecision::TakeOther,
        // All other cases are conflicts (different changes to same file)
        _ => MergeDecision::Conflict,
    }
}

/// Converts a TreeEntry from the parse_tree module to an ObjectTreeEntry for the object module
/// This handles the conversion between different internal representations of tree entries
/// 
/// # Arguments
/// * `entry` - TreeEntry from parse_tree with string SHA
/// 
/// # Returns
/// * `Result<ObjectTreeEntry>` - ObjectTreeEntry with binary hash or conversion error
fn convert_to_object_tree_entry(entry: &TreeEntry) -> Result<ObjectTreeEntry> {
    // Validate SHA string length (should be 40 hex characters)
    let hex_str = if entry.sha.len() == 40 {
        &entry.sha
    } else {
        return Err(anyhow::anyhow!("Invalid SHA length: {}", entry.sha.len()));
    };
    
    // Convert hex string to bytes
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 20 {
        return Err(anyhow::anyhow!("SHA should be 20 bytes"));
    }
    
    // Convert to fixed-size array
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&bytes);
    
    Ok(ObjectTreeEntry {
        mode: entry.mode.clone(),
        name: entry.filename.clone(),
        hash,
    })
}

/// Performs a 3-way merge of Git trees
/// This is the core merge algorithm that combines changes from three tree states
/// 
/// # Arguments
/// * `git_dir` - Path to the .git directory
/// * `base` - SHA of the base tree (common ancestor)
/// * `head` - SHA of the current branch's tree
/// * `other` - SHA of the other branch's tree
/// 
/// # Returns
/// * `Result<String>` - SHA of the newly created merged tree
fn merge_trees(git_dir: &Path, base: &str, head: &str, other: &str) -> Result<String> {
    // Load all three trees into flat maps for easier comparison
    let base_entries = load_tree_map(git_dir, base)?;
    let head_entries = load_tree_map(git_dir, head)?;
    let other_entries = load_tree_map(git_dir, other)?;

    let mut merged_entries: Vec<ObjectTreeEntry> = Vec::new();

    // Collect all unique file paths from all three trees
    let all_paths: HashSet<PathBuf> = base_entries.keys()
        .chain(head_entries.keys())
        .chain(other_entries.keys())
        .cloned()
        .collect();

    // Process each file path
    for path in all_paths {
        let base_entry = base_entries.get(&path);
        let head_entry = head_entries.get(&path);
        let other_entry = other_entries.get(&path);

        // Decide what to do with this file based on 3-way comparison
        let decision = decide_merge_action(base_entry, head_entry, other_entry);
        
        match decision {
            MergeDecision::TakeHead => {
                if let Some(entry) = head_entry {
                    merged_entries.push(convert_to_object_tree_entry(entry)?);
                }
            },
            MergeDecision::TakeOther => {
                if let Some(entry) = other_entry {
                    merged_entries.push(convert_to_object_tree_entry(entry)?);
                }
            },
            MergeDecision::Conflict => {
                anyhow::bail!("Merge conflict on file: {:?}", path);
            },
        }
    }

    // Create and write the new merged tree object
    let tree_obj = Tree { entries: merged_entries };
    let tree_sha = write_object(&tree_obj)?;
    Ok(tree_sha)
}

/// Loads a Git tree into a flat HashMap mapping file paths to tree entries
/// This recursively traverses the tree structure and flattens it for easier processing
/// 
/// # Arguments
/// * `git_dir` - Path to the .git directory
/// * `sha` - SHA of the tree object to load
/// 
/// # Returns
/// * `Result<HashMap<PathBuf, TreeEntry>>` - Map of file paths to tree entries
fn load_tree_map(git_dir: &Path, sha: &str) -> Result<HashMap<PathBuf, TreeEntry>> {
    let mut map = HashMap::new();
    load_tree_map_recursive(git_dir, sha, PathBuf::new(), &mut map)?;
    Ok(map)
}

/// Recursively loads tree entries into a flat map
/// This handles the recursive nature of Git trees (directories contain subtrees)
/// 
/// # Arguments
/// * `git_dir` - Path to the .git directory
/// * `sha` - SHA of the current tree object
/// * `prefix` - Current path prefix for nested directories
/// * `map` - Mutable reference to the map being built
/// 
/// # Returns
/// * `Result<()>` - Success or error
fn load_tree_map_recursive(
    git_dir: &Path,
    sha: &str,
    prefix: PathBuf,
    map: &mut HashMap<PathBuf, TreeEntry>,
) -> Result<()> {
    let content = read_and_parse_git_object(git_dir, sha)?;
    for entry in parse_tree(&content)? {
        let full_path = prefix.join(&entry.filename);
        
        if entry.mode == "40000" {
            // Directory entry - recurse into subtree
            load_tree_map_recursive(git_dir, &entry.sha, full_path, map)?;
        } else {
            // File entry - add to map
            map.insert(full_path, entry.clone());
        }
    }
    Ok(())
}

/// Finds the merge base (common ancestor) of two commits using a breadth-first search
/// This implements a simplified version of Git's merge base algorithm
/// 
/// # Arguments
/// * `git_dir` - Path to the .git directory
/// * `a` - SHA of the first commit
/// * `b` - SHA of the second commit
/// 
/// # Returns
/// * `Result<Option<String>>` - SHA of the merge base commit, or None if no common ancestor
fn find_merge_base(git_dir: &Path, a: &str, b: &str) -> Result<Option<String>> {
    /// Helper function to get parent commits of a given commit
    fn get_parents(git_dir: &Path, commit: &str) -> Result<Vec<String>> {
        let content = read_and_parse_git_object(git_dir, commit)?;
        let content_str = std::str::from_utf8(&content)?;
        // Parse parent lines from commit object
        Ok(content_str
            .lines()
            .filter_map(|l| l.strip_prefix("parent "))
            .map(|s| s.to_string())
            .collect())
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    // Start BFS from both commits simultaneously
    queue.push_back(a.to_string());
    queue.push_back(b.to_string());

    while let Some(current) = queue.pop_front() {
        // If we've seen this commit before, it's a common ancestor
        if !visited.insert(current.clone()) {
            return Ok(Some(current));
        }
        // Add all parents to the queue for further exploration
        for parent in get_parents(git_dir, &current)? {
            queue.push_back(parent);
        }
    }
    Ok(None)
}

/// Reads a commit object and returns its content as a UTF-8 string
/// This is a utility function for parsing commit objects
/// 
/// # Arguments
/// * `git_dir` - Path to the .git directory
/// * `sha` - SHA of the commit object to read
/// 
/// # Returns
/// * `Result<String>` - Commit content as string or error
fn read_commit_content_as_string(git_dir: &Path, sha: &str) -> Result<String> {
    let content = read_and_parse_git_object(git_dir, sha)?;
    let content_str = std::str::from_utf8(&content)
        .context("Invalid UTF-8 in commit object content")?;
    Ok(content_str.to_string())
}