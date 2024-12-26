use anyhow::Result;
use clap::Args;

use crate::{init_system::InitSystem, service::Service};

#[derive(Args)]
#[command(about = "Reload the init system to apply changes to scripts")]
pub struct ReloadArgs;

impl ReloadArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(None);

        service.init_system().reload()?;

        println!("Sent reload command to the init system");
        Ok(())
    }
}
