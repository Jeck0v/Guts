use std::fs;
use std::path::{Path, PathBuf};

pub fn build_flat_tree_from_current_dir() -> Vec<String> {
    fn rec(dir: &Path, base: &Path, out: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let rel = p.strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .to_string();
                out.push(rel.clone());
                if p.is_dir() {
                    rec(&p, base, out);
                }
            }
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let mut all = Vec::new();
    rec(&cwd, &cwd, &mut all);
    all
}
