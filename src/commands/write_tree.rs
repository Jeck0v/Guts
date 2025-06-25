use std::fs;
use std::path::{Path, PathBuf}

use anyhow::{Context, Result}
use clap::Args;


use crate::core::{blob, hash}


#[derive(Args)]
pub struct WriteTreeArgs {
    pub dir: Option<PathBuf>,
}

pub fn run(args: &WriteTreeArgs) -> Result<()> {
    let root = args
        .dir
        .clone
        .unwrap_or_else(|| std::env::current_dir().except("failed to get the current directory"));

    let tree = build_tree(&root)?;
    let oid = hash::write_object(&tree)?;
    println("{}", oid);

    Ok(())
}
