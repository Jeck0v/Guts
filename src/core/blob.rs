use std::fs;
use std::io::{self};
use sha1::{Sha1, Digest};
pub struct Blob {
    pub data: Vec<u8>,
}

impl Blob {
    /// Crée un blob à partir d'un fichier
    pub fn from_file(path: &str) -> io::Result<Self> {
        let data = fs::read(path)?;
        Ok(Blob { data })
    }

    /// Retourne le contenu formaté pour Git ("blob <taille>\0<contenu>")
    pub fn as_git_object(&self) -> Vec<u8> {
        let mut out = format!("blob {}\0", self.data.len()).into_bytes();
        out.extend(&self.data);
        out
    }

    /// Calcule le hash SHA-1 du blob (OID)
    pub fn oid(&self) -> String {
        let data = self.as_git_object();
        let hash = Sha1::digest(&data);
        format!("{:x}", hash)
    }

    /// Écrit le blob dans .git/objects/ et retourne l'OID
    pub fn write(&self, git_dir: &str) -> io::Result<String> {
        let oid = self.oid();
        let dir = format!("{}/objects/{}", git_dir, &oid[..2]);
        let file = format!("{}/{}", dir, &oid[2..]);
        fs::create_dir_all(&dir)?;
        fs::write(&file, self.as_git_object())?;
        Ok(oid)
    }
}
