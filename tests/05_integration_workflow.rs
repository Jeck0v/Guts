use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Test basic repository initialization
#[test]
fn test_init_and_status() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize repository
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized empty Guts repository"));

    // Check status in empty repo
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("No commits yet"))
        .stdout(predicate::str::contains("nothing to commit, working tree clean"));
}

/// Test add and status functionality
#[test]
fn test_add_and_status() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Hello, world!").unwrap();

    // Initialize and add file
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();
    
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("add")
        .arg("test.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added: test.txt"));

    // Check status shows staged file
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes to be committed"))
        .stdout(predicate::str::contains("new file:   test.txt"));
}

/// Test basic commit workflow
#[test]
fn test_commit_workflow() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("readme.txt");
    file.write_str("Project readme").unwrap();

    // Initialize, add, and commit
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("readme.txt").assert().success();

    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));

    // Status should be clean after commit
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to commit, working tree clean"));
}

/// Test file modification detection
#[test]
fn test_file_modification() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("file.txt");
    file.write_str("Original content").unwrap();

    // Initialize, add, commit
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("file.txt").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("commit").arg("-m").arg("Initial").assert().success();

    // Modify file
    file.write_str("Modified content").unwrap();

    // Status should detect modification
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes not staged for commit"))
        .stdout(predicate::str::contains("modified:   file.txt"));
}

/// Test log command with multiple commits
#[test]
fn test_log_command() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("log-test.txt");

    // Initialize
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();

    // First commit
    file.write_str("First version").unwrap();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("log-test.txt").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("commit").arg("-m").arg("First commit").assert().success();

    // Second commit
    file.write_str("Second version").unwrap();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("log-test.txt").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("commit").arg("-m").arg("Second commit").assert().success();

    // Check log shows both commits
    let output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("log")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Second commit"));
    assert!(stdout.contains("First commit"));
}

/// Test ls-files command
#[test]
fn test_ls_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file1 = temp.child("file1.txt");
    let file2 = temp.child("file2.txt");
    file1.write_str("Content 1").unwrap();
    file2.write_str("Content 2").unwrap();

    // Initialize and add files
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("file1.txt").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("file2.txt").assert().success();

    // Check ls-files shows tracked files
    let output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("ls-files")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
}

/// Test show-ref and rev-parse commands
#[test]
fn test_ref_commands() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("ref-test.txt");
    file.write_str("Reference test").unwrap();

    // Initialize, add, commit
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("add").arg("ref-test.txt").assert().success();
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("commit").arg("-m").arg("Reference test").assert().success();

    // Test show-ref
    let output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("show-ref")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("refs/heads/main"));
    
    // Extract hash from show-ref
    let hash = stdout.split_whitespace().next().unwrap();
    assert_eq!(hash.len(), 40);

    // Test rev-parse HEAD
    let output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .unwrap();
    
    let rev_parse_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(hash, rev_parse_hash);
}

/// Test error conditions
#[test]
fn test_error_conditions() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Commands before init should fail
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("add")
        .arg("file.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));

    // Initialize and test empty commit
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).arg("init").assert().success();

    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Empty")
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing to commit"));

    // Add non-existent file
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("add")
        .arg("nonexistent.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("did not match any files"));
}