use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn test_cat_file_round_trip() {
    // Setup temp directory with test file
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("test.txt");
    let original_content = "Hello, Git cat-file!\n";
    file.write_str(original_content).unwrap();

    // Initialize repository using porcelain command
    Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .args(&["init"])
        .assert()
        .success();

    // Create object with guts hash-object
    let hash_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .args(&["hash-object", "test.txt"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hash = String::from_utf8_lossy(&hash_output).trim().to_string();

    // Read it back with guts cat-file
    let cat_output = Command::cargo_bin("guts")
        .unwrap()
        .current_dir(temp.path())
        .args(&["cat-file", &hash])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let retrieved_content = String::from_utf8_lossy(&cat_output);

    // Should get back the original content
    assert_eq!(original_content.trim(), retrieved_content.trim(), "Cat-file should return original content");
}