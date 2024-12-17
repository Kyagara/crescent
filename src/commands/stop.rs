use anyhow::{anyhow, Result};
use clap::Args;

use crate::{application::Application, system::InitSystem};

#[derive(Args)]
#[command(about = "Stop a service")]
pub struct StopArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl StopArgs {
    pub fn run(self) -> Result<()> {
        let service = Application::from(Some(&self.name));
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
