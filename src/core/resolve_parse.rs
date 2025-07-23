use anyhow::Result;
use std::path::Path;
use std::fs;

pub fn resolve_ref(guts_dir: &Path, head_input: &str) -> Result<String> {
    if head_input == "HEAD" {
        let head_path = guts_dir.join("HEAD");
        let content = fs::read_to_string(&head_path)?.trim().to_string();

        if content.starts_with("ref: ") {
            let ref_name = content.trim_start_matches("ref: ").trim();
            return resolve_ref(guts_dir, ref_name);
        } else if content.len() == 40 && content.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(content);
        } else {
            anyhow::bail!("Invalid HEAD content: {}", content);
        }
    }

    if head_input.len() == 40 && head_input.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(head_input.to_string());
    }

    let paths_to_try = [
        guts_dir.join("refs").join("heads").join(head_input),
        guts_dir.join("refs").join("tags").join(head_input),
        guts_dir.join(head_input),
    ];

    for path in paths_to_try {
        if path.exists() {
            let sha = fs::read_to_string(path)?.trim().to_string();
            return Ok(sha);
        }
    }

    anyhow::bail!("Reference '{}' not found", head_input)
}
