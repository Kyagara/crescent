use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, Result};
use clap::Args;

use crate::{profile::Profiles, PROFILES_DIR};

#[derive(Args)]
#[command(about = "Manage profiles. Install default profiles or edit/create a new one")]
pub struct ProfileArgs {
    #[arg(help = "Profile name")]
    pub name: Option<String>,

    #[arg(help = "Installs default profiles", short, long)]
    pub default: bool,
}

impl ProfileArgs {
    pub fn run(self) -> Result<()> {
        let profiles = Profiles::new();

        if self.default {
            profiles.install_default_profiles()?;
            println!("Installed default profiles.");
            return Ok(());
        }

        if self.name.is_none() {
            return Err(anyhow!("No profile name provided."));
        }

        let name = self.name.clone().unwrap();

        let path = PathBuf::from(PROFILES_DIR.to_owned()).join(name + ".toml");
        let mut editor = Command::new("nano").arg(&path).spawn()?;
        let _ = editor.wait();

        println!("Opened '{}' using nano", path.display());
        Ok(())
    }
}
