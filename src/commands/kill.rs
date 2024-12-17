use anyhow::{anyhow, Result};
use clap::Args;

use crate::{application::Application, system::InitSystem};

#[derive(Args)]
#[command(about = "Send a signal to a service. Defaults to SIGTERM (15)")]
pub struct KillArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(help = "Signal to send", default_value_t = 15)]
    pub signal: i32,
}

impl KillArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(Some(&self.name));
        application.exists()?;

        let init_system = application.init_system();

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{}' is not running", application.name));
        }

        eprintln!("Sending signal '{}'", application.name);
        eprintln!("Signal: '{}'", self.signal);
        init_system.kill(self.signal)?;

        println!("Sent signal to '{}'", application.name);
        Ok(())
    }
}
