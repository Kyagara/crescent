use crate::{
    application::Application,
    service::{InitSystem, Service},
};

use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Enable a service for startup")]
pub struct EnableArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl EnableArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(&self.name);
        application.exists()?;

        let init_system = Service::get(Some(&self.name));

        eprintln!("Enabling '{}'", application.service_name);
        init_system.enable()?;

        println!("Sent enable command to '{}'", application.service_name);
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
        let application = Application::from(&self.name);
        application.exists()?;

        let init_system = Service::get(Some(&application.name));

        eprintln!("Disabling '{}'", application.service_name);
        init_system.disable()?;

        println!("Sent disable command to '{}'", application.service_name);
        Ok(())
    }
}
