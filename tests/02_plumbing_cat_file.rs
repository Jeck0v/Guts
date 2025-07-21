use std::env;
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Args;

use guts::core::cat;

#[derive(Args)]
pub struct CatFileArgs {
    /// SHA or partial SHA of the object to read
    pub sha: String,

    /// Path to the `.git` directory (defaults to current directory + ".git")
    #[arg(long, value_name = "DIR")]
    pub git_dir: Option<PathBuf>,
}

pub fn run(args: &CatFileArgs) -> Result<()> {
    let sha = &args.sha;

    if sha.len() < 4 {
        return Err(anyhow!("SHA is too small (need at least 4 characters)"));
    }

    // Determine the git directory path
    let git_dir = match &args.git_dir {
        Some(dir) => dir.clone(),
        None => {
            let current_dir = env::current_dir().context("failed to get current directory")?;
            current_dir.join(".git")
        }
    };

    if !git_dir.exists() {
        return Err(anyhow!("no .git directory found at {}", git_dir.display()));
    }

    // Get the path to the object file
    let object_path = cat::get_object_path(&git_dir, sha);

    // Read the object file contents
    let content = fs::read(&object_path)
        .with_context(|| format!("failed to read object file at {}", object_path.display()))?;

    // Parse the Git object
    let parsed_obj = cat::parse_object(&content)?;

    // Print the parsed content based on its type
    match parsed_obj {
        cat::ParsedObject::Blob(data) => {
            // Print blob content as UTF-8 string if possible, else bytes debug
            match std::str::from_utf8(&data) {
                Ok(text) => println!("{}", text),
                Err(_) => println!("{:?}", data),
            }
        }
        cat::ParsedObject::Tree(entries) => {
            for entry in entries {
                println!("{} {} {}", entry.mode, entry.name, hex::encode(entry.hash));
            }
        }
        cat::ParsedObject::Commit(data) => {
            println!("Commit :");
            println!("tree: {}", data.tree);
            if let Some(parent) = &data.parent {
                println!("parent: {}", parent);
            }
            println!("message: {}", data.message);
        }

        cat::ParsedObject::Other(obj_type, data) => {
            println!("Unknown object type: {}", obj_type);
            println!("{:?}", data);
        }
    }

    Ok(())
}
