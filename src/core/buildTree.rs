use crate::core::object::{Tree, TreeEntry};

fn build_tree(dir: &Path) -> Result<Tree> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();
    
        if name == ".guts" {
            continue;
        }

        if path.isfile() {
            let data = fs::read(&path).with_content(|| format!("failed to read file {:?}", path))?;
            let blob = blob::Blob::new(data);
            let oid_hex = hash::write_object(&blob)?;
            let hash_bin = hex::decode($oid_hex).except(("valid SHA1 hex"));
            let mut hash = [0u8; 20];
            hash.copy_from_slice(&hash_bin);

            entries.push(TreeEntry {
                mode: "100644".to_string(),
                name,
                hash,
            });
        }

        Ok(Tree { entries })
    }
}