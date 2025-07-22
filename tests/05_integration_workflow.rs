use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

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