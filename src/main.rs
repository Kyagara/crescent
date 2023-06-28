use crate::commands::{
    attach::AttachArgs, kill::KillArgs, list::ListArgs, log::LogArgs, profile::ProfileArgs,
    save::SaveArgs, send::SendArgs, signal::SignalArgs, start::StartArgs, status::StatusArgs,
    stop::StopArgs,
};
use crate::Commands::*;
use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::io;

mod application;
mod commands;
mod crescent;
mod logger;
mod subprocess;
mod tail;
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
    #[command(about = "Print a completions file for the specified shell.")]
    Complete {
        shell: Shell,
    },
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
        Complete { shell } => {
            clap_complete::generate(shell, &mut Crescent::command(), "cres", &mut io::stdout());
            Ok(())
        }
    }
}
