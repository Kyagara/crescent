use crate::directory;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Args)]
#[command(about = "Send a command to an application.")]
pub struct SendArgs {
    #[arg(help = "The application name")]
    pub name: String,
    #[arg(help = "The command you want to send", allow_hyphen_values = true)]
    pub command: String,
}

impl SendArgs {
    pub fn run(name: String, command: String) -> Result<()> {
        let mut app_dir = directory::application_dir_by_name(&name)?;

        app_dir.push(name.clone() + ".sock");

        if !app_dir.exists() {
            return Err(anyhow!("Application path does not exist."));
        }

        let mut socket =
            UnixStream::connect(app_dir).context(format!("Couldn't connect to '{name}'."))?;

        let message = format!("{command}\n");

        socket.write_all(message.as_bytes())?;
        socket.flush()?;

        println!("Command sent.");

        Ok(())
    }
}
