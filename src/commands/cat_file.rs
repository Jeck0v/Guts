use std::fs;
use std::{env, path::{Path,PathBuf}};

use clap::Args;

use anyhow::{anyhow, Context, Result};


#[derive(Args)]
pub struct CatFileArgs {
    /// Path to the file to read
    pub sha: String,
}

pub fn run(args: &CatFileArgs) -> Result<()> {
    let sha = &args.sha;
    
    if sha.len() < 4 {
        return Err(anyhow!("SHA is too small (need at least 4 caracters)"));
    }

    let current_dir = env::current_dir().context("failed to get the current directory")?;

    let guts_dir = current_dir.join(".guts");
    if !guts_dir.exists() {
        return  Err(anyhow!("no guts directory found in current path"));
    }

    let object_path = get_object_path(&guts_dir, sha);

    let content = fs::read(&object_path)
        .with_context(|| format!("failed to read object file at {}", object_path.display()))?;

    let (header, body) = parse_object(&content)?;

    println!("{header}\n{body}");

    Ok(())

}

fn get_object_path(guts_dir: &Path, sha: &str) -> PathBuf {
    let (dir, file) = sha.split_at(2);
    guts_dir.join("objects").join(dir).join(file)
}

fn parse_object(data: &[u8]) -> Result<(String, String)> {
    if let Some(null_pos) = data.iter().position(|&b| b == 0) {
        let header = String::from_utf8_lossy(&data[..null_pos]).to_string();
        let body = String::from_utf8_lossy(&data[null_pos + 1..]).to_string();
        Ok((header, body))
    } else {
        Err(anyhow!("invalid object format: no null byte found"))
    }
}
