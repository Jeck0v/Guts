use sha1::Sha1;
/// Data blob Git & header ("blob <size>\0")
pub fn prepare_blob(data: &[u8]) -> Vec<u8> {
    let mut blob = Vec::new();
    let header = format!("blob {}\0", data.len());
    blob.extend_from_slice(header.as_bytes());
    blob.extend_from_slice(data);
    blob
}
/// hash SHA-1 hexadécimal
pub fn sha1_hex(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let digest = hasher.digest().bytes();
    hex::encode(digest)
}
