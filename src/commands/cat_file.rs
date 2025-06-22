use crate::core::object::GitObject;

/// Exécute la commande cat-file
pub fn run(git_dir: &str, option: &str, oid: &str) -> Result<(), String> {
    let obj = GitObject::read(git_dir, oid)
        .map_err(|_| format!("Objet introuvable: {}", oid))?;

    match option {
        "-t" => println!("{}", obj.obj_type),
        "-p" => {
            // Affiche le contenu (en texte si possible)
            match std::str::from_utf8(&obj.content) {
                Ok(s) => print!("{}", s),
                Err(_) => println!("{:?}", obj.content),
            }
        }
        _ => return Err("Option invalide (utilisez -t ou -p)".to_string()),
    }
    Ok(())
}
