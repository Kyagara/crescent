use anyhow::{anyhow, Result};
use clap::Args;

use crate::{init_system::InitSystem, service::Service};

#[derive(Args)]
#[command(about = "Start a service")]
pub struct StartArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl StartArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        if init_system.is_running()? {
            return Err(anyhow!("Service '{}' is already running", service.name));
        }

        eprintln!("Starting '{}'", service.name);
        init_system.start()?;
        println!("Service '{}' started", service.name);
        Ok(())
    }
}

#[derive(Args)]
#[command(about = "Stop a service")]
pub struct StopArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl StopArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{}' is not running", service.name));
        }

        eprintln!("Stopping '{}'", service.name);
        init_system.stop()?;

        println!("Sent stop command to '{}'", service.name);
        Ok(())
    }
}
