use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{Args, ValueHint};

use crate::{init_system::InitSystem, profile::Profiles, service::Service, util};

#[derive(Args)]
#[command(about = "Create and start a new background service")]
pub struct NewArgs {
    #[arg(help = "Path to the executable", value_hint = ValueHint::FilePath)]
    pub exec_path: Option<String>,

    #[arg(
        help = "Arguments for the executable",
        short,
        long,
        allow_hyphen_values = true
    )]
    pub arguments: Option<String>,

    #[arg(
        help = "node, java, etc. Accepts arguments, example: \"java -Xmx512m -jar\"",
        short,
        long,
        allow_hyphen_values = true
    )]
    pub interpreter: Option<String>,

    #[arg(help = "Defaults to the executable name.", short, long)]
    pub name: Option<String>,

    #[arg(help = "Name of the profile to load fields from", short, long)]
    pub profile: Option<String>,
}

impl NewArgs {
    pub fn run(mut self) -> Result<()> {
        // Overwrite all arguments if a profile was provided.
        if let Some(profile_name) = &self.profile {
            let mut profiles = Profiles::new();
            let profile = profiles.get_profile(profile_name)?;

            // If the field is set, it takes precedence over the loaded field.
            // This is useful when the loaded field is None and we want to keep it that way.
            let overwrite_value = |set: Option<String>, loaded: Option<String>| match set {
                Some(field) => Some(field),
                None => match loaded {
                    Some(path) => Some(path),
                    None => set,
                },
            };

            self.exec_path = overwrite_value(self.exec_path, profile.exec_path);
            self.name = overwrite_value(self.name, profile.name);
            self.interpreter = overwrite_value(self.interpreter, profile.interpreter);
            self.arguments = overwrite_value(self.arguments, profile.arguments);
        }

        // Check if after overwriting arguments `exec_path` is not empty and exists.
        let exec_path = match self.exec_path {
            Some(ref str) => {
                let path = Path::new(str);
                if !path.exists() {
                    return Err(anyhow!("Executable path does not exist."));
                }
                let path = path.canonicalize()?;
                path.to_string_lossy().to_string()
            }
            None => return Err(anyhow!("Executable path is empty.")),
        };

        let path = Path::new(&exec_path);

        // If `name` was not provided, try to extract one from the executable.
        if self.name.is_none() {
            self.name = path
                .file_stem()
                .map(|file_name| file_name.to_string_lossy().to_string());
        }

        // Get a name that can be used as an service name.
        let name = self.sanitize_name()?;
        if name.is_empty() {
            return Err(anyhow!("A name for the service could not be determined."));
        }

        let service = Service::from(Some(&name));
        let init_system = service.init_system();

        if init_system.is_running()? {
            return Err(anyhow!(
                "A service with the name '{name}' is already running."
            ));
        }

        let exec_cmd = self.format_exec_cmd(&exec_path);

        util::println_field_value("Service name", &name);
        util::println_field_value("Profile", self.profile.clone().unwrap_or_default());
        util::println_field_value("Exec path", exec_path);
        util::println_field_value("Interpreter", self.interpreter.clone().unwrap_or_default());
        util::println_field_value("Arguments", self.arguments.clone().unwrap_or_default());
        util::println_field_value("CMD", &exec_cmd);
        util::println_white("Scripts paths:");
        init_system.get_scripts_paths().iter().for_each(|path| {
            println!("{path}");
        });

        util::confirm("Create this service?")?;

        init_system.create(&exec_cmd)?;
        eprintln!("Service '{name}' created");

        init_system.reload()?;

        eprintln!("Starting '{name}'");
        init_system.start()?;
        println!("Service '{name}' started");
        Ok(())
    }

    /// Checks if the name is valid and returns a cleaned version.
    ///
    /// Name must:
    /// - Be alphanumeric, can contain `_`, `-`, `.`
    /// - Lowercase
    /// - Max length of 16 characters
    fn sanitize_name(&self) -> Result<String> {
        let name = &self.name.clone().unwrap_or_default();
        let name = name.to_lowercase();

        if name.len() > 16 {
            return Err(anyhow!("Name is too long. Max length is 16 characters."));
        }

        let valid_chars = ['_', '-', '.'];

        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || valid_chars.contains(&c))
        {
            return Err(anyhow!(
                "Name contains invalid characters: '{name}'. Only alphanumeric characters are allowed."
            ));
        }

        Ok(name)
    }

    fn format_exec_cmd(&self, exec_path: &String) -> String {
        let mut exec_cmd = Vec::new();

        if let Some(interpreter) = &self.interpreter {
            exec_cmd.push(interpreter.to_string());
        };

        exec_cmd.push(exec_path.to_string());

        if let Some(arguments) = &self.arguments {
            exec_cmd.push(arguments.to_string());
        }

        exec_cmd.join(" ")
    }
}
