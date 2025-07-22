use assert_cmd::Command;
use assert_fs::prelude::*;
use std::process::Command as StdCommand;

/// Test que guts write-tree produit exactement le même hash que git write-tree
#[test]
fn test_write_tree_compatibility_with_git() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Créer structure avec hiérarchie pour tester la correction
    temp.child("README.md")
        .write_str("# Test Project\nCompatibility test.\n")
        .unwrap();
    temp.child("src").create_dir_all().unwrap();
    temp.child("src/main.rs")
        .write_str("fn main() {\n    println!(\"Hello!\");\n}\n")
        .unwrap();
    temp.child("src/utils").create_dir_all().unwrap();
    temp.child("src/utils/helper.rs")
        .write_str("pub fn help() {}\n")
        .unwrap();

    // === Git réel ===
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["add", "."])
        .output()
        .unwrap();
    let git_output = StdCommand::new("git")
        .current_dir(temp.path())
        .args(["write-tree"])
        .output()
        .expect("Git must be installed");
    let git_hash = String::from_utf8_lossy(&git_output.stdout)
        .trim()
        .to_string();

    // === Guts ===
    std::fs::remove_dir_all(temp.path().join(".git")).unwrap();
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("add")
        .arg(".")
        .assert()
        .success();
    let guts_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .arg("write-tree")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let guts_hash = String::from_utf8_lossy(&guts_output).trim().to_string();

    println!("📊 Write-tree comparison:");
    println!("   Git:  {}", git_hash);
    println!("   Guts: {}", guts_hash);

    assert_eq!(
        git_hash, guts_hash,
        "Guts write-tree must produce identical tree hash to Git"
    );
}
