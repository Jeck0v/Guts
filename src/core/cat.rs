use crate::core::object::Commit;
use crate::core::object::TreeEntry;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Enum representing different parsed Git object types.
/// - Blob holds raw file content bytes.
/// - Tree holds a list of `TreeEntry` structs representing files/directories.
/// - Commit holds a parsed commit object with metadata.
/// - Other holds unknown object types with their raw bytes.
pub enum ParsedObject {
    Blob(Vec<u8>),
    Tree(Vec<TreeEntry>),
    Commit(Commit),
    Other(String, Vec<u8>),
}

/// Given the root `.git` directory and a SHA-1 hash string,
/// constructs and returns the path to the object file.
///
/// Git stores objects in subdirectories named by the first two
/// characters of their SHA, with the remainder as the filename:
/// `.git/objects/XX/YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY`
pub fn get_object_path(guts_dir: &Path, sha: &str) -> PathBuf {
    let (dir, file) = sha.split_at(2);
    guts_dir.join("objects").join(dir).join(file)
}

/// Parses raw Git object data into a structured `ParsedObject`.
///
/// Git object format:
///   "<type> <size>\0<content>"
///
/// Steps:
/// - Find the first null byte to split header and content.
/// - Parse header for object type and size (size is parsed but not verified).
/// - Based on type, parse the body:
///     - "tree": parse as list of TreeEntry structs
///     - "blob": raw bytes returned as-is
///     - "commit": parse as Commit struct
///     - others: return type and raw bytes unchanged
pub fn parse_object(data: &[u8]) -> Result<ParsedObject> {
    // Find the position of the null byte separating header from body
    let null_pos = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow!("invalid object format : missing null separator"))?;

    // Interpret header bytes as UTF-8 string
    let header = std::str::from_utf8(&data[..null_pos])?;
    // The remainder after the null byte is the body/content
    let body = &data[null_pos + 1..];

    // Header format: "<type> <size>"
    let mut parts = header.split(' ');
    let obj_type = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid header format"))?;
    let size_str = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid header format"))?;
    let _size: usize = size_str.parse()?; // Size parsed but not strictly enforced here

    // Dispatch parsing based on object type
    match obj_type {
        "tree" => {
            // Parse tree object body into entries
            let entries = parse_tree_body(body)?;
            Ok(ParsedObject::Tree(entries))
        }
        "blob" => {
            // Blob object: raw data as is
            Ok(ParsedObject::Blob(body.to_vec()))
        }
        "commit" => {
            // Commit object: parse structured commit metadata
            let commit = parse_commit_body(body)?;
            Ok(ParsedObject::Commit(commit))
        }
        _ => {
            // Unknown or unsupported object type: keep raw data and type
            Ok(ParsedObject::Other(obj_type.to_string(), body.to_vec()))
        }
    }
}

/// Parses the body bytes of a Git tree object into a vector of `TreeEntry`.
///
/// Tree entries format (raw bytes):
///   <mode> SPACE <filename> NULL <20-byte SHA1 hash>
/// Entries repeat until the entire body is parsed.
///
/// Returns a vector of parsed `TreeEntry` or an error if format is invalid.
pub fn parse_tree_body(data: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Parse mode string (e.g. "100644") up to the space character
        let mode_end = data[i..]
            .iter()
            .position(|&b| b == b' ')
            .ok_or_else(|| anyhow!("invalid tree entry: missing space after mode"))?;
        let mode = std::str::from_utf8(&data[i..i + mode_end])?.to_string();

        i += mode_end + 1; // Advance past mode and space

        // Parse filename string up to the null byte
        let name_end = data[i..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| anyhow!("invalid tree entry: missing null byte after filename"))?;
        let name = std::str::from_utf8(&data[i..i + name_end])?.to_string();

        i += name_end + 1; // Advance past filename and null byte

        // Next 20 bytes represent SHA1 hash of the referenced object
        if i + 20 > data.len() {
            return Err(anyhow!("invalid tree entry: incomplete SHA1 hash"));
        }
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&data[i..i + 20]);

        i += 20; // Advance past hash bytes

        // Add parsed entry to list
        entries.push(TreeEntry { mode, name, hash });
    }

    Ok(entries)
}

/// Parses the body bytes of a commit object into a `Commit` struct.
///
/// Commit body format is plaintext with lines:
///   tree <tree SHA>
///   parent <parent SHA>  (optional)
///   <empty line>
///   <commit message>
///
/// Returns the parsed commit or an error if mandatory fields are missing.
fn parse_commit_body(body: &[u8]) -> Result<Commit> {
    let text = std::str::from_utf8(body)?;
    let mut tree = String::new();
    let mut parent = None;
    let mut message = String::new();
    let mut in_message = false;

    for line in text.lines() {
        if line.trim().is_empty() {
            // Empty line marks start of commit message
            in_message = true;
            continue;
        }

        if in_message {
            // Accumulate commit message lines
            message.push_str(line);
            message.push('\n');
            continue;
        }

        // Parse tree and parent lines
        if let Some(rest) = line.strip_prefix("tree ") {
            tree = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("parent ") {
            parent = Some(rest.to_string());
        }
    }

    if tree.is_empty() {
        return Err(anyhow!("commit object missing 'tree' field"));
    }

    Ok(Commit {
        tree,
        parent,
        message: message.trim_end().to_string(),
    })
}
