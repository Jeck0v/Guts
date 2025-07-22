use assert_cmd::Command;
use assert_fs::prelude::*;
use std::process::Command as StdCommand;

#[test]
fn test_commit_tree_matches_git() {
    // Create separate temp directories for git and guts
    let git_temp = assert_fs::TempDir::new().unwrap();
    let guts_temp = assert_fs::TempDir::new().unwrap();
    
    // Create test files in both directories
    let git_file = git_temp.child("test.txt");
    let guts_file = guts_temp.child("test.txt");
    git_file.write_str("Hello, commit-tree!\n").unwrap();
    guts_file.write_str("Hello, commit-tree!\n").unwrap();

    // Initialize git repository
    StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["init", "--quiet"])
        .output()
        .expect("Failed to initialize git repo");

    // Configure git user for commits
    StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["config", "user.name", "Test User"])
        .output()
        .expect("Failed to configure git user.name");

    StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["config", "user.email", "test@example.com"])
        .output()
        .expect("Failed to configure git user.email");

    // Initialize guts repository
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["init"])
        .assert()
        .success();

    // Add file to git index and create tree
    StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["add", "test.txt"])
        .output()
        .expect("Failed to add file to git");

    let git_tree_output = StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["write-tree"])
        .output()
        .expect("Failed to create git tree");
    let git_tree_hash = String::from_utf8_lossy(&git_tree_output.stdout).trim().to_string();

    // Add file to guts index using porcelain command and create tree
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["add", "test.txt"])
        .assert()
        .success();

    let guts_tree_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["write-tree"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let guts_tree_hash = String::from_utf8_lossy(&guts_tree_output).trim().to_string();

    // Trees should match
    assert_eq!(git_tree_hash, guts_tree_hash, "Tree hashes should match between git and guts");

    // Define consistent author/committer and timestamp for reproducible commits
    let timestamp = 1234567890i64;
    let author = "Test User <test@example.com>";
    let message = "Test commit message";

    // Create commit with git using explicit timestamp and author (with timezone)
    let git_commit_output = StdCommand::new("git")
        .current_dir(git_temp.path())
        .env("GIT_AUTHOR_DATE", format!("{} +0000", timestamp))
        .env("GIT_COMMITTER_DATE", format!("{} +0000", timestamp))
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .args(&["commit-tree", &git_tree_hash, "-m", message])
        .output()
        .expect("Failed to create git commit");
    let git_commit_hash = String::from_utf8_lossy(&git_commit_output.stdout).trim().to_string();

    // Create commit with guts using same parameters
    let guts_commit_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&[
            "commit-tree", &guts_tree_hash,
            "-m", message,
            "--author", author,
            "--committer", author,
            "--author-date", &timestamp.to_string(),
            "--committer-date", &timestamp.to_string(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let guts_commit_hash = String::from_utf8_lossy(&guts_commit_output).trim().to_string();

    // Commit hashes should be identical when using same inputs
    assert_eq!(git_commit_hash, guts_commit_hash,
        "Git and Guts should produce identical commit hashes with same inputs\nGit: {}\nGuts: {}",
        git_commit_hash, guts_commit_hash);

    // Verify both commits contain the same tree and message by reading them back
    let git_commit_content = StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["cat-file", "-p", &git_commit_hash])
        .output()
        .expect("Failed to read git commit");

    let guts_commit_content = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["cat-file", &guts_commit_hash])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let git_content = String::from_utf8_lossy(&git_commit_content.stdout);
    let guts_content = String::from_utf8_lossy(&guts_commit_content);

    // Both should reference the same tree
    assert!(git_content.contains(&format!("tree {}", git_tree_hash)), 
        "Git commit should contain tree hash");
    assert!(guts_content.contains(&format!("tree {}", guts_tree_hash)), 
        "Guts commit should contain tree hash");
    
    // Both should contain the commit message
    assert!(git_content.contains(message), "Git commit should contain message");
    assert!(guts_content.contains(message), 
        "Guts commit should contain message");
}