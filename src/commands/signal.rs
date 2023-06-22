use crate::{application, subprocess};
use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Send a signal to the application subprocess.")]
pub struct SignalArgs {
    #[arg(help = "Application name.")]
    pub name: String,

    #[arg(help = "Signal to send.")]
    pub signal: u8,
}

impl SignalArgs {
    pub fn run(self) -> Result<()> {
        application::check_app_exists(&self.name)?;

        if !application::app_already_running(&self.name)? {
            return Err(anyhow!("Application not running."));
        }

        let pids = application::app_pids_by_name(&self.name)?;

        subprocess::send_unix_signal(pids[1], self.signal)?;

        println!("Signal sent.");

        Ok(())
    }
}
