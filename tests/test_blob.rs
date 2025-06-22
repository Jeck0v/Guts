use std::fs;
use std::io::Write;
use git_rust::core::blob::Blob;
use git_rust::commands::cat_file;

#[test]
fn test_blob_oid_known_content_and_cat_file() {
    // Crée un fichier temporaire avec un contenu connu
    let file_path = "test-blob.txt";
    let content = b"hello world\n";
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(content).unwrap();

    // Crée le blob à partir du fichier
    let blob = Blob::from_file(file_path).unwrap();
    let oid = blob.oid();
    let git_dir = ".git";

    // Écrit le blob dans .git/objects/
    blob.write(git_dir).unwrap();

    // Hash attendu
    let expected_oid = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
    assert_eq!(oid, expected_oid);

    // Utilise cat-file pour afficher le type
    // Par défaut, Rust cache la sortie standard (println!, etc.) des tests qui passent, pour ne pas polluer la sortie.
    // Pour forcer l'affichage de la sortie, il faut utiliser la commande `cargo test -- --nocapture`
    println!("Type de l'objet:"); 
    cat_file::run(git_dir, "-t", &oid).unwrap();

    // Utilise cat-file pour afficher le contenu
    println!("Contenu de l'objet:");
    cat_file::run(git_dir, "-p", &oid).unwrap();

    // Nettoyage
    fs::remove_file(file_path).unwrap();
}
