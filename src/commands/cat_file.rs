use std::fs;
use std::env;

use anyhow::{anyhow, Context, Result};
use clap::Args;

use crate::core::cat;

use crate::core::cat::ParsedObject;

#[derive(Args)]
pub struct CatFileArgs {
    /// Partial or full SHA of the object to display
    pub sha: String,
}

/// Entry point for the `cat-file` command
///
/// - Validates the SHA length
/// - Finds the corresponding object file under `.guts/objects/`
/// - Reads the object's raw content
/// - Parses the object header and body
/// - Prints them to stdout
pub fn run(args: &CatFileArgs) -> Result<()> {
    let sha = &args.sha;

    if sha.len() < 4 {
        return Err(anyhow!("SHA is too small (need at least 4 characters)"));
    }

    let current_dir = env::current_dir().context("failed to get the current directory")?;

    let guts_dir = current_dir.join(".guts");
    if !guts_dir.exists() {
        return Err(anyhow!("no guts directory found in current path"));
    }

    let object_path = cat::get_object_path(&guts_dir, sha);

    let content = fs::read(&object_path)
        .with_context(|| format!("failed to read object file at {}", object_path.display()))?;

    match cat::parse_object(&content)? {
    ParsedObject::Tree(entries) => {
        for entry in entries {
            println!("{} {} {:x?}", entry.mode, entry.name, entry.hash.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
        }
    }
    ParsedObject::Blob(data) => {
        println!("{}", String::from_utf8_lossy(&data));
    }
    ParsedObject::Commit(data) => {
        println!("Commit :");
        println!("tree: {} ", data.tree);
        if let Some(parent) = &data.parent {
            println!("parent: {}", parent);
        }
        println!("message: {}", data.message);
    }
    ParsedObject::Other(obj_type, _) => {
        println!("Unsupported object type: {}", obj_type);
    }
}

    Ok(())
}