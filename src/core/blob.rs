use crate::core::object::GitObject;

pub struct Blob {
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl GitObject for Blob {
    fn object_type(&self) -> &str {
        "blob"
    }

    fn content(&self) -> Vec<u8> {
        self.data.clone()
    }
}
