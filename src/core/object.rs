use std::fs;
use std::io;

pub struct GitObject {
    pub obj_type: String,
    pub content: Vec<u8>,
}

impl GitObject {
    /// Lit un objet Git à partir de son OID
    pub fn read(git_dir: &str, oid: &str) -> io::Result<Self> {
        let (dir, file) = oid.split_at(2);
        let path = format!("{}/objects/{}/{}", git_dir, dir, file);
        let data = fs::read(&path)?;

        // Trouve le type et le contenu
        if let Some(null_pos) = data.iter().position(|&b| b == 0) {
            let header = &data[..null_pos]; // ex: b"blob 12"
            let content = data[null_pos + 1..].to_vec();
            let obj_type = String::from_utf8_lossy(header)
                .split(' ')
                .next()
                .unwrap_or("unknown")
                .to_string();
            Ok(GitObject { obj_type, content })
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Objet Git mal formé"))
        }
    }
}
