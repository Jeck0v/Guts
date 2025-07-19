// Module pour un index Git simple en format JSON
// Alternative pédagogique à l'index binaire Git complexe

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use crate::core::{blob, hash};

/// Structure simple pour l'index Git
/// Stocke uniquement les fichiers "stagés" avec leur hash SHA-1
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SimpleIndex {
    /// Map : chemin relatif du fichier -> hash SHA-1 du contenu
    pub files: HashMap<String, String>,
}

impl SimpleIndex {
    /// Charge l'index depuis .git/simple_index.json
    /// Si le fichier n'existe pas, retourne un index vide
    pub fn load() -> Result<Self> {
        let index_path = get_simple_index_path()?;
        
        if !index_path.exists() {
            return Ok(SimpleIndex::default());
        }

        let content = fs::read_to_string(&index_path)
            .with_context(|| format!("impossible de lire {:?}", index_path))?;
        
        let index: SimpleIndex = serde_json::from_str(&content)
            .with_context(|| "JSON invalide dans l'index")?;
        
        Ok(index)
    }

    /// Sauvegarde l'index dans .git/simple_index.json
    pub fn save(&self) -> Result<()> {
        let index_path = get_simple_index_path()?;
        
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "impossible de sérialiser l'index")?;
        
        fs::write(&index_path, content)
            .with_context(|| format!("impossible d'écrire {:?}", index_path))?;
        
        Ok(())
    }

    /// Ajoute un fichier à l'index (= le "stage" pour le prochain commit)
    pub fn add_file(&mut self, file_path: &Path) -> Result<()> {
        // Convertir en chemin absolu si nécessaire
        let absolute_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(file_path)
        };

        // Lire le contenu du fichier
        let content = fs::read(&absolute_path)
            .with_context(|| format!("impossible de lire {:?}", absolute_path))?;

        // Créer un blob Git et calculer son hash SHA-1
        let blob = blob::Blob::new(content);
        let file_hash = hash::write_object(&blob)?;

        // Convertir en chemin relatif depuis la racine du repo
        let relative_path = get_relative_path(&absolute_path)?;

        // Ajouter à notre map
        self.files.insert(relative_path, file_hash);
        
        Ok(())
    }

    /// Vérifie si un fichier est dans l'index (stagé)
    pub fn contains_file(&self, file_path: &str) -> bool {
        self.files.contains_key(file_path)
    }

    /// Retourne la liste des fichiers stagés
    pub fn get_staged_files(&self) -> Vec<&String> {
        self.files.keys().collect()
    }
}

/// Trouve la racine du repository Git (dossier qui contient .git/)
fn find_repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()
        .with_context(|| "impossible d'obtenir le répertoire courant")?;

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return Err(anyhow!("pas un repository git")),
        }
    }
}

/// Retourne le chemin vers .git/simple_index.json
fn get_simple_index_path() -> Result<PathBuf> {
    let repo_root = find_repo_root()?;
    Ok(repo_root.join(".git").join("simple_index.json"))
}

/// Convertit un chemin absolu en chemin relatif depuis la racine du repo
fn get_relative_path(file_path: &Path) -> Result<String> {
    let repo_root = find_repo_root()?;
    let relative = file_path.strip_prefix(&repo_root)
        .with_context(|| "le fichier n'est pas dans le repository")?;
    Ok(relative.to_string_lossy().to_string())
}

/// Vérifie si on est dans un repository Git
pub fn is_git_repository() -> Result<bool> {
    match find_repo_root() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Fonction publique pour ajouter un fichier à l'index
/// C'est cette fonction que va appeler la commande `guts add`
pub fn add_file_to_index(file_path: &Path) -> Result<()> {
    let mut index = SimpleIndex::load()?;
    index.add_file(file_path)?;
    index.save()?;
    Ok(())
}