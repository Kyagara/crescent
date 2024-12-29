use std::{env, io};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use once_cell::sync::Lazy;

use crate::{
    commands::{
        attach::AttachArgs,
        edit::EditArgs,
        enable::{DisableArgs, EnableArgs},
        kill::KillArgs,
        list::ListArgs,
        log::LogArgs,
        new::NewArgs,
        profile::ProfileArgs,
        reload::ReloadArgs,
        restart::RestartArgs,
        send::SendArgs,
        start::{StartArgs, StopArgs},
        status::StatusArgs,
    },
    Commands::{
        Attach, Complete, Disable, Edit, Enable, Kill, List, Log, New, Profile, Reload, Restart,
        Send, Start, Status, Stop,
    },
};

/// Profile folder inside crescent's directory.
///
/// All profiles are stored in this folder in the `toml` format.
static PROFILES_DIR: Lazy<String> = Lazy::new(|| {
    let mut path = env::var("HOME").expect("Error retrieving HOME directory.");
    path.push_str("/.crescent/profiles/");
    path
});

/// Service folder inside crescent's directory.
///
/// Command history and stdin of a service are stored inside a named folder. Example: `$HOME/.crescent/apps/<name>/stdin`.
static APPS_DIR: Lazy<String> = Lazy::new(|| {
    let mut path = env::var("HOME").expect("Error retrieving HOME directory.");
    path.push_str("/.crescent/apps/");
    path
});

mod commands;
mod init_system;
mod init_systems;
mod logger;
mod loggers;
mod profile;
mod service;
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
    New(NewArgs),

    Start(StartArgs),
    Stop(StopArgs),
    Kill(KillArgs),
    Restart(RestartArgs),
    Send(SendArgs),

    Enable(EnableArgs),
    Disable(DisableArgs),

    Status(StatusArgs),
    Log(LogArgs),

    Profile(ProfileArgs),
    Edit(EditArgs),

    Reload(ReloadArgs),
    List(ListArgs),

    #[command(about = "Print a completions file for the specified shell")]
    Complete {
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Crescent::parse();

    match cli.commands {
        Attach(args) => AttachArgs::run(args),
        New(args) => NewArgs::run(args),
        Start(args) => StartArgs::run(args),
        Stop(args) => StopArgs::run(args),
        Kill(args) => KillArgs::run(args),
        Restart(args) => RestartArgs::run(args),
        Send(args) => SendArgs::run(args),
        Enable(args) => EnableArgs::run(args),
        Disable(args) => DisableArgs::run(args),
        Status(args) => StatusArgs::run(args),
        Log(args) => LogArgs::run(args),
        Profile(args) => ProfileArgs::run(args),
        Edit(args) => EditArgs::run(args),
        Reload(args) => ReloadArgs::run(args),
        List(args) => ListArgs::run(args),
        Complete { shell } => {
            clap_complete::generate(shell, &mut Crescent::command(), "cres", &mut io::stdout());
            Ok(())
        }
    }
}
