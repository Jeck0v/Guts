use crate::core::repo;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitArgs {
    /// Directory to initialize the repository in (defaults to current directory)
    pub dir: Option<PathBuf>,
}

pub fn run(args: &InitArgs) -> Result<String> {
    let dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));

    let git_dir = dir.join(".git");

    if git_dir.exists() {
        return Err(anyhow!(".git directory already exists in {:?}", dir));
    }

    repo::init(&dir).with_context(|| format!("failed to initialize repository in {:?}", dir))?;
    Ok(format!(
        "Initialized empty Guts repository in {:?}",
        git_dir
    ))
}
