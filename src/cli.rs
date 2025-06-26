use clap::{Parser, Subcommand};

use crate::commands::{hash_object, init, cat_file, write_tree};

#[derive(Parser)]
#[command(name = "guts", version, author, about = "A Git implementation in Rust like Guts")]
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

    /// Launch graphical terminal UI
    Terminal,
}
