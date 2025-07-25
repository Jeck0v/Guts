use anyhow::{Context, Result};
use clap::Args;
use std::{fs, path::{PathBuf, Path}};

#[derive(Args)]
pub struct MergeObject {
    /// Chemin vers la branche actuelle (ex: .git/refs/heads/main)
    pub branch: PathBuf,

    /// Nom de la branche à merger (ex: "feature")
    pub merge_branch: String,
}

pub fn run(args: &MergeObject) -> Result<String> {
    let current_dir = std::env::current_dir().context("Cannot get the current directory")?;
    let git_dir = current_dir.join(".git");

    let current_branch_path = &args.branch;
    let target_branch_name = &args.merge_branch;

    println!("Current branch path: {:?}", current_branch_path);
    println!("Target branch name: {}", target_branch_name);

    // 🔹 Lire les commits pointés par chaque branche
    let current_commit = read_ref_commit_sha(current_branch_path)?;
    let other_commit = read_ref_commit_sha(&git_dir.join("refs").join("heads").join(target_branch_name))?;

    println!("Current commit: {}", current_commit);
    println!("Other commit: {}", other_commit);

    // 3. Trouver l'ancêtre commun (à faire ensuite)
    // 4. Lire les arbres des 3 commits
    // 5. Fusionner les arbres
    // 6. Écrire un commit de merge

    let test = "test";

    Ok(test.to_string())
}

fn read_ref_commit_sha(ref_path: &Path) -> Result<String> {
    let content = fs::read_to_string(ref_path)
        .with_context(|| format!("Failed to read branch ref {:?}", ref_path))?;
    Ok(content.trim().to_string())
}
