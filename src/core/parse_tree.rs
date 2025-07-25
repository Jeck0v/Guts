use anyhow::{Result, anyhow};

#[derive(Clone, Debug)]
pub struct TreeEntry {
    pub mode: String,
    pub filename: String,
    pub sha: String, // SHA-1 hex string
}

pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // 1. Lire mode ASCII jusqu'à espace
        let mode_start = i;
        while i < data.len() && data[i] != b' ' {
            i += 1;
        }
        if i == data.len() {
            return Err(anyhow!("Malformed tree: missing space after mode"));
        }
        let mode = std::str::from_utf8(&data[mode_start..i])?.to_string();
        i += 1; // skip space

        // 2. Lire nom fichier jusqu'à NULL
        let filename_start = i;
        while i < data.len() && data[i] != 0 {
            i += 1;
        }
        if i == data.len() {
            return Err(anyhow!("Malformed tree: missing null after filename"));
        }
        let filename = std::str::from_utf8(&data[filename_start..i])?.to_string();
        i += 1; // skip null byte

        // 3. Lire 20 bytes SHA binaire
        if i + 20 > data.len() {
            return Err(anyhow!("Malformed tree: truncated SHA"));
        }
        let sha_bin = &data[i..i+20];
        i += 20;

        // 4. Convertir SHA binaire en hexadécimal
        let sha = sha_bin.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        entries.push(TreeEntry { mode, filename, sha });
    }

    Ok(entries)
}
