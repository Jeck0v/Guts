/// a Git object that can be serialized and hashed
pub trait GitObject {
    fn object_type(&self) -> &str;
    fn content(&self) -> &[u8];

    /// Complete encoding of the Git object with header : ‘{type} {size}\0{content}’
    fn serialize(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.object_type(), self.content().len());
        let mut full = header.into_bytes();
        full.extend_from_slice(self.content());
        full
    }
}

pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: [u8, 20]
}

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl GitObject for Tree {
    fn serialize(&self) -> Vec<u8> {
        let mut content = Vec::new();

        for entry in &self.entries {
            content.extend(format!("{} {}\0", entry.mode, entry.name).as_bytes());
            content.extend(&entry.hash)
        }

        let header = format!("tree {}\0", content.len());
        let mut out = header.into_bytes();
        out.extend(content);
        out
    }

    fn object_type(&self) -> &'static str {
        "tree"
    }
}