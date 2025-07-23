use assert_cmd::Command;
use assert_fs::prelude::*;
use std::process::Command as StdCommand;

// Test que guts hash-object produit exactement le même hash que git hash-object
#[test]
fn test_hash_object_compatibility_with_git() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    file.write_str("Hello, Git compatibility test!\nSecond line.\n")
        .unwrap();

    // === Git réel ===
    let git_output = StdCommand::new("git")
        .current_dir(temp.path())
        .args(["hash-object", "test.txt"])
        .output()
        .expect("Failed to run git hash-object - Git must be installed");

    assert!(
        git_output.status.success(),
        "Git hash-object failed: {}",
        String::from_utf8_lossy(&git_output.stderr)
    );

    let git_hash = String::from_utf8_lossy(&git_output.stdout)
        .trim()
        .to_string();

    // === Guts ===
    let _ = guts::core::repo::init(temp.path());

    let mut cmd = Command::cargo_bin("guts").unwrap();
    let guts_output = cmd
        .current_dir(temp.path())
        .arg("hash-object")
        .arg("test.txt")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let guts_hash = String::from_utf8_lossy(&guts_output).trim().to_string();

    println!("📊 Hash comparison:");
    println!("   Git:  {}", git_hash);
    println!("   Guts: {}", guts_hash);

    // Verify that the blob file was correctly written in .git/objects/
    let (dir, file_name) = guts_hash.split_at(2);
    let object_path = temp.path().join(".git/objects").join(dir).join(file_name);
    assert!(
        object_path.exists(),
        "Blob object should be stored in .git/objects/"
    );

    assert_eq!(
        git_hash, guts_hash,
        "Guts hash-object must produce identical hash to Git"
    );
}
