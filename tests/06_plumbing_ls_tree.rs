use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_ls_tree_lists_tree_contents() -> Result<()> {
    // Prepare a temporary directory with test files
    let temp = assert_fs::TempDir::new().unwrap();
    let file1 = temp.child("file1.txt");
    let file2 = temp.child("file2.txt");
    file1.write_str("Hello world 1").unwrap();
    file2.write_str("Hello world 2").unwrap();

    // Initialize a .git repository
    let _ = guts::core::repo::init(temp.path());

    // Add files to staging area
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("add")
        .arg("file1.txt")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("add")
        .arg("file2.txt")
        .assert()
        .success();

    // Create a tree from the staged files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    let output = cmd.current_dir(temp.path())
        .arg("write-tree")
        .output()
        .unwrap();

    assert!(output.status.success());
    let tree_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Test ls-tree command
    let mut cmd = Command::cargo_bin("guts").unwrap();
    let output = cmd.current_dir(temp.path())
        .arg("ls-tree")
        .arg(&tree_sha)
        .output()
        .unwrap();

    assert!(output.status.success());
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Should contain both files
    assert!(output_str.contains("file1.txt"));
    assert!(output_str.contains("file2.txt"));
    
    // Should have proper format: mode type hash<TAB>name
    assert!(output_str.contains("100644 blob"));
    
    Ok(())
}

#[test]
fn test_ls_tree_invalid_object() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();
    let _ = guts::core::repo::init(temp.path());

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("ls-tree")
        .arg("invalidhash123456789abcdef0123456789abcdef01234567")
        .assert()
        .failure()
        .stderr(predicates::str::contains("not a valid object name"));

    Ok(())
}

#[test]
fn test_ls_tree_not_a_tree() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("test content").unwrap();

    let _ = guts::core::repo::init(temp.path());

    // Create a blob object
    let mut cmd = Command::cargo_bin("guts").unwrap();
    let output = cmd.current_dir(temp.path())
        .arg("hash-object")
        .arg("test.txt")
        .output()
        .unwrap();

    assert!(output.status.success());
    let blob_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Try to ls-tree on a blob (should fail)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp.path())
        .arg("ls-tree")
        .arg(&blob_sha)
        .assert()
        .failure()
        .stderr(predicates::str::contains("not a tree object"));

    Ok(())
}
