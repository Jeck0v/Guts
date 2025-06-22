use std::fs;
use std::io::Write;
use git_rust::core::blob::Blob;

#[test]
fn test_blob_oid_known_content() {
    // Crée un fichier temporaire avec un contenu connu
    let file_path = "test-blob.txt";
    let content = b"hello world\n";
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(content).unwrap();

    // Crée le blob à partir du fichier
    let blob = Blob::from_file(file_path).unwrap();

    // Hash attendu
    let expected_oid = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
    assert_eq!(blob.oid(), expected_oid);

    // Nettoyage
    fs::remove_file(file_path).unwrap();
}
