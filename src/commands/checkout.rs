use anyhow::{Context, Result};
use clap::Args;
use crate::core::resolve_parse::resolve_ref;
use flate2::read::ZlibDecoder;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::core::parse_tree::{parse_tree};

#[derive(Args)]
pub struct CheckoutObject {
    pub name: Option<String>,

    #[arg(short = 'b', long)]
    pub branch_name: Option<String>,

    #[arg(last = true)]
    pub dir: Option<PathBuf>,
}

pub fn run(args: &CheckoutObject) -> Result<String> {

    let original_dir = std::env::current_dir()?;

    if let Some(dir) = &args.dir {
        std::env::set_current_dir(dir)?;
    }

    let current_dir = std::env::current_dir().context("Cannot get the current directory")?;
    let git_dir = current_dir.join(".git");

    let target_ref = if let Some(name) = &args.name {
        name.clone()
    } else {
        read_head_ref(&git_dir)?
            .ok_or_else(|| anyhow::anyhow!("HEAD is detached. Please specify a branch or commit to checkout"))?
    };

    let sha = resolve_ref(&git_dir, &target_ref)?;

    let commit_content = read_and_parse_git_object(&git_dir, &sha)?;
    
    let commit_str = std::str::from_utf8(&commit_content)
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
            let possible_branch_path = git_dir.join("refs").join("heads").join(&target_ref);
            if possible_branch_path.exists() {
                let head_path = git_dir.join("HEAD");
                std::fs::write(&head_path, format!("ref: refs/heads/{}\n", &target_ref))
                    .with_context(|| format!("failed to update HEAD to point to {}", &target_ref))?;
            }
        }

        clean_working_directory(&current_dir, &git_dir, &tree_sha)?;
    
    
        let tree_content = read_and_parse_git_object(&git_dir, &tree_sha)?;
        parse_tree_object(&git_dir, &tree_content, current_dir)?;
    
        std::env::set_current_dir(&original_dir)?;
        
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

fn parse_tree_object(git_dir: &PathBuf, tree_bytes: &[u8], target_dir: PathBuf) -> Result<()> {
    for entry in parse_tree(&tree_bytes)? {
        let full_path = target_dir.join(&entry.filename);

        if entry.mode == "40000" {
            fs::create_dir_all(&full_path)?;
            let sub_tree_content = read_and_parse_git_object(&git_dir, &entry.sha)?;
            parse_tree_object(&git_dir, &sub_tree_content, full_path)?;
        } else {
            let blob_content = read_and_parse_git_object(&git_dir, &entry.sha)?;
            fs::create_dir_all(&full_path.parent().unwrap())?;
            let mut file = File::create(&full_path)?;
            file.write_all(&blob_content)?;
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

fn clean_working_directory(current_dir: &Path, git_dir: &Path, tree_sha: &str) -> Result<()> {
    let mut tracked_paths = HashSet::new();
    collect_tracked_paths(git_dir, tree_sha, PathBuf::new(), &mut tracked_paths)?;

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path == *git_dir {
            continue;
        }

        let relative_path = path.strip_prefix(current_dir).unwrap();

        if tracked_paths.contains(relative_path) {
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

fn collect_tracked_paths(
    git_dir: &Path,
    tree_sha: &str,
    base_path: PathBuf,
    paths: &mut HashSet<PathBuf>,
) -> Result<()> {
    let tree_content = read_and_parse_git_object(&git_dir, &tree_sha)?;

    for entry in parse_tree(&tree_content)? {
        let mut full_path = base_path.clone();
        full_path.push(&entry.filename);

        paths.insert(full_path.clone());

        if entry.mode == "40000" {
            collect_tracked_paths(git_dir, &entry.sha, full_path, paths)?;
        }
    }

    Ok(())
}

fn has_uncommitted_changes(git_dir: &Path, current_dir: &Path, tree_sha: &str) -> Result<bool> {
    
    let current_head_tree = read_head_tree_sha(git_dir)?;
    
    let tracked_files = list_files_in_tree(git_dir, &current_head_tree)?;
    
    let mut changed = false;
    check_tree_for_changes(git_dir, current_dir, current_dir, &tracked_files, &mut changed)?;

    Ok(changed)
}

fn check_tree_for_changes(
    git_dir: &Path,
    current_dir: &Path,
    path_prefix: &Path,
    tracked_files: &HashSet<PathBuf>,
    changed: &mut bool,
) -> Result<()> {
    for entry in fs::read_dir(path_prefix)? {
        let entry = entry?;
        let path = entry.path();

        if path == *git_dir {
            continue;
        }

        let relative_path = path.strip_prefix(current_dir).unwrap().to_path_buf();

        if path.is_dir() {
            check_tree_for_changes(git_dir, current_dir, &path, tracked_files, changed)?;
        } else {
            let is_tracked = tracked_files.contains(&relative_path);

            if is_tracked {
                if let Some(blob_sha) = find_blob_sha_for_path(git_dir, &relative_path)? {
                    let blob_path = git_dir.join("objects").join(&blob_sha[..2]).join(&blob_sha[2..]);
                    let blob_bytes = read_git_object(&blob_path)?;
                    let (_header, content) = split_header_and_content(&blob_bytes)?;
                    let current_content = fs::read(&path)?;

                    if current_content != content {
                        *changed = true;
                    }
                } else {
                    println!("DEBUG: Could not find blob SHA for tracked file: {:?}", relative_path);
                }
            } else {
                *changed = true;
            }
        }
    }

    for tracked_file in tracked_files {
        let full_path = current_dir.join(tracked_file);
        if !full_path.exists() {
            *changed = true;
        }
    }

    Ok(())
}

fn list_files_in_tree(git_dir: &Path, tree_sha: &str) -> Result<HashSet<PathBuf>> {
    let mut files = HashSet::new();
    list_files_recursive(git_dir, tree_sha, PathBuf::new(), &mut files)?;
    Ok(files)
}

fn list_files_recursive(
    git_dir: &Path,
    tree_sha: &str,
    prefix: PathBuf,
    files: &mut HashSet<PathBuf>,
) -> Result<()> {
    let tree_content = read_and_parse_git_object(git_dir, &tree_sha)?;

    for entry in parse_tree(&tree_content)? {
        let current_path = prefix.join(&entry.filename);

        if entry.mode == "40000" {
            list_files_recursive(git_dir, &entry.sha, current_path, files)?;
        } else {
            files.insert(current_path);
        }
    }
    
    Ok(())
}

fn find_blob_sha_for_path(git_dir: &Path, relative_path: &Path) -> Result<Option<String>> {
    let mut current_tree_sha = read_head_tree_sha(git_dir)?;

    for component in relative_path.components() {
        let component_str = component.as_os_str().to_string_lossy();

        let tree_content = read_and_parse_git_object(git_dir, &current_tree_sha)?;

        let entries = parse_tree(&tree_content)?;

        let found_entry = entries.iter().find(|entry| entry.filename == component_str);

        if let Some(entry) = found_entry {
            if component == relative_path.components().last().unwrap() {
                return Ok(Some(entry.sha.clone()));
            } else if entry.mode == "40000" {
                    current_tree_sha = entry.sha.clone();
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
        }
    }

    Ok(None)
}

fn read_head_tree_sha(git_dir: &Path) -> Result<String> {
    let head_ref = read_head_ref(git_dir)?
        .ok_or_else(|| anyhow::anyhow!("HEAD is detached or invalid"))?;

    let ref_path = git_dir.join("refs").join("heads").join(&head_ref);
    let commit_sha = fs::read_to_string(&ref_path)
        .context("Failed to read HEAD ref file")?;
    let commit_sha = commit_sha.trim();

    let commit_content = read_and_parse_git_object(git_dir, &commit_sha)?;

    let commit_str = std::str::from_utf8(&commit_content)
        .context("Commit content is not valid UTF-8")?;

    extract_tree_sha(commit_str)
}


fn read_and_parse_git_object(git_dir: &Path, sha: &str) -> Result<Vec<u8>> {
    let obj_path = git_dir.join("objects").join(&sha[..2]).join(&sha[2..]);
    let bytes = read_git_object(&obj_path)?;
    let (_header, content) = split_header_and_content(&bytes)?;
    Ok(content.to_vec())
}