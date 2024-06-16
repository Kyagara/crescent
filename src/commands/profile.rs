use crate::{profile::Profiles, util};

use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Verify and print a profile")]
pub struct ProfileArgs {
    #[arg(help = "Profile name")]
    pub profile_name: String,
}

impl ProfileArgs {
    pub fn run(self) -> Result<()> {
        let mut profiles = Profiles::new();
        let profile = profiles.get_profile(&self.profile_name)?;

        util::print_title_cyan(&format!("Profile: {}", self.profile_name));

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

        if let Some(stop_command) = profile.stop_command {
            util::println_field_white("stop_command", stop_command);
        }

        Ok(())
    }
}
