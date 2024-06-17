use std::{path::PathBuf, process::Command};

use crate::{
    service::{InitSystem, Service},
    APPS_DIR, PROFILES_DIR,
};

use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};

#[derive(Args)]
#[command(about = "Edit service units or a profile")]
pub struct EditArgs {
    #[arg(help = "Edit service or profile", value_enum)]
    pub kind: EditKind,

    #[arg(help = "Service/Profile name")]
    pub name: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum EditKind {
    Service,
    Profile,
}

impl EditArgs {
    pub fn run(self) -> Result<()> {
        let path = PathBuf::from(APPS_DIR).join(&self.name);
        if !path.exists() {
            return Err(anyhow!("Application does not exist"));
        }

        if self.kind == EditKind::Profile {
            let path = PathBuf::from(PROFILES_DIR).join(self.name.clone() + ".toml");
            if !path.exists() {
                return Err(anyhow!("Profile does not exist"));
            }

            let mut editor = Command::new("nano").arg(&path).spawn()?;
            let _ = editor.wait();
            println!("Opened '{}' using nano", path.display());
            return Ok(());
        }

        let init_system = Service::get();
        let scripts = init_system.get_scripts_paths();

        for script in scripts {
            println!("Opened '{}' using nano", path.display());
            let mut editor = Command::new("nano").arg(script).spawn()?;
            let _ = editor.wait();
        }

        Ok(())
    }
}
