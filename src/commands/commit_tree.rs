use std::env;
use clap::Args;
use std::path::PathBuf;
use anyhow::Result;
use crate::core::hash;
use crate::core::object::Commit;

#[derive(Args)]
pub struct CommitObject {
    pub tree: String,
    #[arg(short = 'p', long)]
    pub parent: Option<String>,
    #[arg(short = 'm', long)]
    pub message: String,
    #[arg(long)]
    pub git_dir: Option<PathBuf>,
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

pub fn run(args: &CommitObject) -> Result<String> {
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("could not get the current dir"));
    
    let git_dir = args.git_dir.clone().unwrap_or_else(|| {
        current_dir.join(".git")
    });

    if !git_dir.exists() {
        anyhow::bail!("No .git directory at {}", git_dir.display());
    }

    let commit = Commit {
        tree: args.tree.clone(),
        parent: args.parent.clone(),
        message: args.message.clone(),
    };

    let oid = hash::write_object(&commit)?;
    Ok(oid)
}
