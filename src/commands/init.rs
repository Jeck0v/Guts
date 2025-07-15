use std::path::PathBuf;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use crate::core::repo;

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

    let guts_dir = dir.join(".guts");

    if guts_dir.exists() {
        return Err(anyhow!(".guts directory already exists in {:?}", dir));
    }

    repo::init(&dir).with_context(|| format!("failed to initialize repository in {:?}", dir))?;
    Ok(format!("Initialized empty Guts repository in {:?}", guts_dir))
}
