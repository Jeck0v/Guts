use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_commit_success() {
    // Create temporary directory
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("hello.txt");
    file.write_str("Hello, world!\n").unwrap();

    // Initialize guts repo
    let _ = guts::core::repo::init(temp.path());

    // Add file to staging area
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("hello.txt");
    cmd.assert().success();

    // Test commit command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Initial commit");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));

    // Verify HEAD was created/updated
    let head_path = temp.path().join(".git/HEAD");
    assert!(head_path.exists());

    // Verify refs/heads/main was created
    let main_ref_path = temp.path().join(".git/refs/heads/main");
    assert!(main_ref_path.exists());

    // Verify commit hash is valid (40 characters)
    let commit_hash = fs::read_to_string(&main_ref_path).unwrap();
    assert_eq!(commit_hash.trim().len(), 40);

    // Verify index was cleared after commit
    let index_path = temp.path().join(".git/simple_index.json");
    let index_content = fs::read_to_string(&index_path).unwrap();
    let index: serde_json::Value = serde_json::from_str(&index_content).unwrap();
    assert!(index["files"].as_object().unwrap().is_empty());
}

#[test]
fn test_commit_with_parent() {
    // Create temporary directory
    let temp = assert_fs::TempDir::new().unwrap();
    let file1 = temp.child("file1.txt");
    let file2 = temp.child("file2.txt");
    file1.write_str("Content 1\n").unwrap();
    file2.write_str("Content 2\n").unwrap();

    // Initialize guts repo
    let _ = guts::core::repo::init(temp.path());

    // First commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("file1.txt");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("First commit");
    cmd.assert().success();

    // Get first commit hash
    let main_ref_path = temp.path().join(".git/refs/heads/main");
    let first_commit = fs::read_to_string(&main_ref_path).unwrap().trim().to_string();

    // Second commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("file2.txt");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Second commit");
    cmd.assert().success();

    // Get second commit hash
    let second_commit = fs::read_to_string(&main_ref_path).unwrap().trim().to_string();

    // Verify they are different commits
    assert_ne!(first_commit, second_commit);

    // Verify second commit object contains parent reference
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("cat-file").arg(&second_commit);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&first_commit))
        .stdout(predicate::str::contains("Second commit"));
}

#[test]
fn test_commit_nothing_to_commit() {
    // Create temporary directory
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize guts repo
    let _ = guts::core::repo::init(temp.path());

    // Try to commit without staging anything
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Empty commit");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("nothing to commit"));
}

#[test]
fn test_commit_error_not_git_repo() {
    // Test error when not in a git repository
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Test commit");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));
}

#[test]
fn test_commit_workflow_complete() {
    // Complete workflow test: init → add → status → commit → status
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Test content\n").unwrap();

    // 1. Init repo
    let _ = guts::core::repo::init(temp.path());

    // 2. Add file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("test.txt");
    cmd.assert().success();

    // 3. Status should show staged file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Changes to be committed"))
        .stdout(predicate::str::contains("test.txt"));

    // 4. Commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Add test file");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Add test file"));

    // 5. Status should show clean working tree (files are now committed)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("nothing to commit, working tree clean"));
}