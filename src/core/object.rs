
/// Trait representing a Git object that can be serialized and hashed.
/// Any Git object (blob, tree, commit, etc.) should implement this trait.
pub trait GitObject {
    /// Returns the type of the Git object as a string slice.
    /// For example: "blob", "tree", "commit", etc.
    fn object_type(&self) -> &str;

    /// Returns the raw content bytes of the object without the Git header.
    /// This content is what gets stored and hashed by Git.
    fn content(&self) -> Vec<u8>;

    /// Serializes the entire Git object into a byte vector,
    /// including the Git header in the format: "{type} {size}\0{content}".
    ///
    /// The header contains the object type and content length,
    /// followed by a null byte (`\0`), then the actual content bytes.
    fn serialize(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.object_type(), self.content().len());
        let mut full = header.into_bytes(); // Convert header string to bytes
        full.extend_from_slice(&self.content()); // Append the content bytes after the header
        full
    }
}

/// Represents a single entry in a Git tree object.
/// Each entry corresponds to a file or a directory in the tree.
pub struct TreeEntry {
    pub mode: String,   // File mode as a string, e.g. "100644" for normal files
    pub name: String,   // File or directory name
    pub hash: [u8; 20], // SHA-1 hash of the object the entry points to (20 bytes)
}

/// Represents a Git tree object, which contains multiple tree entries.
/// This corresponds to a directory in Git's internal structure.
pub struct Tree {
    pub entries: Vec<TreeEntry>, // List of entries (files or subdirectories)
}

impl GitObject for Tree {
    /// Serializes the entire tree object including header and entries.
    fn serialize(&self) -> Vec<u8> {
        let content = self.content(); // Get the raw content bytes for the tree entries

        let header = format!("tree {}\0", content.len()); // Create tree header with content length
        let mut out = header.into_bytes(); // Convert header to bytes
        out.extend(content); // Append the tree entries content
        out
    }

    /// Returns the object type string, which is always "tree" for this struct.
    fn object_type(&self) -> &'static str {
        "tree"
    }

    /// Constructs the raw content bytes for the tree entries,
    /// following Git's internal format for tree objects.
    ///
    /// Each entry is serialized as:
    /// "{mode} {name}\0{hash}"
    /// where mode and name are strings,
    /// followed by a null byte, then the 20-byte SHA-1 hash.
    fn content(&self) -> Vec<u8> {
        let mut content = Vec::new();

        for entry in &self.entries {
            // Add "{mode} {name}\0" as bytes
            content.extend(format!("{} {}\0", entry.mode, entry.name).as_bytes());
            // Add the 20-byte hash bytes directly
            content.extend(&entry.hash);
        }

        content
    }
}

pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub message: String,
    pub author: String,
    pub committer: String,
    pub author_date: i64,
    pub committer_date: i64,
}

impl GitObject for Commit {
    fn object_type(&self) -> &str {
        "commit"
    }

    fn content(&self) -> Vec<u8> {
        let mut content = Vec::new();

        content.extend(format!("tree {}\n", self.tree).as_bytes());

        if let Some(ref p) = self.parent {
            content.extend(format!("parent {}\n", p).as_bytes());
        }

        let timezone = "+0000";

        let author_line = format!(
            "author {} {} {}\n",
            self.author, self.author_date, timezone
        );
        let committer_line = format!(
            "committer {} {} {}\n",
            self.committer, self.committer_date, timezone
        );

        content.extend(author_line.as_bytes());
        content.extend(committer_line.as_bytes());
        content.extend(b"\n");

        content.extend(self.message.as_bytes());
        if !self.message.ends_with('\n') {
            content.extend(b"\n");
        }

        content
    }
}
