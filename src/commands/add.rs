use std::path::PathBuf;
use anyhow::{anyhow, Result};
use clap::Args;
use crate::core::simple_index;

/// Arguments pour la commande `guts add`
#[derive(Args)]
pub struct AddArgs {
    /// Fichier(s) à ajouter au staging area
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

/// Fonction principale de la commande `guts add`
/// Ajoute des fichiers au staging area (index)
pub fn run(args: &AddArgs) -> Result<String> {
    // Vérifier qu'on est dans un repo git
    if !simple_index::is_git_repository()? {
        return Err(anyhow!("fatal: not a git repository"));
    }

    let mut added_files = Vec::new();
    let mut output = String::new();

    // Traiter chaque fichier demandé
    for file_path in &args.files {
        // Vérifications de base
        if !file_path.exists() {
            return Err(anyhow!("pathspec '{}' did not match any files", file_path.display()));
        }

        if file_path.is_dir() {
            return Err(anyhow!("'{}' is a directory", file_path.display()));
        }

        // Ajouter le fichier à l'index JSON
        simple_index::add_file_to_index(file_path)?;
        added_files.push(file_path.display().to_string());
    }

    // Message de confirmation
    if added_files.len() == 1 {
        output.push_str(&format!("Ajouté : {}", added_files[0]));
    } else {
        output.push_str(&format!("Ajoutés {} fichiers :", added_files.len()));
        for file in &added_files {
            output.push_str(&format!("\n  - {}", file));
        }
    }

    Ok(output)
}