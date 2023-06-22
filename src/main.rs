use crate::commands::{
    attach::AttachArgs, kill::KillArgs, list::ListArgs, log::LogArgs, profile::ProfileArgs,
    save::SaveArgs, send::SendArgs, signal::SignalArgs, start::StartArgs, status::StatusArgs,
    stop::StopArgs,
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
mod test_util;
mod util;

#[derive(Parser)]
#[command(name = "crescent", version, about)]
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
    Status(StatusArgs),
    Profile(ProfileArgs),
    Save(SaveArgs),
}

fn main() -> Result<()> {
    let cli = Crescent::parse();

    match cli.commands {
        Start(args) => StartArgs::run(args),
        List(args) => ListArgs::run(args),
        Send(args) => SendArgs::run(args),
        Log(args) => LogArgs::run(args),
        Attach(args) => AttachArgs::run(args),
        Signal(args) => SignalArgs::run(args),
        Stop(args) => StopArgs::run(args),
        Status(args) => StatusArgs::run(args),
        Kill(args) => KillArgs::run(args),
        Profile(args) => ProfileArgs::run(args),
        Save(args) => SaveArgs::run(args),
    }?;

    Ok(())
}
