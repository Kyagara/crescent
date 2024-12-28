use anyhow::{anyhow, Result};
use clap::Args;

use crate::{init_system::InitSystem, service::Service};

#[derive(Args)]
#[command(about = "Restart a service")]
pub struct RestartArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl RestartArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{}' is not running", service.name));
        }

        eprintln!("Restarting '{}'", service.name);
        init_system.restart()?;

        println!("Sent restart command to '{}'", service.name);
        Ok(())
    }
}
