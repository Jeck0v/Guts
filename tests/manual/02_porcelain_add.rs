use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_add_single_file() {
    // Créer un répertoire temporaire
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("hello.txt");
    file.write_str("Hello, world!\n").unwrap();

    // Initialiser un repo guts
    let _ = guts::core::repo::init(temp.path());

    // Tester la commande `guts add hello.txt`
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("hello.txt");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added: hello.txt"));

    // Vérifier que l'index JSON a été créé
    let index_path = temp.path().join(".git/simple_index.json");
    assert!(index_path.exists());

    // Vérifier le contenu de l'index
    let index_content = fs::read_to_string(&index_path).unwrap();
    assert!(index_content.contains("hello.txt"));
    assert!(index_content.contains("files"));
}

#[test]
fn test_add_multiple_files() {
    // Créer un répertoire temporaire avec plusieurs fichiers
    let temp = assert_fs::TempDir::new().unwrap();
    let file1 = temp.child("file1.txt");
    let file2 = temp.child("file2.txt");
    file1.write_str("Content 1\n").unwrap();
    file2.write_str("Content 2\n").unwrap();

    // Initialiser un repo guts
    let _ = guts::core::repo::init(temp.path());

    // Tester `guts add file1.txt file2.txt`
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("add")
        .arg("file1.txt")
        .arg("file2.txt");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added 2 files"));

    // Vérifier l'index
    let index_path = temp.path().join(".git/simple_index.json");
    let index_content = fs::read_to_string(&index_path).unwrap();
    assert!(index_content.contains("file1.txt"));
    assert!(index_content.contains("file2.txt"));
}

#[test]
fn test_add_workflow_with_status_and_write_tree() {
    // Test complet du workflow add → status → write-tree
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Test content\n").unwrap();

    // Init repo
    let _ = guts::core::repo::init(temp.path());

    // 1. Add file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("test.txt");
    cmd.assert().success();

    // 2. Status should show file as staged
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Changes to be committed"))
        .stdout(predicate::str::contains("test.txt"));

    // 3. Write-tree should create a tree with the file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("write-tree");

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let tree_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(tree_hash.len(), 40); // SHA-1 hash length

    // 4. Cat-file should show the tree contains our file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("cat-file").arg(&tree_hash);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.txt"))
        .stdout(predicate::str::contains("100644"));
}

#[test]
fn test_add_error_file_not_exists() {
    // Test erreur quand le fichier n'existe pas
    let temp = assert_fs::TempDir::new().unwrap();
    let _ = guts::core::repo::init(temp.path());

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("add")
        .arg("nonexistent.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("did not match any files"));
}

#[test]
fn test_add_error_not_git_repo() {
    // Test erreur quand on n'est pas dans un repo git
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("content").unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("test.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));
}
