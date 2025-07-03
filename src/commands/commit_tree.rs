use std::env;

use clap::Args;

use std::path::PathBuf;

use anyhow::Result;

use crate::core::hash;

use crate::core::object::Commit;

/// Structure representing arguments for the 'commit' subcommand
#[derive(Args)]
pub struct CommitObject {
    /// The tree object ID this commit points to
    pub tree: String,

    /// Optional parent commit object ID (-p or --parent)
    #[arg(short = 'p', long)]
    pub parent: Option<String>,
    
    /// Commit message (-m or --message)
    #[arg(short = 'm', long)]
    pub message: String,

    /// Optional path to the .guts directory (--git-dir)
    #[arg(long)]
    pub git_dir: Option<PathBuf>
}

/// Function to execute the commit logic
pub fn run(args: &CommitObject) -> Result<()> {
    // Determine the path to the .guts directory
    // If not specified, use current directory + ".guts"
    let git_dir = args.git_dir.clone().unwrap_or_else(|| {
        env::current_dir().expect("could not get the current dir").join(".guts")
    });

    // Check if the .guts directory exists; return error if not
    if !git_dir.exists() {
        anyhow::bail!("No .guts directory at {}", git_dir.display());
    }

    // Create a new Commit object using the provided arguments
    let commit = Commit {
        tree: args.tree.clone(),
        parent: args.parent.clone(),
        message: args.message.clone(),
    };

    // Write the commit object to disk and get its object ID (hash)
    let oid = hash::write_object(&commit)?;

    // Output the resulting object ID to stdout
    println!("{}", oid);
    
    Ok(())
}
