use std::fs;
use std::path::Path;

use assert_fs::prelude::*;
use assert_fs::TempDir;
use guts::core::repo;

#[test]
fn test_init_creates_repository_structure() {
    let temp = TempDir::new().unwrap();
    let path = temp.path();

    repo::init(path).unwrap();

    let git_dir = path.join(".git");

    assert!(git_dir.exists());
    assert!(git_dir.join("objects").is_dir());
    assert!(git_dir.join("refs/heads").is_dir());
    assert_eq!(
        fs::read_to_string(git_dir.join("HEAD")).unwrap(),
        "ref: refs/heads/main\n"
    );
    assert!(git_dir.join("config").exists());
}
