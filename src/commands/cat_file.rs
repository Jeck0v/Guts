use crate::core::cat;
use crate::core::cat::ParsedObject;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::env;
use std::fs;

#[derive(Args)]
pub struct CatFileArgs {
    pub sha: String,
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<std::path::PathBuf>,
}

pub fn run(args: &CatFileArgs) -> Result<String> {
    let sha = &args.sha;

    if sha.len() < 4 {
        return Err(anyhow!("SHA is too small (need at least 4 characters)"));
    }

    // Determine current directory to use
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));
    let git_dir = current_dir.join(".git");

    if !git_dir.exists() {
        return Err(anyhow!("no git directory found in current path"));
    }

    let object_path = cat::get_object_path(&git_dir, sha);
    let content = fs::read(&object_path)
        .with_context(|| format!("Failed to read object file at {}", object_path.display()))?;

    let decompressed = decompress_object(&content)?;
    let result = match cat::parse_object(&decompressed)? {
        ParsedObject::Tree(entries) => entries
            .iter()
            .map(|entry| {
                let hash_hex: String = entry.hash.iter().map(|b| format!("{:02x}", b)).collect();
                format!("{} {} {}", entry.mode, entry.name, hash_hex)
            })
            .collect::<Vec<String>>()
            .join("\n"),
        ParsedObject::Blob(data) => String::from_utf8_lossy(&data).to_string(),
        ParsedObject::Commit(data) => {
            let mut out = String::new();
            out += &format!("tree {}\n", data.tree);
            if let Some(parent) = &data.parent {
                out += &format!("parent {}\n", parent);
            }
            out += &format!("author {} {} +0000\n", data.author, data.author_date);
            out += &format!("committer {} {} +0000\n", data.committer, data.committer_date);
            out += "\n";
            out += &data.message;
            if !data.message.ends_with('\n') {
                out += "\n";
            }
            out
        }
        ParsedObject::Other(obj_type, _) => {
            format!("Unsupported object type: {}", obj_type)
        }
    };

    Ok(result)
}

fn decompress_object(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Ok(decompressed),
        Err(_) => Ok(data.to_vec()), // If decompression fails, assume data is already uncompressed
    }
}
