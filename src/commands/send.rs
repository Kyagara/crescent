use std::{fs::OpenOptions, io::Write};

use anyhow::{anyhow, Result};
use clap::Args;

use crate::application::Application;

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
        let application = Application::from(Some(&self.name));
        application.exists()?;

        let stdin = application.stdin_path()?;
        let mut stdin = OpenOptions::new().append(true).open(stdin)?;

        if self.command.join(" ").trim().is_empty() {
            return Err(anyhow!("Command empty"));
        }

        let mut cmd = self.command.join(" ");
        eprintln!("Sending command to '{}'", application.name);
        eprintln!("Command: {cmd}");

        cmd += "\n";
        stdin.write_all(cmd.as_bytes())?;

        println!("Command sent");
        Ok(())
    }
}
