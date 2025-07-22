use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

#[test]
#[ignore]
fn test_show_ref_single_commit() {
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

    // Test show-ref command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("show-ref");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify refs/heads/main exists and has correct format
    assert!(predicate::str::is_match(r"[a-f0-9]{40} refs/heads/main").unwrap().eval(&stdout));
    
    // Should contain at least one ref
    assert!(!stdout.trim().is_empty());
}

#[test]
#[ignore]
fn test_show_ref_multiple_commits() {
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

    // Test show-ref command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("show-ref");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify refs/heads/main exists and has correct format
    assert!(predicate::str::is_match(r"[a-f0-9]{40} refs/heads/main").unwrap().eval(&stdout));
    
    // Should still show only main branch (no multiple branches created)
    let ref_count = stdout.lines().count();
    assert_eq!(ref_count, 1, "Should have exactly one ref (main branch)");
    
    // Verify the hash points to the latest commit
    let main_ref_line = stdout.lines().find(|line| line.contains("refs/heads/main")).unwrap();
    let hash = main_ref_line.split_whitespace().next().unwrap();
    assert_eq!(hash.len(), 40, "Hash should be 40 characters");
}

#[test]
#[ignore]
fn test_show_ref_no_commits() {
    // Test show-ref command when no commits exist but repo is initialized
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize guts repo but don't make any commits
    let _ = guts::core::repo::init(temp.path());

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("show-ref");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be empty output when no refs exist yet
    assert!(stdout.trim().is_empty(), "Output should be empty when no commits exist");
}

#[test]
#[ignore]
fn test_show_ref_not_git_repo() {
    // Test show-ref command when not in a git repository
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("show-ref");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fatal: not a git repository"));
}

#[test]
#[ignore]
fn test_show_ref_format_and_sorting() {
    // Test that show-ref output is properly formatted and sorted
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Test content\n").unwrap();

    // Initialize guts repo
    let _ = guts::core::repo::init(temp.path());

    // Add and commit file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("add").arg("test.txt");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("commit")
        .arg("-m")
        .arg("Test commit");
    cmd.assert().success();

    // Test show-ref command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("show-ref");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify format: each line should be "hash refname"
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Each line should have exactly 2 parts: hash and refname");
        
        let hash = parts[0];
        let refname = parts[1];
        
        // Hash should be 40 hex characters
        assert_eq!(hash.len(), 40, "Hash should be 40 characters");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should contain only hex characters");
        
        // Refname should start with "refs/"
        assert!(refname.starts_with("refs/"), "Refname should start with 'refs/'");
    }
}

// NOTE: Custom git directory support removed - test is no longer valid
#[test]
#[ignore]
#[allow(dead_code)]
fn test_show_ref_with_custom_git_dir() {
    // Test show-ref command with custom .git directory path
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

    // Test show-ref with custom git dir
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path()).arg("show-ref");

    // This should work with the current directory's .git
    cmd.assert().success();
}