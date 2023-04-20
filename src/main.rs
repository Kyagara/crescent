use clap::{Parser, Subcommand};
mod commands;
mod directory;
mod process;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about = "Process manager written in Rust.")]
struct Crescent {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start(commands::start::StartArgs),
    List(commands::list::ListArgs),
    Send(commands::send::SendArgs),
    Log(commands::log::LogArgs),
}

fn main() -> Result<()> {
    let cli = Crescent::parse();

    match cli.commands {
        Commands::Start(args) => {
            commands::start::StartArgs::run(args.file_path, args.name, args.command, args.arguments)
        }
        Commands::List(_) => commands::list::ListArgs::run(),
        Commands::Send(args) => commands::send::SendArgs::run(args.name, args.command),
        Commands::Log(args) => commands::log::LogArgs::run(args.name, args.lines),
    }?;

    Ok(())
}
