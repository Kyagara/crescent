use std::{fs, io, path::PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::{
    commands::{
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
    },
    Commands::{
        Attach, Complete, Disable, Edit, Enable, Kill, List, Log, Profile, Reload, Restart, Send,
        Start, Status, Stop,
    },
};

/// User's home directory.
pub const HOME_DIR: &str = env!("HOME", "Error retrieving HOME directory.");

/// Profile directory inside crescent's directory.
///
/// All profiles are stored in this folder in the `toml` format.
pub const PROFILES_DIR: &str = concat!(
    env!("HOME", "Error retrieving HOME directory."),
    "/.crescent/profiles/"
);

/// Application directory inside crescent's directory.
///
/// Command history and stdin for an application are stored inside a folder named after the application. Example: `$HOME/.crescent/apps/<name>/stdin`.
pub const APPS_DIR: &str = concat!(
    env!("HOME", "Error retrieving HOME directory."),
    "/.crescent/apps/"
);

mod application;
mod commands;
mod logger;
mod loggers;
mod profile;
mod system;
mod systems;
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
    // Create directories if they don't exist
    fs::create_dir_all(PathBuf::from(APPS_DIR))?;
    fs::create_dir_all(PathBuf::from(PROFILES_DIR))?;

    let cli = Crescent::parse();

    match cli.commands {
        Attach(args) => AttachArgs::run(args),
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
