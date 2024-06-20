use crate::service::{InitSystem, Service};

use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Reload the init system to apply changes to scripts")]
pub struct ReloadArgs;

impl ReloadArgs {
    pub fn run() -> Result<()> {
        let init_system = Service::get(None);

        init_system.reload()?;

        println!("Sent reload command to the init system");
        Ok(())
    }
}
