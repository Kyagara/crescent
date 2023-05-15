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

    match subprocess::check_and_send_signal(&pids[1], signal) {
        Ok(exists) => {
            if exists {
                println!("Signal sent.");
            } else {
                println!("Subprocess not found, signal not sent.");
            }
            Ok(())
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_signal_run() -> Result<()> {
        let command = SignalArgs {
            name: "signal_run".to_string(),
            signal: 0,
        };

        assert_eq!(command.name, "signal_run");
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");

        Ok(())
    }
}
