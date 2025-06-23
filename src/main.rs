use anyhow::Result;
use clap::Parser;

use guts::cli::{Cli, Commands};

/// Entry point of the Guts CLI application
/// Parses the command-line arguments and dispatches to the corresponding command
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => guts::commands::init::run(&args)?,
        Commands::HashObject(args) => guts::commands::hash_object::run(&args)?,
        Commands::CatFile(args) => guts::commands::cat_file::run(&args)?,
    }

    Ok(())
}
