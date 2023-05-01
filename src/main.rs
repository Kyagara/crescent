use crate::commands::{
    attach::AttachArgs, list::ListArgs, log::LogArgs, send::SendArgs, start::StartArgs,
};
use crate::Commands::*;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod directory;
mod process;
mod tail;

#[derive(Parser)]
#[command(author, version, about = "Process manager written in Rust.")]
struct Crescent {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start(StartArgs),
    List(ListArgs),
    Send(SendArgs),
    Log(LogArgs),
    Attach(AttachArgs),
}

fn main() -> Result<()> {
    let cli = Crescent::parse();

    match cli.commands {
        Start(args) => StartArgs::run(args.file_path, args.name, args.interpreter, args.arguments),
        List(_) => ListArgs::run(),
        Send(args) => SendArgs::run(args.name, args.command),
        Log(args) => LogArgs::run(args.name, args.lines),
        Attach(args) => AttachArgs::run(args.name),
    }?;

    Ok(())
}
