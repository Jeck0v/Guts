use anyhow::{Context, Result};
use clap::Args;
use crate::core::resolve_parse::resolve_ref;
use flate2::read::ZlibDecoder;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct CheckoutObject {
    pub name: Option<String>,

    #[arg(short = 'b', long)]
    pub branch_name: Option<String>
}

pub fn run(args: &CheckoutObject) -> Result<String> {
    let current_dir = std::env::current_dir().context("Cannot get the current directory")?;
    let git_dir = current_dir.join(".git");

    let fallback = read_head_ref(&git_dir)?
        .unwrap_or("HEAD (detached)".to_string());

    let target_ref = args.name.as_deref().unwrap_or(fallback.as_str());

    println!("{:?}", target_ref);

    let sha = resolve_ref(&git_dir, target_ref)?;
    let path_obj = git_dir.join("objects").join(&sha[..2]).join(&sha[2..]);

    if !path_obj.exists() {
        anyhow::bail!("Git object file not found at {:?}", path_obj);
    }
    

    let decompressed_bytes = read_git_object(&path_obj)?;
    
    let (_header, commit_content) = split_header_and_content(&decompressed_bytes)?;
    let commit_str = std::str::from_utf8(commit_content)
    .context("Commit content is not valid UTF-8")?;
    let tree_sha = extract_tree_sha(commit_str)?;
    

    if has_uncommitted_changes(&git_dir, &current_dir, &tree_sha)? {
        anyhow::bail!("You have uncommitted changes. Commit or stash them before switching branches.");
    } else {
        if let Some(branch_name) = &args.branch_name {
            let refs_path = git_dir.join("refs").join("heads").join(branch_name);
            if refs_path.exists() {
                anyhow::bail!("Branch '{}' already exists", branch_name);
            }
            std::fs::write(&refs_path, format!("{}\n", sha))
                .with_context(|| format!("Failed to create a branch at {:?}", refs_path))?;

            let head_path = git_dir.join("HEAD");
            std::fs::write(&head_path, format!("ref: refs/heads/{}\n", branch_name))
                .with_context(|| format!("failed to update HEAD to point to {}", branch_name))?;

        } else {
            let possible_branch_path = git_dir.join("refs").join("heads").join(target_ref);
            if possible_branch_path.exists() {
                let head_path = git_dir.join("HEAD");
                std::fs::write(&head_path, format!("ref: refs/heads/{}\n", target_ref))
                    .with_context(|| format!("failed to update HEAD to point to {}", target_ref))?;
            }
        }

        
    


    
            clean_working_directory(&current_dir, &git_dir)?;
        
            println!("Tree SHA: {}", tree_sha);
        
            let tree_path = git_dir.join("objects").join(&tree_sha[..2]).join(&tree_sha[2..]);
            let tree_bytes = read_git_object(&tree_path)?;
            let (_header, tree_content) = split_header_and_content(&tree_bytes)?;
            parse_tree_object(git_dir, tree_content, current_dir)?;
        
            Ok(tree_sha)
    }
}


fn extract_tree_sha(commit_text: &str) -> Result<String> {
    for line in commit_text.lines() {
        if let Some(rest) = line.strip_prefix("tree ") {
            return Ok(rest.trim().to_string());
        }
    }
    anyhow::bail!("Tree SHA not found in commit object");
}

fn split_header_and_content(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    if let Some(null_index) = bytes.iter().position(|&b| b == 0) {
        let (header, content) = bytes.split_at(null_index + 1);
        Ok((header, content))
    } else {
        anyhow::bail!("No null separator found in Git object");
    }
}

fn read_git_object(path: &Path) -> Result<Vec<u8>> {
    let file = File::open(path).context("Failed to open object file")?;
    let mut decoder = ZlibDecoder::new(file);

    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

fn parse_tree_object(git_dir: PathBuf,tree_bytes: &[u8], target_dir: PathBuf) -> Result<()> {

    let git_dir = &git_dir;

    println!("{:?}", &tree_bytes);

    let mut cursor = Cursor::new(tree_bytes);

    while (cursor.position() as usize) < tree_bytes.len() {
        let mut mode = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            cursor.read_exact(&mut byte)?;
            if byte[0] == b' ' {
                break;
            }
            mode.push(byte[0]);
        }

        let mut filename = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            cursor.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            filename.push(byte[0]);
        }

        let mut sha_bytes = [0u8; 20];
        cursor.read_exact(&mut sha_bytes)?;

        let mode_str = String::from_utf8_lossy(&mode);
        let filename_str = String::from_utf8_lossy(&filename);
        let sha_hex = sha_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        let full_path = target_dir.join(&*filename_str);

        println!("{} {} {}", mode_str, sha_hex, filename_str);

        if mode_str == "40000" {
            fs::create_dir_all(&full_path);
            let tree_path = git_dir.join("objects").join(&sha_hex[..2]).join(&sha_hex[2..]);
            let tree_bytes  = read_git_object(&tree_path)?;
            let (_header, sub_tree_content) = split_header_and_content(&tree_bytes)?;
            parse_tree_object(git_dir.to_path_buf(), sub_tree_content, full_path)?;

        } else {
            let blob_path = git_dir.join("objects").join(&sha_hex[..2]).join(&sha_hex[2..]);
            let blob_bytes = read_git_object(&blob_path)?;
            let (_header, blob_content) = split_header_and_content(&blob_bytes)?;
            fs::create_dir_all(&full_path.parent().unwrap());
            let mut file = File::create(&full_path)?;
            file.write_all(blob_content)?;
        }
    }

    Ok(())
}


fn read_head_ref(git_dir: &Path) -> Result<Option<String>> {
    let head_path = git_dir.join("HEAD");
    let content = fs::read_to_string(&head_path)
        .with_context(|| format!("Failed to read {:?}", head_path))?;

    if let Some(stripped) = content.strip_prefix("ref: ") {
        let name = Path::new(stripped.trim())
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        Ok(name)
    } else {
        Ok(None)
    }
}


fn clean_working_directory(current_dir: &Path, git_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path == *git_dir {
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove directory {:?}", path))?;
        } else {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove file {:?}", path))?;
        }
    }

    Ok(())
}



fn has_uncommitted_changes(git_dir: &Path, current_dir: &Path, tree_sha: &str) -> Result<bool> {
    let tree_path = git_dir.join("objects").join(&tree_sha[..2]).join(&tree_sha[2..]);
    let tree_bytes = read_git_object(&tree_path)?;
    let (_header, tree_content) = split_header_and_content(&tree_bytes)?;

    let mut changed = false;
    check_tree_for_changes(git_dir, tree_content, current_dir, current_dir, &mut changed)?;

    Ok(changed)
}


fn check_tree_for_changes(
    git_dir: &Path,
    tree_bytes: &[u8],
    current_dir: &Path,
    path_prefix: &Path,
    changed: &mut bool,
) -> Result<()> {
    let mut cursor = Cursor::new(tree_bytes);

    while (cursor.position() as usize) < tree_bytes.len() {
        let mut mode = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            cursor.read_exact(&mut byte)?;
            if byte[0] == b' ' {
                break;
            }
            mode.push(byte[0]);
        }

        let mut filename = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            cursor.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            filename.push(byte[0]);
        }

        let mut sha_bytes = [0u8; 20];
        cursor.read_exact(&mut sha_bytes)?;

        let mode_str = String::from_utf8_lossy(&mode);
        let filename_str = String::from_utf8_lossy(&filename);
        let sha_hex = sha_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        let file_path = path_prefix.join(&*filename_str);

        if mode_str == "40000" {
            let subtree_path = git_dir.join("objects").join(&sha_hex[..2]).join(&sha_hex[2..]);
            let tree_bytes = read_git_object(&subtree_path)?;
            let (_header, sub_tree_content) = split_header_and_content(&tree_bytes)?;
            check_tree_for_changes(git_dir, sub_tree_content, current_dir, &file_path, changed)?;
        } else {
            let full_path = current_dir.join(&file_path);

            if !full_path.exists() {
                println!("File deleted: {:?}", full_path);
                *changed = true;
            } else {
                let blob_path = git_dir.join("objects").join(&sha_hex[..2]).join(&sha_hex[2..]);
                let blob_bytes = read_git_object(&blob_path)?;
                let (_header, content) = split_header_and_content(&blob_bytes)?;
                let current_content = fs::read(&full_path)?;

                if current_content != content {
                    println!("File modified: {:?}", full_path);
                    *changed = true;
                }
            }
        }
    }

    Ok(())
}
