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
            print!("{}", output);
        }
        Commands::WriteTree(args) => {
            let output = guts::commands::write_tree::run(&args)?;
            println!("{}", output);
        }
        Commands::CommitTree(args) => {
            let output = guts::commands::commit_tree::run(&args)?;
            println!("{}", output);
        }
        Commands::Status(args) => {
            let output = guts::commands::status::run(&args)?;
            println!("{}", output);
        }
        Commands::Add(args) => {
            let output = guts::commands::add::run(&args)?;
            println!("{}", output);
        }
        Commands::Rm(args) => {
            let output = guts::commands::rm::run(&args)?;
            println!("{}", output);
        }
        Commands::Commit(args) => {
            let output = guts::commands::commit::run(&args)?;
            println!("{}", output);
        }
        Commands::RevParse(args) => {
            let output = guts::commands::rev_parse::run(&args)?;
            println!("{}", output)
        }
        Commands::Log(args) => {
            let output = guts::commands::log::run(&args)?;
            println!("{}", output);
        }
        Commands::LsFiles(args) => {
            let output = guts::commands::ls_files::run(&args)?;
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        Commands::LsTree(args) => {
            let output = guts::commands::ls_tree::run(&args)?;
            println!("{}", output);
        }
        Commands::ShowRef(args) => {
            let output = guts::commands::show_ref::run(&args)?;
            println!("{}", output);
        }
        Commands::Tui => terminal::run_app()?,  
    }

    Ok(())
}
