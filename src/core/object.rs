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
