use crate::core::{build_tree, hash};
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct WriteTreeArgs {
    pub dir: Option<PathBuf>,
    /// Current directory for the operation (injected by TUI)
    pub tui_dir: Option<PathBuf>,
}

pub fn run(args: &WriteTreeArgs) -> Result<String> {
    let root = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get the current directory"));

    let tree = build_tree::build_tree(&root)?;
    let oid = hash::write_object(&tree)?;

    Ok(oid)
}
