use crate::directory;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Args)]
#[command(about = "Send a command to an application.")]
pub struct SendArgs {
    #[arg(help = "The application name")]
    pub name: String,
    #[arg(help = "The command to send", allow_hyphen_values = true)]
    pub command: String,
}

impl SendArgs {
    pub fn run(self) -> Result<()> {
        let mut app_dir = directory::application_dir_by_name(&self.name)?;

        if !app_dir.exists() {
            return Err(anyhow!("Application does not exist."));
        }

        app_dir.push(self.name.clone() + ".sock");

        let mut socket = UnixStream::connect(app_dir)
            .context(format!("Error connecting to '{}' socket.", self.name))?;

        let message = format!("{}\n", self.command);

        socket.write_all(message.as_bytes())?;
        socket.flush()?;

        println!("Command sent.");

        Ok(())
    }
}
