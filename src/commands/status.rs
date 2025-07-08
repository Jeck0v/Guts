use std::fs;
use std::path::{PathBuf};
use anyhow::{Context,Result};
use clap::Args;


#[derive(Args)]
pub struct StatusObject {
    pub guts_dir : Option<PathBuf>
}

pub fn run(args: &StatusObject) -> Result<()> {
    let guts_dir = args.guts_dir.clone().unwrap_or_else(|| std::env::current_dir().expect("failed to get the current directory").join(".guts"));
    if !guts_dir.exists() {
        return Err(anyhow::anyhow!("Path {:?} does not exist", guts_dir.display()));
    }

    let head_path = guts_dir.join("HEAD");
    if !head_path.exists() {
        return Err(anyhow::anyhow!("File {:?} does not exist", head_path.display()));
    }

    let head_content = fs::read_to_string(&head_path).with_context(|| format!("Impossible to read {}", head_path.display()))?;

    let reference = head_content.strip_prefix("ref: ").ok_or_else(|| anyhow::anyhow!("HEAD doesnt contains a valid ref"))?.trim();

    let ref_path: PathBuf = guts_dir.join(reference);
    
    if !ref_path.exists() {
        println!("No commit done on {}", reference);
    } else {
        let commit_sha = fs::read_to_string(&ref_path).with_context(|| format!("Impossible to read {}", ref_path.display()))?;

        if commit_sha.is_empty() {
            println!("No commit done on {}", reference);
        } else {
            println!("Sha of commit : {}", commit_sha);
        }
    }


    


    Ok(())
}