use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Test the `guts rev-parse` command
/// This tests SHA resolution, branch reference, and HEAD resolution
#[test]
fn test_rev_parse_manual() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    println!("🗂️  Created test directory: {}", temp_dir.path().display());

    // 1. Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();
    println!("✅ Repository initialized");

    // 2. Create and commit a file to have something to reference
    let readme = temp_dir.child("README.md");
    readme.write_str("# Test Project\n").unwrap();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("README.md");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Initial commit");
    cmd.assert().success();
    println!("✅ Initial commit created");

    // Get the actual commit hash from main branch
    let main_ref_path = temp_dir.path().join(".git/refs/heads/main");
    let expected_sha = std::fs::read_to_string(&main_ref_path).unwrap().trim().to_string();
    println!("📋 Expected SHA: {}", expected_sha);

    // 3. Test rev-parse with HEAD
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("HEAD");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&expected_sha));
    println!("✅ rev-parse HEAD works");

    // 4. Test rev-parse with branch name
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("main");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&expected_sha));
    println!("✅ rev-parse main works");

    // 5. Test rev-parse with full SHA
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg(&expected_sha);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&expected_sha));
    println!("✅ rev-parse with SHA works");

    // 6. Test rev-parse with non-existent reference
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("nonexistent");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Reference 'nonexistent' not found"));
    println!("✅ rev-parse error handling works");

    // 7. Test rev-parse outside git repository
    let non_git_dir = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(non_git_dir.path()).arg("rev-parse").arg("HEAD");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Cannot get current directory").or(
            predicate::str::contains("not a git repository")
        ));
    println!("✅ rev-parse outside git repo handled");

    println!("🎉 All rev-parse tests passed!");
}

/// Test rev-parse edge cases
#[test] 
fn test_rev_parse_edge_cases() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    
    // Initialize repo
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();

    // Test rev-parse HEAD in empty repository (should fail)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("HEAD");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("branch exists but no commits yet").or(
            predicate::str::contains("Reference 'HEAD' not found")
        ));
    println!("✅ rev-parse HEAD in empty repo handled");

    // Test with invalid SHA format (not 40 chars)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("abc123");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Reference 'abc123' not found"));
    println!("✅ rev-parse invalid SHA handled");

    println!("🎉 All rev-parse edge case tests passed!");
}