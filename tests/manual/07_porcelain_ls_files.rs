use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Manual test for ls-files command
#[test]
fn test_ls_files_manual() {
    // Create temporary directory
    let temp_dir = assert_fs::TempDir::new().unwrap();
    println!("🗂️  Created test directory: {}", temp_dir.path().display());

    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialized empty Guts repository"));
    println!("✅ Repository initialized");

    // Test ls-files on empty repository (should return empty)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "ls-files should be empty in new repository");
    println!("✅ Empty repository shows no files");

    // Create some files
    let file1 = temp_dir.child("file1.txt");
    file1.write_str("Content of file1").unwrap();
    
    let file2 = temp_dir.child("file2.txt");
    file2.write_str("Content of file2").unwrap();

    let subdir = temp_dir.child("subdir");
    subdir.create_dir_all().unwrap();
    let file3 = subdir.child("file3.txt");
    file3.write_str("Content of file3").unwrap();
    
    println!("📁 Created test files: file1.txt, file2.txt, subdir/file3.txt");

    // Test ls-files before adding (should still be empty)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "ls-files should be empty before adding files");
    println!("✅ Files not yet added don't appear in ls-files");

    // Add files to index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("file1.txt");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("file2.txt");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg("subdir/file3.txt");
    cmd.assert().success();
    println!("✅ Files added to index");

    // Test ls-files after adding
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify all files are listed
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
    assert!(stdout.contains("subdir/file3.txt"));
    println!("✅ All staged files shown in ls-files output");

    // Verify files are sorted alphabetically
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    assert_eq!(lines.len(), 3);
    
    // Check alphabetical order
    let mut sorted_lines = lines.clone();
    sorted_lines.sort();
    assert_eq!(lines, sorted_lines, "Files should be sorted alphabetically");
    println!("✅ Files are sorted alphabetically");

    // Commit the files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Add test files");
    cmd.assert().success();
    println!("✅ Files committed");

    // Test ls-files after commit (should still show the same files)
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout_after_commit = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), stdout_after_commit.trim(), 
               "ls-files should show same files after commit");
    println!("✅ ls-files shows same files after commit");

    // Create another file but don't add it
    let file4 = temp_dir.child("file4.txt");
    file4.write_str("Content of file4").unwrap();

    // Test that untracked file doesn't appear in ls-files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("file4.txt"), "Untracked files should not appear in ls-files");
    println!("✅ Untracked files don't appear in ls-files");

    // Modify an existing tracked file
    file1.write_str("Modified content of file1").unwrap();

    // Test that modified tracked file still appears in ls-files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-files");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file1.txt"), "Modified tracked files should still appear in ls-files");
    println!("✅ Modified tracked files still appear in ls-files");

    println!("🎉 ls-files manual test completed successfully!");
}