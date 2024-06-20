use std::{fs::OpenOptions, io::Write};

use crate::application::Application;

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Send a command to a service")]
pub struct SendArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(help = "Command", allow_hyphen_values = true)]
    pub command: Vec<String>,
}

impl SendArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(&self.name);
        application.exists()?;

        let stdin = application.stdin_path()?;
        let mut stdin = OpenOptions::new().append(true).open(stdin)?;

        if self.command.join(" ").trim().is_empty() {
            return Err(anyhow!("Command empty"));
        }

        let mut cmd = self.command.join(" ");
        eprintln!("Sending command to '{}'", application.service_name);
        eprintln!("Command: {cmd}");

        cmd += "\n";
        stdin.write_all(cmd.as_bytes())?;

        println!("Command sent");
        Ok(())
    }
}
