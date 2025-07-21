use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path};

use globset::{Glob, GlobSet, GlobSetBuilder};

/// .gutsignore
pub struct IgnoreMatcher {
    matcher: GlobSet,
}

impl IgnoreMatcher {
    pub fn from_gutsignore(repo_root: &Path) -> std::io::Result<Self> {
        let ignore_path = repo_root.join(".gutsignore");

        if !ignore_path.exists() {
            return Ok(Self::empty());
        }

        let file = File::open(ignore_path)?;
        let reader = BufReader::new(file);

        let mut builder = GlobSetBuilder::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let glob = Glob::new(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            builder.add(glob);
        }

        let matcher = builder
            .build()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(IgnoreMatcher { matcher })
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        self.matcher.is_match(path)
    }

    pub fn empty() -> Self {
        IgnoreMatcher {
            matcher: GlobSetBuilder::new().build().unwrap(),
        }
    }
}
