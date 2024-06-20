use crate::{
    application::Application,
    service::{InitSystem, Service},
};

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Restart a service")]
pub struct RestartArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl RestartArgs {
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

        eprintln!("Restarting '{}'", application.service_name);
        init_system.restart()?;

        println!("Sent restart command to '{}'", application.service_name);
        Ok(())
    }
}
