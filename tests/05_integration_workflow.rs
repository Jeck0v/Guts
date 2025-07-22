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

/// Test .gutsignore functionality
#[test]
fn test_gutsignore_functionality() {
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

    // 2. Create .gutsignore file
    let gutsignore = temp_dir.child(".gutsignore");
    gutsignore.write_str("# Test .gutsignore\n*.log\n*.tmp\nbuild/\ntemp/*\n!temp/keep.txt\nnode_modules\n.env\nsecret.key\ntest_*.bak\n").unwrap();
    println!("✅ .gutsignore created");

    // 3. Create test files
    temp_dir.child("README.md").write_str("# Test Project\nThis is a test.").unwrap();
    temp_dir.child("app.log").write_str("[INFO] Application log").unwrap();
    temp_dir.child("temp.tmp").write_str("Temporary file").unwrap();
    temp_dir.child(".env").write_str("SECRET=value").unwrap();
    temp_dir.child("secret.key").write_str("SECRET_KEY_DATA").unwrap();
    temp_dir.child("test_data.bak").write_str("Backup file").unwrap();
    temp_dir.child("important.txt").write_str("Important file").unwrap();

    // Create directories with files
    temp_dir.child("build").create_dir_all().unwrap();
    temp_dir.child("build/output.js").write_str("console.log('build');").unwrap();
    temp_dir.child("temp").create_dir_all().unwrap();
    temp_dir.child("temp/temp_file.txt").write_str("Should be ignored").unwrap();
    temp_dir.child("temp/keep.txt").write_str("Should NOT be ignored").unwrap();
    temp_dir.child("node_modules").create_dir_all().unwrap();
    temp_dir.child("node_modules/package.json").write_str(r#"{"name":"test"}"#).unwrap();

    println!("✅ Test files created");

    // 4. Check status - should only show non-ignored files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    let output = cmd.assert().success();
    let stdout_str = String::from_utf8_lossy(&output.get_output().stdout);
    
    // Files that should be visible (not ignored)
    assert!(stdout_str.contains("README.md"));
    assert!(stdout_str.contains("important.txt"));
    assert!(stdout_str.contains("temp/keep.txt"));
    assert!(stdout_str.contains(".gutsignore"));
    
    // Files that should be ignored (not visible in untracked)
    assert!(!stdout_str.contains("app.log"));
    assert!(!stdout_str.contains("temp.tmp"));
    assert!(!stdout_str.contains(".env"));
    assert!(!stdout_str.contains("secret.key"));
    assert!(!stdout_str.contains("test_data.bak"));
    assert!(!stdout_str.contains("build/output.js"));
    assert!(!stdout_str.contains("temp/temp_file.txt"));
    assert!(!stdout_str.contains("node_modules"));
    
    println!("✅ Status correctly shows only non-ignored files");

    // 5. Try to add ignored files - should be skipped
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("app.log");
    let output = cmd.assert().success();
    let stdout_str = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout_str.contains("Added 0 files"));
    println!("✅ Ignored file (app.log) correctly skipped during add");

    // 6. Add non-ignored file - should work
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("important.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added: important.txt"));
    println!("✅ Non-ignored file (important.txt) successfully added");

    // 7. Test negation pattern - temp/keep.txt should be addable despite temp/*
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("temp/keep.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Added: temp/keep.txt"));
    println!("✅ Negation pattern (!temp/keep.txt) works correctly");

    // 8. Try to add temp/temp_file.txt - should be ignored
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("temp/temp_file.txt");
    let output = cmd.assert().success();
    let stdout_str = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout_str.contains("Added 0 files"));
    println!("✅ File in temp/ directory correctly ignored");

    // 9. Test adding all files with "." - should skip ignored ones
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg(".");
    let output = cmd.assert().success();
    let stdout_str = String::from_utf8_lossy(&output.get_output().stdout);
    
    // Should have added only non-ignored files
    assert!(stdout_str.contains("README.md"));
    assert!(stdout_str.contains(".gutsignore"));
    // Should not mention ignored files
    assert!(!stdout_str.contains("app.log"));
    assert!(!stdout_str.contains("secret.key"));
    println!("✅ Add all (.) correctly skips ignored files");

    // 10. Final status check
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    let output = cmd.assert().success();
    let stdout_str = String::from_utf8_lossy(&output.get_output().stdout);
    
    // Should show staged files
    assert!(stdout_str.contains("Changes to be committed"));
    assert!(stdout_str.contains("new file:   important.txt"));
    assert!(stdout_str.contains("new file:   temp/keep.txt"));
    assert!(stdout_str.contains("new file:   README.md"));
    assert!(stdout_str.contains("new file:   .gutsignore"));
    println!("✅ Final status shows all added files correctly");

    // 11. Test that .gitignore fallback works
    fs::remove_file(temp_dir.path().join(".gutsignore")).unwrap();
    temp_dir.child(".gitignore").write_str("fallback.txt\n").unwrap();
    temp_dir.child("fallback.txt").write_str("Should be ignored by .gitignore").unwrap();
    temp_dir.child("normal.txt").write_str("Should not be ignored").unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    let output = cmd.assert().success();
    let stdout_str = String::from_utf8_lossy(&output.get_output().stdout);
    
    // Should show normal.txt but not fallback.txt
    assert!(stdout_str.contains("normal.txt"));
    assert!(!stdout_str.contains("fallback.txt"));
    println!("✅ .gitignore fallback works correctly");

    println!("🎉 .gutsignore functionality test passed successfully!");
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