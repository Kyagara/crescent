use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{Args, ValueHint};

use crate::{application::Application, profile::Profiles, system::InitSystem};

#[derive(Args)]
#[command(about = "Starts an executable as a background service")]
pub struct StartArgs {
    #[arg(help = "Path to the executable", value_hint = ValueHint::FilePath, required = false, required_unless_present = "profile")]
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
        long
    )]
    pub interpreter: Option<String>,

    #[arg(
        help = "Defaults to the executable name. Service will be named \"cres.<name>.service\"",
        short,
        long
    )]
    pub name: Option<String>,

    #[arg(help = "Name of the profile to load fields from", short, long)]
    pub profile: Option<String>,

    #[arg(help = "Overwrite existing service file(s)", short, long)]
    pub force: bool,
}

impl StartArgs {
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

        let exec_path = self.exec_path.clone();

        // Check if even after overwriting arguments `exec_path` is still empty.
        if self.exec_path.is_none() {
            return Err(anyhow!("Executable path is empty."));
        }

        let exec_path = exec_path.unwrap();

        // First, check if `exec_path` provided is the name of an application.
        let service = Application::from(Some(&exec_path));

        // If `force` is not set, check if an application with the same name already exists.
        if !self.force && service.exists().is_ok() {
            let name = exec_path;

            let init_system = service.init_system();
            if init_system.is_running()? {
                return Err(anyhow!(
                    "A service with the name '{name}' is already running."
                ));
            }

            eprintln!("Starting '{name}'");
            init_system.start()?;
            println!("Service '{name}' started");
            return Ok(());
        }

        // An application could not be found or `force` was set, check if `exec_path` is a valid file.
        let path = Path::new(&exec_path);

        if !path.exists() {
            return Err(anyhow!("The path '{exec_path}' does not exist."));
        }

        if !path.is_file() {
            return Err(anyhow!("The path '{exec_path}' does not point to a file."));
        }

        // If `name` was not provided, try to extract one from the executable.
        if self.name.is_none() {
            self.name = path
                .file_stem()
                .map(|file_name| file_name.to_string_lossy().to_string());
        }

        // Get a name that can be used as an application name.
        let name = self.sanitize_name()?;
        if name.is_empty() {
            return Err(anyhow!("A name for the service could not be determined."));
        }

        let application = Application::from(Some(&name));
        let init_system = service.init_system();

        if init_system.is_running()? {
            return Err(anyhow!(
                "A service with the name '{name}' is already running."
            ));
        }

        // If the application does not exist or force is set, create the application and scripts.
        if application.exists().is_err() || self.force {
            let exec_cmd = self.format_exec_cmd(exec_path.to_string());
            eprintln!("CMD: '{exec_cmd}'");

            init_system.create(&exec_cmd)?;
            eprintln!("Service '{name}' created");

            init_system.reload()?;
        }

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
        let name = self.name.clone().unwrap_or_default();
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

    fn format_exec_cmd(&self, exec_path: String) -> String {
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
