mod terminal;

use anyhow::Result;
use clap::Parser;

use guts::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => guts::commands::init::run(&args)?,
        Commands::HashObject(args) => guts::commands::hash_object::run(&args)?,
        Commands::CatFile(args) => guts::commands::cat_file::run(&args)?,
        Commands::WriteTree(args) => guts::commands::write_tree::run(&args)?,
        Commands::Terminal => terminal::run_terminal()?,
    }

    Ok(())
}
