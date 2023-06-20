use crate::{application, subprocess::SocketEvent};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Args)]
#[command(about = "Send a command to an application.")]
pub struct SendArgs {
    #[arg(help = "Application name.")]
    pub name: String,

    #[arg(help = "Command to send.", allow_hyphen_values = true)]
    pub command: Vec<String>,
}

impl SendArgs {
    pub fn run(self) -> Result<()> {
        application::check_app_exists(&self.name)?;

        if self.command.join(" ").trim().is_empty() {
            return Err(anyhow!("Command empty."));
        }

        let mut app_dir = application::app_dir_by_name(&self.name)?;

        app_dir.push(self.name.clone() + ".sock");

        let mut stream = UnixStream::connect(app_dir)
            .context(format!("Error connecting to '{}' socket.", self.name))?;

        let event = serde_json::to_vec(&SocketEvent::WriteStdin(self.command.join(" ")))?;

        stream.write_all(&event)?;
        stream.flush()?;

        println!("Command sent.");

        Ok(())
    }
}
