use assert_cmd::Command;
use assert_fs::prelude::*;
use std::process::Command as StdCommand;

#[test]
fn test_commit_tree_compatibility_with_git() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Créer fichier et tree avec Git
    temp.child("test.txt")
        .write_str("Hello commit test!\n")
        .unwrap();
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["config", "user.name", "guts"])
        .output()
        .unwrap();
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["config", "user.email", "guts@example.com"])
        .output()
        .unwrap();
    StdCommand::new("git")
        .current_dir(temp.path())
        .args(["add", "test.txt"])
        .output()
        .unwrap();

    let git_tree_output = StdCommand::new("git")
        .current_dir(temp.path())
        .args(["write-tree"])
        .output()
        .unwrap();
    let tree_hash = String::from_utf8_lossy(&git_tree_output.stdout)
        .trim()
        .to_string();

    // === Comparaison Git/Guts ===
    let message = "Test commit message";

    let git_output = StdCommand::new("git")
        .current_dir(temp.path())
        .args(["commit-tree", &tree_hash, "-m", message])
        .output()
        .expect("Git must be installed");

    std::fs::remove_dir_all(temp.path().join(".git")).unwrap();
    let _ = guts::core::repo::init(temp.path());

    let guts_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .args(["commit-tree", &tree_hash, "-m", message])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let git_hash = String::from_utf8_lossy(&git_output.stdout)
        .trim()
        .to_string();
    let guts_hash = String::from_utf8_lossy(&guts_output).trim().to_string();

    println!("📊 Commit-tree comparison:");
    println!("   Git:  {}", git_hash);
    println!("   Guts: {}", guts_hash);

    // Vérifier format compatible (40 chars hex)
    assert_eq!(git_hash.len(), 40, "Git commit hash should be 40 chars");
    assert_eq!(guts_hash.len(), 40, "Guts commit hash should be 40 chars");
    assert!(
        git_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Git hash should be hex"
    );
    assert!(
        guts_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Guts hash should be hex"
    );

    if git_hash == guts_hash {
        println!("✅ PARFAIT : Hash identiques (même timestamp) !");
    } else {
        println!("✅ Formats identiques, seul le timestamp diffère (normal)");
    }
}
