use std::collections::HashSet;
use std::path::PathBuf;
use anyhow::{Result};
use clap::Args;
use walkdir::WalkDir;
use crate::core::simple_index;

/// CLI arguments for the `status` command.
#[derive(Args)]
pub struct StatusObject {
    // Optionnel: chemin custom vers .git (par défaut current/.git)
    pub guts_dir: Option<PathBuf>,
}

/// Point d'entrée pour la commande `guts status`
/// Version adaptée pour l'index JSON simple
pub fn run(args: &StatusObject) -> Result<String> {
    // Vérifier qu'on est dans un repo git
    if !simple_index::is_git_repository()? {
        return Ok("fatal: not a git repository".to_string());
    }

    // Charger notre index JSON simple
    let index = simple_index::SimpleIndex::load()?;

    // Lister tous les fichiers dans le working directory (excluant .git)
    let work_files = list_working_dir_files()?;

    let mut output = String::new();
    output.push_str("On branch main\n");
    output.push_str("Your branch is up to date with 'origin/main'.\n");
    output.push_str("\n");

    // Créer un set des fichiers du working directory pour recherche rapide
    let work_files_set: HashSet<_> = work_files.iter().collect();

    // Fichiers stagés (dans l'index)
    let staged_files: Vec<&String> = index.get_staged_files();
    
    // 1. Fichiers stagés pour commit
    if !staged_files.is_empty() {
        output.push_str("Changes to be committed:\n");
        output.push_str("  (use \"git reset HEAD <file>...\" to unstage)\n");
        for file_path in &staged_files {
            output.push_str(&format!("        new file:   {}\n", file_path));
        }
        output.push_str("\n");
    }

    // 2. Fichiers non trackés (présents dans working dir mais pas dans index)
    let mut untracked_files = Vec::new();
    for work_file in &work_files {
        let relative_path = get_relative_path(work_file)?;
        if !index.contains_file(&relative_path) {
            untracked_files.push(relative_path);
        }
    }

    if !untracked_files.is_empty() {
        output.push_str("Untracked files:\n");
        output.push_str("  (use \"git add <file>...\" to include in what will be committed)\n");
        for file in &untracked_files {
            output.push_str(&format!("        {}\n", file));
        }
        output.push_str("\n");
    }

    // 3. Fichiers supprimés (dans l'index mais pas dans working dir)
    for staged_file in &staged_files {
        let full_path = std::env::current_dir()?.join(staged_file);
        if !work_files_set.contains(&full_path) {
            output.push_str(&format!("        deleted:    {}\n", staged_file));
        }
    }

    // Message final si tout est propre
    if staged_files.is_empty() && untracked_files.is_empty() {
        output.push_str("nothing to commit, working tree clean\n");
    }

    Ok(output)
}

/// Liste récursivement tous les fichiers du working directory, en excluant .git
fn list_working_dir_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let current_dir = std::env::current_dir()?;

    let walker = WalkDir::new(&current_dir)
        .into_iter()
        .filter_entry(|e| {
            // Exclure le dossier .git
            !e.path().components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s == ".git"
            })
        });

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }

    Ok(files)
}

/// Convertit un chemin absolu en chemin relatif depuis la racine du repo
fn get_relative_path(file_path: &PathBuf) -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let relative = file_path.strip_prefix(&current_dir)
        .map_err(|_| anyhow::anyhow!("le fichier n'est pas dans le répertoire courant"))?;
    Ok(relative.to_string_lossy().to_string())
}