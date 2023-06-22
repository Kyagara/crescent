use super::signal;
use crate::{application, subprocess::SocketEvent};
use anyhow::{Context, Result};
use clap::Args;
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Args)]
#[command(about = "Send a stop command or a SIGTERM signal to the application subprocess.")]
pub struct StopArgs {
    #[arg(help = "Application name.")]
    pub name: String,

    #[arg(
        short,
        long,
        help = "Ignore 'stop command' if defined and send a SIGTERM signal."
    )]
    pub force: bool,
}

impl StopArgs {
    pub fn run(self) -> Result<()> {
        if self.force {
            let signal = signal::SignalArgs {
                name: self.name,
                signal: 15,
            };

            return signal.run();
        }

        let mut app_dir = application::app_dir_by_name(&self.name)?;

        app_dir.push(self.name.clone() + ".sock");

        let mut stream = UnixStream::connect(app_dir)
            .context(format!("Error connecting to '{}' socket.", self.name))?;

        let event = serde_json::to_vec(&SocketEvent::Stop)?;

        stream.write_all(&event)?;

        println!("Stop command sent.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_stop_run() -> Result<()> {
        let name = "unit_stop_run".to_string();
        let command = StopArgs { name, force: true };
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");
        Ok(())
    }
}
