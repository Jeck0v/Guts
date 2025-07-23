use crate::core::cat::{get_object_path, parse_object, ParsedObject};
use crate::core::simple_index;
use anyhow::{anyhow, Result};
use clap::Args;
use std::fs;
use std::path::PathBuf;

/// Arguments for the `guts log` command
#[derive(Args)]
pub struct LogArgs {
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

/// Entry point for the `guts log` command
/// Traverses the commit chain from HEAD to root, printing each commit's SHA and first line of message.
pub fn run(args: &LogArgs) -> Result<String> {
    // Determine current directory to use
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

    // Check if we're in a git repository
    if !simple_index::is_git_repository_from(args.dir.as_ref())? {
        return Err(anyhow!("fatal: not a git repository"));
    }

    // Use the standard .git directory
    let git_dir = current_dir.join(".git");

    // Read HEAD to get current commit
    let head_path = git_dir.join("HEAD");
    if !head_path.exists() {
        return Err(anyhow!("fatal: not a git repository (HEAD missing)"));
    }

    let head_content = fs::read_to_string(&head_path)?.trim().to_string();

    // Get the commit hash
    let commit_hash = if head_content.starts_with("ref: ") {
        // HEAD points to a branch
        let ref_path = head_content.strip_prefix("ref: ").unwrap();
        let ref_file = git_dir.join(ref_path);
        if !ref_file.exists() {
            return Err(anyhow!("fatal: branch exists but no commits yet"));
        }
        fs::read_to_string(ref_file)?.trim().to_string()
    } else {
        // Detached HEAD, direct commit hash
        head_content
    };

    // Traverse commit chain
    let mut output = String::new();
    let mut current_hash = commit_hash;
    loop {
        let commit_obj_path = get_object_path(&git_dir, &current_hash);
        if !commit_obj_path.exists() {
            return Err(anyhow!("fatal: commit object {} not found", current_hash));
        }

        let commit_data = fs::read(&commit_obj_path)?;
        let decompressed = decompress_object(&commit_data)?;
        let parsed = parse_object(&decompressed)?;

        let (parent, message) = match parsed {
            ParsedObject::Commit(ref commit) => (commit.parent.clone(), commit.message.clone()),
            _ => return Err(anyhow!("fatal: object {} is not a commit", current_hash)),
        };

        let first_line = message.lines().next().unwrap_or("");
        output.push_str(&format!("{} {}\n", current_hash, first_line));

        if let Some(parent_hash) = parent {
            current_hash = parent_hash;
        } else {
            break;
        }
    }

    Ok(output)
}


/// Decompress Git object data (Git uses zlib compression)
/// But our simple implementation stores objects uncompressed, so try both
fn decompress_object(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(_) => Ok(data.to_vec()), // If decompression fails, assume data is already uncompressed
    }
}
