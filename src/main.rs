use std::{fs, io, path::PathBuf};

use crate::commands::{
    attach::AttachArgs,
    edit::EditArgs,
    enable::{DisableArgs, EnableArgs},
    kill::KillArgs,
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
    Attach, Complete, Disable, Edit, Enable, Kill, List, Log, Profile, Reload, Restart, Send,
    Start, Status, Stop,
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

mod application;
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
    Attach(AttachArgs),
    Start(StartArgs),
    Stop(StopArgs),
    Kill(KillArgs),
    Restart(RestartArgs),
    Send(SendArgs),
    Log(LogArgs),
    Status(StatusArgs),
    List,
    Profile(ProfileArgs),
    Edit(EditArgs),
    Reload,
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
        Attach(args) => AttachArgs::run(args),
        Start(args) => StartArgs::run(args),
        List => ListArgs::run(),
        Stop(args) => StopArgs::run(args),
        Kill(args) => KillArgs::run(args),
        Restart(args) => RestartArgs::run(args),
        Send(args) => SendArgs::run(args),
        Log(args) => LogArgs::run(args),
        Status(args) => StatusArgs::run(args),
        Profile(args) => ProfileArgs::run(args),
        Edit(args) => EditArgs::run(args),
        Reload => ReloadArgs::run(),
        Enable(args) => EnableArgs::run(args),
        Disable(args) => DisableArgs::run(args),
        Complete { shell } => {
            clap_complete::generate(shell, &mut Crescent::command(), "cres", &mut io::stdout());
            Ok(())
        }
    }
}
