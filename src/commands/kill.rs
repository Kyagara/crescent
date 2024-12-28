use anyhow::{anyhow, Result};
use clap::Args;

use crate::{init_system::InitSystem, service::Service};

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
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{}' is not running", service.name));
        }

        eprintln!("Sending signal '{}'", service.name);
        eprintln!("Signal: '{}'", self.signal);
        init_system.kill(self.signal)?;

        println!("Sent signal to '{}'", service.name);
        Ok(())
    }
}
