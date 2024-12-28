use anyhow::Result;
use clap::Args;

use crate::{init_system::InitSystem, service::Service};

#[derive(Args)]
#[command(about = "Enable a service to start at boot")]
pub struct EnableArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl EnableArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        eprintln!("Enabling '{}'", service.name);
        init_system.enable()?;

        println!("Sent enable command to '{}'", service.name);
        Ok(())
    }
}

#[derive(Args)]
#[command(about = "Disable a service from starting at boot")]
pub struct DisableArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl DisableArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        eprintln!("Disabling '{}'", service.name);
        init_system.disable()?;

        println!("Sent disable command to '{}'", service.name);
        Ok(())
    }
}
