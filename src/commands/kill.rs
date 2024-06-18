use crate::service::{InitSystem, Service};

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Send a signal to a service. Defaults to SIGTERM (15)")]
pub struct KillArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(help = "Signal to send")]
    pub signal: Option<i32>,
}

impl KillArgs {
    pub fn run(self) -> Result<()> {
        let mut init_system = Service::get();
        init_system.set_service_name(&self.name);

        let service_name = format!("cres.{}.service", self.name);

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{service_name}' is not running"));
        }

        let signal = self.signal.unwrap_or(15);

        eprintln!("Sending signal '{service_name}'");
        init_system.kill(signal)?;

        println!("Sent signal '{signal}' to '{service_name}'");
        Ok(())
    }
}
