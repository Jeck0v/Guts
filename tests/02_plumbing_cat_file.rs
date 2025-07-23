use assert_cmd::Command;
use assert_fs::prelude::*;
use std::process::Command as StdCommand;

/// Test que guts cat-file produit exactement la même sortie que git cat-file
#[test]
fn test_cat_file_compatibility_with_git() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Hello, Git compatibility test!\nSecond line.\n")
        .unwrap();

    // Init git repo and create object
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();
    let hash_output = StdCommand::new("git")
        .current_dir(temp.path())
        .args(["hash-object", "-w", "test.txt"])
        .output()
        .expect("Failed to create git object");

    let hash = String::from_utf8_lossy(&hash_output.stdout)
        .trim()
        .to_string();

    // === Git réel ===
    let git_output = StdCommand::new("git")
        .current_dir(temp.path())
        .args(["cat-file", "-p", &hash])
        .output()
        .expect("Failed to run git cat-file");

    let git_content = String::from_utf8_lossy(&git_output.stdout).to_string();

    // === Guts ===
    let mut cmd = Command::cargo_bin("guts").unwrap();
    let guts_output = cmd
        .current_dir(temp.path())
        .arg("cat-file")
        .arg(&hash)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let guts_content = String::from_utf8_lossy(&guts_output).to_string();

    assert_eq!(
        git_content, guts_content,
        "Guts cat-file must produce identical output to Git"
    );
}
