use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

/// Pattern that can be negated (for !pattern support)
struct IgnorePattern {
    glob_set: GlobSet,
    is_negation: bool,
}

/// .gutsignore and .gitignore support
pub struct IgnoreMatcher {
    patterns: Vec<IgnorePattern>,
}

impl IgnoreMatcher {
    pub fn from_gutsignore(repo_root: &Path) -> std::io::Result<Self> {
        let guts_ignore_path = repo_root.join(".gutsignore");
        let git_ignore_path = repo_root.join(".gitignore");

        if !guts_ignore_path.exists() && !git_ignore_path.exists() {
            return Ok(Self::empty());
        }

        let ignore_path = if guts_ignore_path.exists() {
            guts_ignore_path
        } else {
            git_ignore_path
        };

        let file = File::open(ignore_path)?;
        let reader = BufReader::new(file);

        let mut patterns = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let (pattern, is_negation) = if trimmed.starts_with('!') {
                (&trimmed[1..], true)
            } else {
                (trimmed, false)
            };

            let mut builder = GlobSetBuilder::new();

            // Handle directory patterns (ending with /)
            if pattern.ends_with('/') {
                let dir_pattern = format!("{}**", pattern);
                let glob = Glob::new(&dir_pattern)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                builder.add(glob);
            } else {
                // Add the pattern as-is
                let glob = Glob::new(pattern)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                builder.add(glob);

                // Also add a directory version for patterns that might match directories
                if !pattern.contains('/') || !pattern.contains('*') {
                    let dir_pattern = format!("{}/", pattern);
                    if let Ok(dir_glob) = Glob::new(&dir_pattern) {
                        builder.add(dir_glob);
                    }
                }
            }

            let glob_set = builder
                .build()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            patterns.push(IgnorePattern {
                glob_set,
                is_negation,
            });
        }

        Ok(IgnoreMatcher { patterns })
    }

    pub fn is_ignored(&self, path: &Path, repo_root: &Path) -> bool {
        // Convert to relative path from repo root
        let relative_path = match path.strip_prefix(repo_root) {
            Ok(rel) => rel,
            Err(_) => path,
        };

        let mut ignored = false;

        // Process patterns in order
        for pattern in &self.patterns {
            if pattern.glob_set.is_match(relative_path) {
                if pattern.is_negation {
                    ignored = false;
                } else {
                    ignored = true;
                }
            }
        }

        ignored
    }

    pub fn empty() -> Self {
        IgnoreMatcher {
            patterns: Vec::new(),
        }
    }
}
