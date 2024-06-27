use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    profile::{Profile, Profiles},
    service::{InitSystem, Service},
    APPS_DIR,
};

use anyhow::{anyhow, Result};
use clap::{Args, ValueHint};

#[derive(Args)]
#[command(about = "Starts an executable as a background service")]
pub struct StartArgs {
    #[arg(help = "Path to the executable", value_hint = ValueHint::AnyPath)]
    pub exec_path: Option<String>,

    #[arg(
        help = "Defaults to the executable name. Service will be named \"cres.*.service\"",
        short,
        long
    )]
    pub name: Option<String>,

    #[arg(
        help = "node, java, etc. Accepts arguments, example: \"java -Xmx512m -jar\"",
        short,
        long
    )]
    pub interpreter: Option<String>,

    #[arg(
        help = "Arguments for the executable",
        short,
        long,
        allow_hyphen_values = true
    )]
    pub arguments: Option<String>,

    #[arg(help = "Overwrite existing service file(s)", short, long)]
    pub force: bool,

    #[arg(help = "Name of the profile to load fields from", short, long)]
    pub profile: Option<String>,
}

impl From<Profile> for StartArgs {
    fn from(profile: Profile) -> Self {
        Self {
            exec_path: profile.exec_path,
            name: profile.name,
            interpreter: profile.interpreter,
            arguments: profile.arguments,

            // These fields will be ignored.
            force: false,
            profile: None,
        }
    }
}

impl StartArgs {
    pub fn run(mut self) -> Result<()> {
        // Overwrite arguments if profile is provided.
        if let Some(name) = &self.profile {
            let mut profiles = Profiles::new();
            let profile = profiles.get_profile(name)?;
            let force = self.force;
            self = self.overwrite_args(profile.into());
            self.force = force;
        }

        let Some(path) = &self.exec_path else {
            return Err(anyhow!("Executable path not provided."));
        };

        let exec_path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(err) => {
                return Err(anyhow!(
                    "Error retrieving absolute file path of executable: {err}. Path: '{path}'."
                ))
            }
        };

        let name = &self.name.clone();

        // If name is not provided, use the file name.
        let name = match name {
            Some(name) => name,
            None => exec_path
                .file_stem()
                .expect("File name not found.")
                .to_str()
                .expect("Failed converting OsStr."),
        };

        if name.contains(char::is_whitespace) {
            return Err(anyhow!("Name contains whitespace."));
        }

        let init_system = Service::get(Some(name));

        let stdin = PathBuf::from(APPS_DIR).join(name).join("stdin");
        if stdin.exists() {
            // An application with the same '<name>' has a stdin file created.
            // Check if a service with the name 'cres.<name>.service' is already running.
            if init_system.is_running()? {
                // If running, avoid creating/starting a new service.
                return Err(anyhow!(
                    "A service with the same name '{name}' is already running."
                ));
            }
        }

        let script = init_system.get_scripts_paths();
        let script_path = Path::new(&script[0]);
        if !script_path.exists() || self.force {
            let exec_cmd = self.format_exec_cmd(&exec_path);
            eprintln!("CMD: '{exec_cmd}'");

            init_system.create(&exec_cmd)?;
            eprintln!("Service '{name}' created");
        }

        init_system.start()?;
        println!("Service '{name}' started");
        Ok(())
    }

    fn format_exec_cmd(self, exec_path: &Path) -> String {
        let mut exec_cmd = Vec::new();

        if let Some(interpreter) = self.interpreter {
            exec_cmd.push(interpreter);
        };

        exec_cmd.push(exec_path.to_string_lossy().to_string());

        if let Some(arguments) = self.arguments {
            exec_cmd.push(arguments);
        }

        exec_cmd.join(" ")
    }

    fn overwrite_args(self, loaded_args: Self) -> Self {
        let overwrite_value = |set: Option<String>, loaded: Option<String>| match set {
            // If the field is set, it takes precedence over the loaded field.
            // This is useful when the loaded field is None and we want to keep it that way.
            Some(field) => Some(field),
            None => match loaded {
                Some(path) => Some(path),
                None => set,
            },
        };

        let name = overwrite_value(self.name, loaded_args.name);
        let exec_path = overwrite_value(self.exec_path, loaded_args.exec_path);
        let interpreter = overwrite_value(self.interpreter, loaded_args.interpreter);
        let arguments = overwrite_value(self.arguments, loaded_args.arguments);

        Self {
            exec_path,
            name,
            interpreter,
            arguments,

            // These fields will be ignored.
            force: false,
            profile: None,
        }
    }
}
