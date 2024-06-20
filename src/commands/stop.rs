use crate::{
    application::Application,
    service::{InitSystem, Service},
};

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
        let application = Application::from(&self.name);
        application.exists()?;

        let init_system = Service::get(Some(&application.name));

        if !init_system.is_running()? {
            return Err(anyhow!(
                "Service '{}' is not running",
                application.service_name
            ));
        }

        eprintln!("Stopping '{}'", application.service_name);
        init_system.stop()?;

        println!("Sent stop command to '{}'", application.service_name);
        Ok(())
    }
}
