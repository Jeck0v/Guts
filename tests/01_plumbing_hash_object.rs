use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn test_hash_object_matches_git() {
    // Prepare a temporary directory with a test file
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("hello.txt");
    file.write_str("Hello, world!\n").unwrap();

    // Initialize repositories  
    Command::cargo_bin("guts").unwrap().current_dir(temp.path()).args(&["init"]).assert().success();
    Command::new("git").current_dir(temp.path()).args(&["init", "--quiet"]).assert().success();

    // Get hash from official Git
    let git_output = Command::new("git")
        .current_dir(temp.path())
        .args(&["hash-object", "hello.txt"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let git_hash = String::from_utf8_lossy(&git_output).trim().to_string();

    // Get hash from our implementation
    let guts_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .args(&["hash-object", "hello.txt"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let guts_hash = String::from_utf8_lossy(&guts_output).trim().to_string();

    // Compare the hashes
    assert_eq!(git_hash, guts_hash, "Hash from guts should match Git's hash");
}
