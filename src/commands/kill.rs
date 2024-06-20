use crate::{
    application::Application,
    service::{InitSystem, Service},
};

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Send a signal to a service. Defaults to SIGTERM (15)")]
pub struct KillArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(help = "Signal to send", default_value_t = 15)]
    pub signal: i32,
}

impl KillArgs {
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

        eprintln!("Sending signal '{}'", application.service_name);
        eprintln!("Signal: '{}'", self.signal);
        init_system.kill(self.signal)?;

        println!("Sent signal to '{}'", application.service_name);
        Ok(())
    }
}
