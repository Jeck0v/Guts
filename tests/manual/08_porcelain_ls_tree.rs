use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

/// Manual test for ls-tree command
#[test]
fn test_ls_tree_manual() {
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

    // Create test files with different content
    let file1 = temp_dir.child("apple.txt");
    file1.write_str("Apple content").unwrap();
    
    let file2 = temp_dir.child("banana.txt");
    file2.write_str("Banana content").unwrap();

    // Create subdirectory with files
    let subdir = temp_dir.child("fruits");
    subdir.create_dir_all().unwrap();
    let file3 = subdir.child("cherry.txt");
    file3.write_str("Cherry content").unwrap();
    
    let file4 = subdir.child("date.txt");
    file4.write_str("Date content").unwrap();
    
    println!("📁 Created test files: apple.txt, banana.txt, fruits/cherry.txt, fruits/date.txt");

    // Add all files to index
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg(".");
    cmd.assert().success();
    println!("✅ Files added to index");

    // Commit to create tree objects
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commit")
        .arg("-m")
        .arg("Add test files");
    cmd.assert().success();
    println!("✅ Files committed");

    // Get commit hash
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("rev-parse").arg("HEAD");
    let output = cmd.output().unwrap();
    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("📝 Commit hash: {}", commit_hash);

    // Get tree hash from commit
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("cat-file").arg(&commit_hash);
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
    println!("🌳 Root tree hash: {}", tree_hash);

    // Test ls-tree on root tree
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-tree").arg(tree_hash);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify output format and content
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    println!("📄 ls-tree output:\n{}", stdout);
    
    // Should contain 3 entries: apple.txt, banana.txt, and fruits/ directory
    assert_eq!(lines.len(), 3, "Root tree should contain 3 entries");
    
    // Check file entries format: "100644 blob <hash>\t<filename>"
    let apple_line = lines.iter().find(|&&line| line.ends_with("apple.txt")).unwrap();
    let banana_line = lines.iter().find(|&&line| line.ends_with("banana.txt")).unwrap();
    let fruits_line = lines.iter().find(|&&line| line.ends_with("fruits")).unwrap();
    
    assert!(apple_line.starts_with("100644 blob "), "apple.txt should be a blob with mode 100644");
    assert!(banana_line.starts_with("100644 blob "), "banana.txt should be a blob with mode 100644");
    assert!(fruits_line.starts_with("040000 tree "), "fruits should be a tree with mode 040000");
    println!("✅ File modes and types are correct");

    // Verify entries are sorted alphabetically
    let mut sorted_lines = lines.clone();
    sorted_lines.sort();
    assert_eq!(lines, sorted_lines, "Tree entries should be sorted alphabetically");
    println!("✅ Tree entries are sorted alphabetically");

    // Extract subtree hash for fruits directory
    let fruits_hash = fruits_line
        .split_whitespace()
        .nth(2)
        .unwrap();
    println!("🌿 Fruits subtree hash: {}", fruits_hash);

    // Test ls-tree on subtree
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-tree").arg(fruits_hash);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let sublines: Vec<&str> = stdout.trim().split('\n').collect();
    println!("📄 ls-tree fruits subtree output:\n{}", stdout);
    
    // Should contain 2 entries: cherry.txt and date.txt
    assert_eq!(sublines.len(), 2, "Fruits subtree should contain 2 entries");
    
    let cherry_line = sublines.iter().find(|&&line| line.ends_with("cherry.txt")).unwrap();
    let date_line = sublines.iter().find(|&&line| line.ends_with("date.txt")).unwrap();
    
    assert!(cherry_line.starts_with("100644 blob "), "cherry.txt should be a blob with mode 100644");
    assert!(date_line.starts_with("100644 blob "), "date.txt should be a blob with mode 100644");
    println!("✅ Subtree entries are correct");

    // Verify subtree entries are also sorted
    let mut sorted_sublines = sublines.clone();
    sorted_sublines.sort();
    assert_eq!(sublines, sorted_sublines, "Subtree entries should be sorted alphabetically");
    println!("✅ Subtree entries are sorted alphabetically");

    // Test ls-tree with non-existent hash should fail
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-tree").arg("0000000000000000000000000000000000000000");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Object not found"));
    println!("✅ Non-existent tree hash correctly fails");

    // Test ls-tree with blob hash should fail
    let apple_blob_hash = apple_line
        .split_whitespace()
        .nth(2)
        .unwrap();
    
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("ls-tree").arg(apple_blob_hash);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Object is not a tree"));
    println!("✅ Using blob hash with ls-tree correctly fails");

    println!("🎉 ls-tree manual test completed successfully!");
}