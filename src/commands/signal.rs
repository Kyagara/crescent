use crate::{application, subprocess};
use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Send a signal to an application.")]
pub struct SignalArgs {
    #[arg(help = "The application name.")]
    pub name: String,
    #[arg(help = "The signal to send.")]
    pub signal: u8,
}

impl SignalArgs {
    pub fn run(self) -> Result<()> {
        generic_send_signal_command(&self.name, &self.signal)
    }
}

pub fn generic_send_signal_command(name: &String, signal: &u8) -> Result<()> {
    let app_dir = application::app_dir_by_name(name)?;

    if !app_dir.exists() {
        return Err(anyhow!("Application does not exist."));
    }

    let pids = application::app_pids_by_name(name)?;

    if pids.len() < 2 {
        return Err(anyhow!("Application not running."));
    }

    subprocess::check_and_send_signal(&pids[1], signal)?;

    println!("Signal sent.");

    Ok(())
}
