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

    let guts_dir = path.join(".guts");

    assert!(guts_dir.exists());
    assert!(guts_dir.join("objects").is_dir());
    assert!(guts_dir.join("refs/heads").is_dir());
    assert_eq!(
        fs::read_to_string(guts_dir.join("HEAD")).unwrap(),
        "ref: refs/heads/master\n"
    );
    assert!(guts_dir.join("config").exists());
}
