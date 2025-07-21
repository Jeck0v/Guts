use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

/// Basic workflow test that focuses on working functionality
/// Tests: init → add → commit → modify → add → commit
#[test]
fn test_basic_git_workflow() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    println!("🗂️  Created test directory: {}", temp_dir.path().display());

    // 1. Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialized empty Guts repository"));
    println!("✅ Repository initialized");

    // 2. Check initial status
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No commits yet"))
        .stdout(predicate::str::contains("nothing to commit, working tree clean"));
    println!("✅ Initial status correct");

    // 3. Create and add a file
    let readme = temp_dir.child("README.md");
    readme.write_str("# My Project\n\nThis is a test project.\n").unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("README.md");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added: README.md"));
    println!("✅ File added successfully");

    // 4. Check staged status
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Changes to be committed"))
        .stdout(predicate::str::contains("new file:   README.md"));
    println!("✅ File staged correctly");

    // 5. Commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Initial commit");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initial commit"));
    println!("✅ First commit successful");

    // 6. Check clean status
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("nothing to commit, working tree clean"));
    println!("✅ Working tree clean after commit");

    // 7. Modify file
    readme.write_str("# My Project\n\nThis is a test project.\n\n## Updated!\nAdded new content.\n").unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Changes not staged for commit"))
        .stdout(predicate::str::contains("modified:   README.md"));
    println!("✅ File modification detected");

    // 8. Stage modification and commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("README.md");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added: README.md"));

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Update README");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update README"));
    println!("✅ Second commit successful");

    // 9. Test log command shows both commits
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("log");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify both commits are present in log
    assert!(stdout.contains("Update README"));
    assert!(stdout.contains("Initial commit"));
    
    // Verify order: Update README should appear before Initial commit
    let update_pos = stdout.find("Update README").unwrap();
    let initial_pos = stdout.find("Initial commit").unwrap();
    assert!(update_pos < initial_pos, "Update README should appear before Initial commit in log output");
    
    // Verify SHA format for both commits
    assert!(predicate::str::is_match(r"[a-f0-9]{40} Update README").unwrap().eval(&stdout));
    assert!(predicate::str::is_match(r"[a-f0-9]{40} Initial commit").unwrap().eval(&stdout));
    println!("✅ Log command shows commit history correctly");

    // 10. Final status check
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("nothing to commit, working tree clean"));
    println!("✅ Final working tree clean");

    // 11. Verify repository structure
    assert!(temp_dir.path().join("README.md").exists());
    assert!(temp_dir.path().join(".git/HEAD").exists());
    assert!(temp_dir.path().join(".git/refs/heads/main").exists());

    // Check we have valid commits
    let commit_hash = fs::read_to_string(temp_dir.path().join(".git/refs/heads/main")).unwrap();
    assert_eq!(commit_hash.trim().len(), 40);
    println!("✅ Repository structure verified");

    // 12. Test show-ref command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("show-ref");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify show-ref shows main branch with correct format
    assert!(predicate::str::is_match(r"[a-f0-9]{40} refs/heads/main").unwrap().eval(&stdout));
    
    // Verify the hash in show-ref matches the actual HEAD
    let show_ref_hash = stdout
        .lines()
        .find(|line| line.contains("refs/heads/main"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    assert_eq!(show_ref_hash, commit_hash.trim());
    println!("✅ Show-ref command works correctly");

    // 13. Test rev-parse command
    // Test rev-parse with HEAD
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("HEAD");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), commit_hash.trim());
    
    // Test rev-parse with branch name
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("main");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), commit_hash.trim());
    
    // Test rev-parse with full SHA
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg(commit_hash.trim());
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), commit_hash.trim());
    println!("✅ Rev-parse command works correctly");

    // 14. Test ls-files command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify ls-files shows the tracked file
    assert!(stdout.contains("README.md"));
    assert_eq!(stdout.trim(), "README.md", "ls-files should show only README.md");
    println!("✅ ls-files command works correctly");

    // 15. Test ls-tree command
    // First get the tree hash from the current commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("cat-file").arg(commit_hash.trim());
    let output = cmd.output().unwrap();
    let commit_content = String::from_utf8_lossy(&output.stdout);
    
    // Extract tree hash
    let tree_hash = commit_content
        .lines()
        .find(|line| line.starts_with("tree: "))
        .unwrap()
        .strip_prefix("tree: ")
        .unwrap()
        .trim();
    
    // Test ls-tree with the tree hash
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-tree").arg(tree_hash);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify ls-tree shows the blob entry for README.md
    assert!(stdout.contains("README.md"));
    assert!(predicate::str::is_match(r"100644 blob [a-f0-9]{40}\tREADME\.md").unwrap().eval(&stdout));
    println!("✅ ls-tree command works correctly");

    println!("🎉 Basic workflow test passed successfully!");
}

/// Test error conditions
#[test]
fn test_error_conditions() {
    let temp_dir = assert_fs::TempDir::new().unwrap();

    // 1. Commands before init should fail appropriately
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("file.txt");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a git repository"));

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("log");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("fatal: not a git repository"));

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("show-ref");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fatal: not a git repository"));

    // 2. Init and try empty commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();

    // 2a. Log command should fail when no commits exist
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("log");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("branch exists but no commits yet"));

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Empty");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("nothing to commit"));

    // 3. Add non-existent file
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("nonexistent.txt");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("did not match any files"));

    // 4. Show-ref in repo with no commits should show empty output
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("show-ref");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "show-ref should be empty when no commits exist");

    println!("✅ Error conditions handled correctly");
}