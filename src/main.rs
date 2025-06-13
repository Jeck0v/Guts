use anyhow::Result;
mod commands;
mod core;

use commands::hash_object::run_hash_object_command;

fn main() -> Result<()> {
    // Recup arg1 -> file path
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    run_hash_object_command(file_path)
}
