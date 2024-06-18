use crate::{profile::Profiles, util};

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Manage profiles")]
pub struct ProfileArgs {
    #[arg(help = "Profile name")]
    pub profile_name: Option<String>,

    #[arg(help = "Installs default profiles", short, long)]
    pub default: bool,
}

impl ProfileArgs {
    pub fn run(self) -> Result<()> {
        let mut profiles = Profiles::new();

        if self.default {
            profiles.install_default_profiles()?;
            println!("Installed default profiles.");
            return Ok(());
        }

        let profile_name = if let Some(name) = self.profile_name {
            name
        } else {
            return Err(anyhow!("No profile name provided."));
        };

        let profile = profiles.get_profile(&profile_name)?;

        util::print_title_cyan(&format!("Profile: {profile_name}"));

        if let Some(exec_path) = profile.exec_path {
            util::println_field_white("exec_path", exec_path.to_string_lossy());
        }

        if let Some(name) = profile.name {
            util::println_field_white("name", name);
        }

        if let Some(interpreter) = profile.interpreter {
            util::println_field_white("interpreter", interpreter);
        }

        if let Some(arguments) = profile.arguments {
            util::println_field_white("arguments", arguments);
        }

        Ok(())
    }
}
