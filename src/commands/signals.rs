use std::{io::Write, os::unix::net::UnixStream};

use crate::{
    application,
    subprocess::{self, SocketEvent},
};

use anyhow::{anyhow, Context, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Send a signal to the application subprocess.")]
pub struct SignalArgs {
    #[arg(help = "Application name.")]
    pub name: String,

    #[arg(help = "Signal to send.")]
    pub signal: u8,
}

#[derive(Args)]
#[command(about = "Send a stop command or a SIGTERM signal to the application subprocess.")]
pub struct StopArgs {
    #[arg(help = "Application name.")]
    pub name: String,

    #[arg(short, long, help = "Ignore 'stop_command' and send a SIGTERM signal.")]
    pub force: bool,
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

impl StopArgs {
    pub fn run(self) -> Result<()> {
        if self.force {
            let signal = SignalArgs {
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

#[derive(Args)]
#[command(about = "Send a SIGKILL signal to the application subprocess.")]
pub struct KillArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

impl KillArgs {
    pub fn run(self) -> Result<()> {
        let signal = SignalArgs {
            name: self.name,
            signal: 9,
        };

        signal.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test_utils;
    use anyhow::Context;
    use std::{
        env,
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    #[test]
    fn unit_signal_run() -> Result<()> {
        let name = "unit_signal_run".to_string();
        let command = SignalArgs {
            name: name.clone(),
            signal: 0,
        };

        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");

        let home = env::var("HOME").context("Error getting HOME env.")?;
        let mut path = PathBuf::from(home);
        path.push(".crescent/apps/unit_signal_run");
        fs::create_dir_all(path.clone())?;
        path.push("unit_signal_run.pid");
        let mut file = File::create(path)?;

        let command = SignalArgs {
            name: name.clone(),
            signal: 0,
        };

        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application not running.");

        let pid = std::process::id();
        writeln!(&mut file, "{}", pid)?;
        writeln!(&mut file, "{}", pid)?;

        let command = SignalArgs {
            name: name.clone(),
            signal: 0,
        };
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application not running.");

        test_utils::delete_app_folder(&name)?;
        Ok(())
    }

    #[test]
    fn unit_stop_run() -> Result<()> {
        let name = "unit_stop_run".to_string();
        let command = StopArgs { name, force: true };
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");
        Ok(())
    }
}
