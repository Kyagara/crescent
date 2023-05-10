use crate::commands::{
    attach::AttachArgs, kill::KillArgs, list::ListArgs, log::LogArgs, send::SendArgs,
    signal::SignalArgs, start::StartArgs, stop::StopArgs,
};
use crate::Commands::*;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod application;
mod commands;
mod crescent;
mod logger;
mod subprocess;
mod tail;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Process manager for game servers or services."
)]
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
    Signal(SignalArgs),
    Stop(StopArgs),
    Kill(KillArgs),
}

fn main() -> Result<()> {
    let cli = Crescent::parse();

    match cli.commands {
        Start(args) => StartArgs::run(args),
        List(_) => ListArgs::run(),
        Send(args) => SendArgs::run(args),
        Log(args) => LogArgs::run(args),
        Attach(args) => AttachArgs::run(args),
        Signal(args) => SignalArgs::run(args),
        Stop(args) => StopArgs::run(args),
        Kill(args) => KillArgs::run(args),
    }?;

    Ok(())
}
