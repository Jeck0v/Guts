use std::{fs, path::PathBuf};
use std::env;

use anyhow::{anyhow, Context, Result};
use clap::Args;

use guts::core::cat;

#[derive(Args)]
pub struct CatFileArgs {
    /// SHA or partial SHA of the object to read
    pub sha: String,

    /// Path to the `.guts` directory (defaults to current directory + ".guts")
    #[arg(long, value_name = "DIR")]
    pub git_dir: Option<PathBuf>,
}

pub fn run(args: &CatFileArgs) -> Result<()> {
    let sha = &args.sha;

    if sha.len() < 4 {
        return Err(anyhow!("SHA is too small (need at least 4 characters)"));
    }

    // Determine the guts directory path
    let guts_dir = match &args.git_dir {
        Some(dir) => dir.clone(),
        None => {
            let current_dir = env::current_dir().context("failed to get current directory")?;
            current_dir.join(".guts")
        }
    };

    if !guts_dir.exists() {
        return Err(anyhow!("no .guts directory found at {}", guts_dir.display()));
    }

    // Get the path to the object file
    let object_path = cat::get_object_path(&guts_dir, sha);

    // Read the object file contents
    let content = fs::read(&object_path)
        .with_context(|| format!("failed to read object file at {}", object_path.display()))?;

    // Parse the Git object (header and body)
    let (header, body) = cat::parse_object(&content)?;

    // Print the header and the body
    println!("{header}\n{body}");

    Ok(())
}
