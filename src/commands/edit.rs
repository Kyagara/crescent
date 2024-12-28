use std::process::Command;

use anyhow::Result;
use clap::Args;

use crate::{init_system::InitSystem, service::Service};

#[derive(Args)]
#[command(about = "Edit service scripts")]
pub struct EditArgs {
    #[arg(help = "Service name")]
    pub name: String,
}

impl EditArgs {
    pub fn run(self) -> Result<()> {
        let service = Service::from(Some(&self.name));
        service.exists()?;

        let init_system = service.init_system();

        let scripts = init_system.get_scripts_paths();

        for script in scripts {
            println!("Opened '{}' using nano", script);
            let mut editor = Command::new("nano").arg(script).spawn()?;
            let _ = editor.wait();
        }

        println!("Make sure to 'reload' if any changes were made");
        Ok(())
    }
}
