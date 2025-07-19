use std::path::PathBuf;
use anyhow::Result;
use clap::Args;
use crate::core::{hash, simple_index};
use crate::core::object::{Tree, TreeEntry};

#[derive(Args)]
pub struct WriteTreeArgs {
    pub dir: Option<PathBuf>,
}

/// Nouvelle version de write-tree qui utilise l'index JSON simple
/// Au lieu de lire le filesystem, lit l'index pour créer le tree
pub fn run(args: &WriteTreeArgs) -> Result<String> {
    // Vérifier qu'on est dans un repo git
    if !simple_index::is_git_repository()? {
        return Err(anyhow::anyhow!("fatal: not a git repository"));
    }

    // Charger l'index JSON
    let index = simple_index::SimpleIndex::load()?;
    
    // Créer le tree depuis l'index (pas le filesystem)
    let tree = build_tree_from_index(&index)?;
    
    // Écrire le tree object et retourner son hash
    let oid = hash::write_object(&tree)?;

    Ok(oid)
}

/// Construit un tree Git object depuis l'index JSON
/// Plus simple et correct que de scanner le filesystem
fn build_tree_from_index(index: &simple_index::SimpleIndex) -> Result<Tree> {
    let mut entries = Vec::new();

    // Pour chaque fichier dans l'index
    for (file_path, file_hash) in &index.files {
        // Décoder le hash SHA-1 hex en bytes
        let hash_bin = hex::decode(file_hash)
            .map_err(|_| anyhow::anyhow!("invalid SHA-1 hash: {}", file_hash))?;

        // Créer le hash array de 20 bytes
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&hash_bin);

        // Extraire juste le nom du fichier (pas le chemin complet)
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid file path: {}", file_path))?
            .to_string_lossy()
            .to_string();

        // Créer l'entrée du tree
        entries.push(TreeEntry {
            mode: "100644".to_string(), // Mode pour fichier normal
            name: file_name,
            hash,
        });
    }

    // Trier les entrées par nom (requis par Git)
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Tree { entries })
}