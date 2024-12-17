use anyhow::Result;
use clap::Args;

use crate::{application::Application, system::InitSystem};

#[derive(Args)]
#[command(about = "Enable a service for startup")]
pub struct EnableArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl EnableArgs {
    pub fn run(self) -> Result<()> {
        let service = Application::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        eprintln!("Enabling '{}'", service.name);
        init_system.enable()?;

        println!("Sent enable command to '{}'", service.name);
        Ok(())
    }
}

#[derive(Args)]
#[command(about = "Disable a service for startup")]
pub struct DisableArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl DisableArgs {
    pub fn run(self) -> Result<()> {
        let service = Application::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        eprintln!("Disabling '{}'", service.name);
        init_system.disable()?;

        println!("Sent disable command to '{}'", service.name);
        Ok(())
    }
}
