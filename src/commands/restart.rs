use anyhow::{anyhow, Result};
use clap::Args;

use crate::{application::Application, system::InitSystem};

#[derive(Args)]
#[command(about = "Restart a service")]
pub struct RestartArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl RestartArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(Some(&self.name));
        application.exists()?;

        let init_system = application.init_system();

        if !init_system.is_running()? {
            return Err(anyhow!("Service '{}' is not running", application.name));
        }

        eprintln!("Restarting '{}'", application.name);
        init_system.restart()?;

        println!("Sent restart command to '{}'", application.name);
        Ok(())
    }
}
