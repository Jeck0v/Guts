use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::core::{hash, build_tree};

/// Struct representing the command line arguments for the `write-tree` command.
/// The user can optionally specify a directory (`dir`) where the tree will be built.
/// If not specified, the current directory will be used.
#[derive(Args)]
pub struct WriteTreeArgs {
    pub dir: Option<PathBuf>,
}

/// The main function that runs the `write-tree` command.
/// It takes the parsed arguments, builds a tree object from the specified directory,
/// writes the tree object to the git object store,
/// and then prints the SHA-1 object ID of the tree.
///
/// # Errors
/// Returns an error if any step fails (getting current directory, reading files,
/// or writing the tree object).
pub fn run(args: &WriteTreeArgs) -> Result<()> {
    // Determine the root directory to build the tree from.
    // If no directory is specified by the user, use the current working directory.
    let root = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get the current directory"));

    // Build a Tree object by reading the contents of the directory.
    let tree = build_tree::build_tree(&root)?;

    // Write the Tree object to the object store and get the SHA-1 object ID.
    let oid = hash::write_object(&tree)?;

    // Print the SHA-1 hash of the created tree object.
    println!("{}", oid);

    Ok(())
}
