mod terminal;

use anyhow::Result;
use clap::Parser;
use guts::cli::{Cli, Commands};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // No arguments → launch TUI
        terminal::run_app()?;
        return Ok(());
    }

    let cli = Cli::parse();

    // refactored for TUI output
    match cli.command {
        
        Commands::Init(args) => {
            let output = guts::commands::init::run(&args)?;
            println!("{}", output);
        }
        Commands::HashObject(args) => {
            let output = guts::commands::hash_object::run(&args)?;
            println!("{}", output);
        }
        Commands::CatFile(args) => {
            let output = guts::commands::cat_file::run(&args)?;
            println!("{}", output);
        }
        Commands::WriteTree(args) => {
            let output = guts::commands::write_tree::run(&args)?;
            println!("{}", output);
        }
        Commands::CommitTree(args) => {
            let output = guts::commands::commit_tree::run(&args)?;
            println!("{}", output);
        }
        Commands::Tui => terminal::run_app()?,
        Commands::Status(args) => {
            let output = guts::commands::status::run(&args)?;
            println!("{}", output);
        }
        Commands::Add(args) => {
            let output = guts::commands::add::run(&args)?;
            println!("{}", output);
        }
    }

    Ok(())
}
