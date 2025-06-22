use crate::core::blob::Blob;

/// Exécute la commande hash-object : crée un blob et affiche son OID
pub fn run(file_path: &str, git_dir: &str) -> Result<(), String> {
    let blob = Blob::from_file(file_path)
        .map_err(|_| format!("Fichier introuvable: {}", file_path))?;
    let oid = blob.write(git_dir)
        .map_err(|e| format!("Erreur d'écriture: {}", e))?;
    println!("{}", oid);
    Ok(())
}
