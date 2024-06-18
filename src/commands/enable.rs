use std::path::PathBuf;

use crate::{
    service::{InitSystem, Service},
    APPS_DIR,
};

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Enable a service for startup")]
pub struct EnableArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl EnableArgs {
    pub fn run(self) -> Result<()> {
        let path = PathBuf::from(APPS_DIR).join(&self.name);
        if !path.exists() {
            return Err(anyhow!("Application '{}' does not exist", self.name));
        }

        let mut init_system = Service::get();
        init_system.set_service_name(&self.name);

        let service_name = format!("cres.{}.service", self.name);

        eprintln!("Enabling '{service_name}'");
        init_system.enable()?;

        println!("Sent enable command to '{service_name}'");
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
        let path = PathBuf::from(APPS_DIR).join(&self.name);
        if !path.exists() {
            return Err(anyhow!("Application '{}' does not exist", self.name));
        }

        let mut init_system = Service::get();
        init_system.set_service_name(&self.name);

        let service_name = format!("cres.{}.service", self.name);

        eprintln!("Disabling '{service_name}'");
        init_system.disable()?;

        println!("Sent disable command to '{service_name}'");
        Ok(())
    }
}
