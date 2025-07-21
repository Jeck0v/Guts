use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
#[ignore]
fn test_log_single_commit() {
    // Create temporary directory
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("hello.txt");
    file.write_str("Hello, world!\n").unwrap();

    // Initialize guts repo
    let _ = guts::core::repo::init(temp.path());

    // Add and commit file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("hello.txt");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Initial commit");
    cmd.assert().success();

    // Test log command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("log");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"))
        .stdout(predicate::str::is_match(r"^[a-f0-9]{40} Initial commit\n$").unwrap());
}

#[test]
#[ignore]
fn test_log_multiple_commits() {
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

    // Test log command shows both commits in reverse chronological order
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("log");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify both commits are present
    assert!(stdout.contains("First commit"));
    assert!(stdout.contains("Second commit"));
    
    // Verify order: Second commit should appear before First commit
    let second_pos = stdout.find("Second commit").unwrap();
    let first_pos = stdout.find("First commit").unwrap();
    assert!(second_pos < first_pos, "Second commit should appear before First commit in log output");

    // Verify SHA format for both commits
    assert!(predicate::str::is_match(r"[a-f0-9]{40} Second commit").unwrap().eval(&stdout));
    assert!(predicate::str::is_match(r"[a-f0-9]{40} First commit").unwrap().eval(&stdout));
}

#[test]
#[ignore]
fn test_log_multiline_commit_message() {
    // Test that only the first line of commit message is shown
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Test content\n").unwrap();

    // Initialize guts repo
    let _ = guts::core::repo::init(temp.path());

    // Add and commit with multiline message
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("test.txt");
    cmd.assert().success();

    let multiline_message = "First line of commit\n\nThis is the body of the commit\nwith multiple lines";
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg(multiline_message);
    cmd.assert().success();

    // Test log command shows only first line
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("log");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("First line of commit"))
        .stdout(predicate::str::contains("This is the body").not())
        .stdout(predicate::str::contains("with multiple lines").not());
}

#[test]
#[ignore]
fn test_log_no_commits() {
    // Test log command when no commits exist
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize guts repo but don't make any commits
    let _ = guts::core::repo::init(temp.path());

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("log");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("branch exists but no commits yet"));
}

#[test]
#[ignore]
fn test_log_not_git_repo() {
    // Test log command when not in a git repository
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("log");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("fatal: not a git repository"));
}

#[test]
#[ignore]
fn test_log_with_custom_git_dir() {
    // Test log command with custom .git directory path
    let temp = assert_fs::TempDir::new().unwrap();
    let custom_git_dir = temp.child("custom_git");
    let file = temp.child("test.txt");
    file.write_str("Test content\n").unwrap();

    // Initialize with custom directory
    let _ = guts::core::repo::init(custom_git_dir.path());

    // Create simple_index.json in custom directory
    let index_path = custom_git_dir.path().join("simple_index.json");
    fs::write(&index_path, r#"{"files":{}}"#).unwrap();

    // Add file using custom git dir
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("--guts-dir")
        .arg(custom_git_dir.path())
        .arg("add")
        .arg("test.txt");
    cmd.assert().success();

    // Commit using custom git dir
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("--guts-dir")
        .arg(custom_git_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Custom dir commit");
    cmd.assert().success();

    // Test log with custom git dir
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("--guts-dir")
        .arg(custom_git_dir.path())
        .arg("log");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Custom dir commit"));
}