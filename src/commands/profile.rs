use crate::{crescent, util};
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Verify and print a profile.")]
pub struct ProfileArgs {
    #[arg(help = "Profile name.")]
    pub profile: String,

    #[arg(short, long, help = "Prints the profile in prettified json.")]
    pub json: bool,
}

impl ProfileArgs {
    pub fn run(self) -> Result<()> {
        let profile = crescent::get_profile(&self.profile)?;
        let profile_pretty = serde_json::to_string_pretty(&profile)?;

        if self.json {
            println!("{profile_pretty}");
            return Ok(());
        }

        util::print_title_cyan(&format!("Profile '{}'", self.profile));

        if let Some(comment) = profile.__comment {
            util::println_field_white("Comment", comment);
        }

        if let Some(version) = profile.__version {
            util::println_field_white("Version", version);
        }

        if let Some(file_path) = profile.file_path {
            util::println_field_white("File Path", file_path);
        }

        if let Some(name) = profile.name {
            util::println_field_white("Name", name);
        }

        if let Some(interpreter) = profile.interpreter {
            util::println_field_white("Interpreter", interpreter);
        }

        if let Some(interpreter_arguments) = profile.interpreter_arguments {
            util::println_field_white("Interpreter arguments", interpreter_arguments.join(" "));
        }

        if let Some(application_arguments) = profile.application_arguments {
            util::println_field_white("Application arguments", application_arguments.join(" "));
        }

        if let Some(stop_command) = profile.stop_command {
            util::println_field_white("Stop command", stop_command);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_profile_run() -> Result<()> {
        let command = ProfileArgs {
            profile: "profile_not_found".to_string(),
            json: false,
        };
        assert_eq!(command.profile, "profile_not_found");
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Profile not found.");
        Ok(())
    }
}
