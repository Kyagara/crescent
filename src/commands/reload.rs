use anyhow::Result;
use clap::Args;

use crate::{application::Application, system::InitSystem};

#[derive(Args)]
#[command(about = "Reload the init system to apply changes to scripts")]
pub struct ReloadArgs;

impl ReloadArgs {
    pub fn run() -> Result<()> {
        let service = Application::from(None);

        service.init_system().reload()?;

        println!("Sent reload command to the init system");
        Ok(())
    }
}
