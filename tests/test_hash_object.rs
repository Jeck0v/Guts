use std::fs;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_hash_object_creates_blob_and_prints_oid() {
    // Préparer un répertoire temporaire avec un fichier de test
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("hello.txt");
    file.write_str("Hello, world!\n").unwrap();

    // Initialiser un dépôt .guts
    let _ = guts::core::repo::init(temp.path());

    // Exécuter la commande `guts hash-object <file>`
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("hash-object")
        .arg("hello.txt");

    // Capturer la sortie
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"^[a-f0-9]{40}\n$").unwrap());

    // Vérifier que le fichier blob a bien été écrit
    let oid_output = cmd.output().unwrap().stdout;
    let oid = String::from_utf8_lossy(&oid_output).trim().to_string();
    let (dir, file_name) = oid.split_at(2);

    let object_path = temp
        .path()
        .join(".guts/objects")
        .join(dir)
        .join(file_name);
    assert!(object_path.exists());
}
