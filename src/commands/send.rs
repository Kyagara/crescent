use std::{fs::OpenOptions, io::Write, path::PathBuf};

use crate::APPS_DIR;

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
        let path = PathBuf::from(APPS_DIR).join(&self.name);
        if !path.exists() {
            return Err(anyhow!("Application does not exist"));
        }

        let stdin = path.join("stdin");
        let stdin = if stdin.exists() { Some(stdin) } else { None };
        if stdin.is_none() {
            return Err(anyhow!("'{}' stdin does not exist", self.name));
        }

        let mut stdin = OpenOptions::new().write(true).open(stdin.unwrap())?;

        if self.command.join(" ").trim().is_empty() {
            return Err(anyhow!("Command empty"));
        }

        let mut cmd = self.command.join(" ");
        eprintln!("Sending command to application '{}'", self.name);
        eprintln!("Command: {cmd}");

        cmd += "\n";
        stdin.write_all(cmd.as_bytes())?;

        println!("Command sent");
        Ok(())
    }
}
