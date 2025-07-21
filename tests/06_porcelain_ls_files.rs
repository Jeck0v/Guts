use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_ls_files_empty_index() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    
    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();
    
    // Test ls-files on empty index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_ls_files_single_file() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    
    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();
    
    // Create a test file
    let test_file = temp_dir.child("test.txt");
    test_file.write_str("Hello, world!").unwrap();
    
    // Add file to index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("add")
        .arg("test.txt");
    cmd.assert().success();
    
    // Test ls-files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_ls_files_multiple_files() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    
    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();
    
    // Create multiple test files
    let file1 = temp_dir.child("file1.txt");
    file1.write_str("Content 1").unwrap();
    
    let file2 = temp_dir.child("file2.txt");
    file2.write_str("Content 2").unwrap();
    
    let file3 = temp_dir.child("another.txt");
    file3.write_str("Content 3").unwrap();
    
    // Add files to index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("add")
        .arg("file1.txt")
        .arg("file2.txt")
        .arg("another.txt");
    cmd.assert().success();
    
    // Test ls-files - should be sorted alphabetically
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("another.txt"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
    
    // Verify the order is alphabetical
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "another.txt");
    assert_eq!(lines[1], "file1.txt");
    assert_eq!(lines[2], "file2.txt");
}

#[test]
fn test_ls_files_with_subdirectory() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    
    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();
    
    // Create files in root and subdirectory
    let root_file = temp_dir.child("root.txt");
    root_file.write_str("Root content").unwrap();
    
    let subdir = temp_dir.child("subdir");
    subdir.create_dir_all().unwrap();
    
    let sub_file = subdir.child("sub.txt");
    sub_file.write_str("Sub content").unwrap();
    
    // Add files to index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("add")
        .arg("root.txt")
        .arg("subdir");
    cmd.assert().success();
    
    // Test ls-files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("root.txt"))
        .stdout(predicate::str::contains("subdir"));
}

#[test]
fn test_ls_files_after_remove() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    
    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert().success();
    
    // Create test files
    let file1 = temp_dir.child("keep.txt");
    file1.write_str("Keep this").unwrap();
    
    let file2 = temp_dir.child("remove.txt");
    file2.write_str("Remove this").unwrap();
    
    // Add files to index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("add")
        .arg("keep.txt")
        .arg("remove.txt");
    cmd.assert().success();
    
    // Verify both files are in index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("keep.txt"))
        .stdout(predicate::str::contains("remove.txt"));
    
    // Remove one file from index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("rm")
        .arg("remove.txt");
    cmd.assert().success();
    
    // Verify only the kept file remains in index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("keep.txt"))
        .stdout(predicate::str::contains("remove.txt").not());
}

#[test]
fn test_ls_files_help() {
    // Test that help works
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.arg("ls-files").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all files in the index"))
        .stdout(predicate::str::contains("Usage: guts"));
}
