use std::{path::{Path, PathBuf}};
use anyhow::{anyhow, Result};
use crate::core::object::TreeEntry;
use crate::core::object::Commit;

/// Enum representing parsed Git objects.
/// - Blob holds raw file data.
/// - Tree holds a list of TreeEntry (files/directories with mode, name, hash).
/// - Other represents any other Git object type with its raw bytes.
pub enum ParsedObject {
    Blob(Vec<u8>),
    Tree(Vec<TreeEntry>),
    Commit(Commit),
    Other(String, Vec<u8>)
}

/// Given the root `.guts` directory and a SHA-1 hash string,
/// returns the file path where the object is stored.
///
/// Git stores objects as:
///   .guts/objects/XX/YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY
/// where XX = first two hex chars of SHA, and YYYYYY... = rest
pub fn get_object_path(guts_dir: &Path, sha: &str) -> PathBuf {
    let (dir, file) = sha.split_at(2);
    guts_dir.join("objects").join(dir).join(file)
}

/// Parses raw Git object data into a structured `ParsedObject`.
///
/// Git object format:
///   <type> <size>\0<content>
/// 
/// This function:
/// - Finds the first null byte to separate header and body
/// - Parses header into object type and size (size checked but not strictly used)
/// - Based on object type, it further parses the body:
///     - For "tree", it parses entries into `TreeEntry` structs
///     - For "blob", it returns raw bytes
///     - For others, returns type + raw bytes
pub fn parse_object(data: &[u8]) -> Result<ParsedObject> {
    // Find null byte separator between header and body
    let null_pos = data.iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow!("invalid object format : not null"))?;

    // Parse header as UTF-8 string slice
    let header = std::str::from_utf8(&data[..null_pos])?;
    let body = &data[null_pos + 1..];

    // Header format: "<type> <size>"
    let mut parts = header.split(' ');
    let obj_type = parts.next().ok_or_else(|| anyhow!("Invalid header format"))?;
    let size_str = parts.next().ok_or_else(|| anyhow!("Invalid header format"))?;
    let _size: usize = size_str.parse()?; // We parse but do not verify body length here

    match obj_type {
        "tree" => {
            // Parse tree entries from body bytes
            let entries = parse_tree_body(body)?;
            Ok(ParsedObject::Tree(entries))
        }
        "blob" => {
            // Just return the blob content bytes
            Ok(ParsedObject::Blob(body.to_vec()))
        }
        "commit" => {
            let commit = parse_commit_body(body)?;
            // Just return the blob content bytes
            Ok(ParsedObject::Commit(commit))
        }
        _ => {
            // For any other Git object type, return raw data with type
            Ok(ParsedObject::Other(obj_type.to_string(), body.to_vec()))
        }
    }
}

/// Parses the body bytes of a Git tree object into a list of TreeEntry.
///
/// Tree entries format in raw bytes:
///   <mode> SP <filename> NULL <20-byte SHA1 hash>
/// Repeated until all entries are parsed.
///
/// Returns a Vec of TreeEntry structs or an error.
pub fn parse_tree_body(data: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Parse the file mode string until space (e.g. "100644")
        let mode_end = data[i..].iter()
            .position(|&b| b == b' ')
            .ok_or_else(|| anyhow!("invalid tree entry: no space after mode"))?;
        let mode = std::str::from_utf8(&data[i..i + mode_end])?.to_string();

        i += mode_end + 1; // Move past mode + space

        // Parse the file name string until NULL byte
        let name_end = data[i..].iter()
            .position(|&b| b == 0)
            .ok_or_else(|| anyhow!("invalid tree entry: no null byte after filename"))?;
        let name = std::str::from_utf8(&data[i..i + name_end])?.to_string();

        i += name_end + 1; // Move past filename + null byte

        // Next 20 bytes are SHA1 hash of the entry object (blob or subtree)
        if i + 20 > data.len() {
            return Err(anyhow!("invalid tree entry: incomplete SHA1 hash"));
        }
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&data[i..i + 20]);

        i += 20; // Move past hash bytes

        // Collect the parsed entry
        entries.push(TreeEntry { mode, name, hash });
    }

    Ok(entries)
}


fn parse_commit_body(body: &[u8]) -> Result<Commit> {
    let text = std::str::from_utf8(body)?;
    let mut tree = String::new();
    let mut parent = None;
    let mut message = String::new();
    let mut in_message = false;

    for line in text.lines() {
        if line.trim().is_empty() {
            in_message = true;
            continue;
        }

        if in_message {
            message.push_str(line);
            message.push('\n');
            continue;
        }

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

