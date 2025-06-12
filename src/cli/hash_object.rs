use anyhow::Result;
use crate::internals::object;
pub fn run_hash_object_command(path: &str) -> Result<()> {
    let sha = object::hash_blob_from_path(path)?;
    println!("{}", sha);
    Ok(())
}
