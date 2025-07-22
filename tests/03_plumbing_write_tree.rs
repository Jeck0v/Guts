use assert_cmd::Command;
use assert_fs::prelude::*;
use std::process::Command as StdCommand;

#[test]
fn test_write_tree_matches_git() {
    // Setup temp directory with test files
    let temp = assert_fs::TempDir::new().unwrap();
    let file1 = temp.child("file1.txt");
    let file2 = temp.child("file2.txt");
    file1.write_str("Content of file 1\n").unwrap();
    file2.write_str("Content of file 2\n").unwrap();

    // Create separate temp directories for git and guts
    let git_temp = assert_fs::TempDir::new().unwrap();
    let guts_temp = temp; // Use the original temp for guts
    
    // Copy files to git temp directory
    let git_file1 = git_temp.child("file1.txt");
    let git_file2 = git_temp.child("file2.txt");
    git_file1.write_str("Content of file 1\n").unwrap();
    git_file2.write_str("Content of file 2\n").unwrap();

    // Initialize git repository
    StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["init", "--quiet"])
        .output()
        .expect("Failed to initialize git repo");

    // Initialize guts repository
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["init"])
        .assert()
        .success();

    // Add files to git index
    StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["add", "file1.txt", "file2.txt"])
        .output()
        .expect("Failed to add files to git");

    // Add files to guts index using porcelain command
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["add", "file1.txt"])
        .assert()
        .success();

    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["add", "file2.txt"])
        .assert()
        .success();

    // Create tree with git
    let git_tree_output = StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["write-tree"])
        .output()
        .expect("Failed to create git tree");
    let git_tree_hash = String::from_utf8_lossy(&git_tree_output.stdout).trim().to_string();

    // Create tree with guts
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

    // Compare tree hashes - they should be identical
    assert_eq!(git_tree_hash, guts_tree_hash, 
        "Git and Guts should produce identical tree hashes\nGit: {}\nGuts: {}", 
        git_tree_hash, guts_tree_hash);

    // Verify both trees contain the same content
    let git_tree_content = StdCommand::new("git")
        .current_dir(git_temp.path())
        .args(&["cat-file", "-p", &git_tree_hash])
        .output()
        .expect("Failed to read git tree");

    let guts_tree_content = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(guts_temp.path())
        .args(&["cat-file", &guts_tree_hash])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let git_content = String::from_utf8_lossy(&git_tree_content.stdout);
    let guts_content = String::from_utf8_lossy(&guts_tree_content);

    // Both should contain references to both files
    assert!(git_content.contains("file1.txt"), "Git tree should contain file1.txt");
    assert!(git_content.contains("file2.txt"), "Git tree should contain file2.txt");
    assert!(guts_content.contains("file1.txt"), "Guts tree should contain file1.txt");
    assert!(guts_content.contains("file2.txt"), "Guts tree should contain file2.txt");
}