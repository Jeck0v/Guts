use clap::{Parser, Subcommand};

use crate::commands::{
    add, cat_file, commit, commit_tree, hash_object, init, log, rev_parse, rm, show_ref, status,
    write_tree,ls_tree,ls_files
};

#[derive(Parser)]
#[command(
    name = "guts",
    version,
    author,
    about = "A Git implementation in Rust like Guts"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
/// we add the functions we're going to call and put in the main.rs commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new guts repository
    Init(init::InitArgs),

    /// Hash a file as a Git blob
    HashObject(hash_object::HashObjectArgs),

    /// Read a blob
    CatFile(cat_file::CatFileArgs),

    /// Write a tree
    WriteTree(write_tree::WriteTreeArgs),

    // Commit a tree
    CommitTree(commit_tree::CommitObject),

    // Get the status of the current repo
    Status(status::StatusObject),

    /// Add files to the staging area
    Add(add::AddArgs),

    /// Remove files from the staging area
    Rm(rm::RmArgs),

    /// Create a new commit
    Commit(commit::CommitArgs),

    /// Convert ref/branch/HEAD into SHA-1.
    RevParse(rev_parse::RevParse),

    /// Log head
    Log(log::LogArgs),
  
    /// List the contents of a tree object
    LsTree(ls_tree::LsTreeArgs),

    /// Show all refs and their hashes
    ShowRef(show_ref::ShowRefArgs),

    /// List all files in the index
    LsFiles(ls_files::LsFilesArgs),

    /// Launch graphical terminal UI
    Tui,
}
