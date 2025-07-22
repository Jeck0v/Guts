use assert_cmd::Command;
use assert_fs::prelude::*;

/// Test diagnostic pour comprendre ce qui se passe avec .gutsignore
#[test]
fn debug_gutsignore_integration() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    println!("🗂️  Test directory: {}", temp_dir.path().display());

    // Initialize repository
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("init");
    let output = cmd.output().unwrap();
    println!("Init output: {}", String::from_utf8_lossy(&output.stdout));
    assert!(output.status.success());

    // Create .gutsignore
    let gutsignore = temp_dir.child(".gutsignore");
    gutsignore.write_str("*.log\n").unwrap();
    println!("✅ Created .gutsignore with: *.log");

    // Create test files
    let readme = temp_dir.child("README.md");
    readme.write_str("# Test").unwrap();

    let log_file = temp_dir.child("debug.log");
    log_file.write_str("log content").unwrap();

    println!("✅ Created files: README.md, debug.log");

    // Try to add all files
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("add").arg(".");
    let output = cmd.output().unwrap();
    println!("Add output: {}", String::from_utf8_lossy(&output.stdout));
    println!("Add stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Check status and print everything
    let mut cmd = Command::cargo_bin("guts").unwrap();
    cmd.current_dir(temp_dir.path()).arg("status");
    let output = cmd.output().unwrap();
    println!("Status output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("Status stderr:");
    println!("{}", String::from_utf8_lossy(&output.stderr));

    // List all files in directory for debugging
    println!("\nFiles in directory:");
    if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  - {}", entry.file_name().to_string_lossy());
            }
        }
    }
}