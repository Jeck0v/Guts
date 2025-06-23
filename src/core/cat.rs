use std::{path::{Path,PathBuf}};

use anyhow::{anyhow, Result};

/// Constructs the path to the stored object based on SHA
///
/// Splits SHA into:
/// - first 2 chars as directory
/// - rest as filename
pub fn get_object_path(guts_dir: &Path, sha: &str) -> PathBuf {
    let (dir, file) = sha.split_at(2);
    guts_dir.join("objects").join(dir).join(file)
}

/// Parses the raw git object bytes into (header, body)
///
/// Git objects are stored as `<type> <size>\0<content>`
/// This function splits on the null byte to separate header and body
pub fn parse_object(data: &[u8]) -> Result<(String, String)> {
    if let Some(null_pos) = data.iter().position(|&b| b == 0) {
        let header = String::from_utf8_lossy(&data[..null_pos]).to_string();
        let body = String::from_utf8_lossy(&data[null_pos + 1..]).to_string();
        Ok((header, body))
    } else {
        Err(anyhow!("invalid object format: no null byte found"))
    }
}