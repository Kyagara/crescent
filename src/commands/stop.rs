use crate::service::{InitSystem, Service};

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Stop a service")]
pub struct StopArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl StopArgs {
    pub fn run(self) -> Result<()> {
        let mut init_system = Service::get_init_system();
        init_system.set_service_name(&self.name);

        let service_name = format!("cres.{}.service", self.name);

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{service_name}' is not running"));
        }

        eprintln!("Stopping '{service_name}'");
        init_system.stop()?;
        println!("Sent stop command to '{service_name}'");
        Ok(())
    }
}
