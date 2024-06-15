use std::path::PathBuf;
use std::{fs, io};

use crate::commands::{
    list::ListArgs, log::LogArgs, profile::ProfileArgs, send::SendArgs, start::StartArgs,
    status::StatusArgs,
};
use crate::Commands::{Complete, List, Log, Profile, Send, Start, Status};

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
    List(ListArgs),
    Send(SendArgs),
    Log(LogArgs),
    Status(StatusArgs),
    Profile(ProfileArgs),
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
        Send(args) => SendArgs::run(args),
        Log(args) => LogArgs::run(args),
        Status(args) => StatusArgs::run(args),
        Profile(args) => ProfileArgs::run(args),
        Complete { shell } => {
            clap_complete::generate(shell, &mut Crescent::command(), "cres", &mut io::stdout());
            Ok(())
        }
    }
}
