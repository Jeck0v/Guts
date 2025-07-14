use std::fs;
use std::env;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use crate::core::cat;
use crate::core::cat::ParsedObject;

#[derive(Args)]
pub struct CatFileArgs {
    pub sha: String,
}

pub fn run(args: &CatFileArgs) -> Result<String> {
    let sha = &args.sha;

    if sha.len() < 4 {
        return Err(anyhow!("SHA is too small (need at least 4 characters)"));
    }

    let current_dir = env::current_dir().context("Failed to get the current directory")?;
    let guts_dir = current_dir.join(".guts");

    if !guts_dir.exists() {
        return Err(anyhow!("no guts directory found in current path"));
    }

    let object_path = cat::get_object_path(&guts_dir, sha);
    let content = fs::read(&object_path)
        .with_context(|| format!("Failed to read object file at {}", object_path.display()))?;

    let result = match cat::parse_object(&content)? {
        ParsedObject::Tree(entries) => {
            entries
                .iter()
                .map(|entry| {
                    let hash_hex: String = entry.hash.iter().map(|b| format!("{:02x}", b)).collect();
                    format!("{} {} {}", entry.mode, entry.name, hash_hex)
                })
                .collect::<Vec<String>>()
                .join("\n")
        }
        ParsedObject::Blob(data) => {
            String::from_utf8_lossy(&data).to_string()
        }
        ParsedObject::Commit(data) => {
            let mut out = String::new();
            out += &format!("tree: {}\n", data.tree);
            if let Some(parent) = &data.parent {
                out += &format!("parent: {}\n", parent);
            }
            out += &format!("message: {}", data.message);
            out
        }
        ParsedObject::Other(obj_type, _) => {
            format!("Unsupported object type: {}", obj_type)
        }
    };

    Ok(result)
}
