use std::path::PathBuf;
use std::{fs, io};

use crate::commands::{
    edit::EditArgs,
    enable::{DisableArgs, EnableArgs},
    list::ListArgs,
    log::LogArgs,
    profile::ProfileArgs,
    reload::ReloadArgs,
    restart::RestartArgs,
    send::SendArgs,
    start::StartArgs,
    status::StatusArgs,
    stop::StopArgs,
};
use crate::Commands::{
    Complete, Disable, Edit, Enable, List, Log, Profile, Reload, Restart, Send, Start, Status, Stop,
};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

pub const HOME_DIR: &str = env!("HOME", "Error retrieving HOME directory.");

pub const PROFILES_DIR: &str = concat!(
    env!("HOME", "Error retrieving HOME directory."),
    "/.crescent/profiles/"
);

pub const APPS_DIR: &str = concat!(
    env!("HOME", "Error retrieving HOME directory."),
    "/.crescent/apps/"
);

mod commands;
mod logger;
mod loggers;
mod profile;
mod service;
mod services;
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
    Stop(StopArgs),
    Restart(RestartArgs),
    Send(SendArgs),
    Log(LogArgs),
    Status(StatusArgs),
    List(ListArgs),
    Profile(ProfileArgs),
    Edit(EditArgs),
    Reload(ReloadArgs),
    Enable(EnableArgs),
    Disable(DisableArgs),
    #[command(about = "Print a completions file for the specified shell")]
    Complete {
        shell: Shell,
    },
}

fn main() -> Result<()> {
    // Create directories if they don't exist
    fs::create_dir_all(PathBuf::from(APPS_DIR))?;
    fs::create_dir_all(PathBuf::from(PROFILES_DIR))?;

    let cli = Crescent::parse();

    match cli.commands {
        Start(args) => StartArgs::run(args),
        List(_) => ListArgs::run(),
        Stop(args) => StopArgs::run(args),
        Restart(args) => RestartArgs::run(args),
        Send(args) => SendArgs::run(args),
        Log(args) => LogArgs::run(args),
        Status(args) => StatusArgs::run(args),
        Profile(args) => ProfileArgs::run(args),
        Edit(args) => EditArgs::run(args),
        Reload(_) => ReloadArgs::run(),
        Enable(args) => EnableArgs::run(args),
        Disable(args) => DisableArgs::run(args),
        Complete { shell } => {
            clap_complete::generate(shell, &mut Crescent::command(), "cres", &mut io::stdout());
            Ok(())
        }
    }
}
