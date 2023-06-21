use crate::{
    application::{self, ApplicationInfo},
    crescent::{self, Profile},
    logger, subprocess,
};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use daemonize::Daemonize;
use log::{info, LevelFilter};
use std::{
    fs::{self, File},
    path::Path,
};

#[derive(Args)]
#[command(about = "Start an application from the file path provided.")]
pub struct StartArgs {
    pub file_path: Option<String>,

    #[arg(
        short = 'n',
        long = "name",
        help = "Application name. Defaults to the executable name."
    )]
    pub name: Option<String>,

    #[arg(
        short = 'i',
        long = "interpreter",
        help = "node, python3, java. Not needed if file path is an executable."
    )]
    pub interpreter: Option<String>,

    #[arg(
        long = "interpreter-args",
        help = "Arguments for the interpreter. Not needed if file path is an executable.",
        allow_hyphen_values = true
    )]
    pub interpreter_arguments: Option<Vec<String>>,

    #[arg(
        short = 'a',
        long = "arguments",
        help = "Arguments for the executable. Example: -a \"-Xms10G -Xmx10G.\"",
        allow_hyphen_values = true
    )]
    pub application_arguments: Option<Vec<String>>,

    #[arg(
        short = 'p',
        long = "profile",
        help = "Name or path to the profile to load fields from."
    )]
    pub profile: Option<String>,
}

impl From<Profile> for StartArgs {
    fn from(profile: Profile) -> Self {
        Self {
            file_path: profile.file_path,
            name: profile.name,
            interpreter: profile.interpreter,
            interpreter_arguments: profile.interpreter_arguments,
            application_arguments: profile.application_arguments,
            profile: None,
        }
    }
}

static LOGGER: logger::Logger = logger::Logger;

impl StartArgs {
    pub fn run(mut self) -> Result<()> {
        if !cfg!(test) {
            log::set_logger(&LOGGER).unwrap();
            log::set_max_level(LevelFilter::Info);
        }

        let mut profile_name = String::new();

        let stop_command = match &self.profile {
            Some(profile_str) => {
                profile_name = profile_str.clone();
                let profile = crescent::get_profile(profile_str)?;
                self = self.overwrite_args(profile.clone().into())?;
                profile.stop_command
            }
            None => None,
        };

        let path = match &self.file_path {
            Some(path) => path,
            None => return Err(anyhow!("Executable path not provided.")),
        };

        let file_path = match fs::canonicalize(path) {
            Ok(path) => path,
            Err(err) => return Err(anyhow!("Error retrieving absolute file path: {err}.")),
        };

        let name = match &self.name {
            Some(name) => name.to_string(),
            None => file_path.file_stem().unwrap().to_str().unwrap().to_string(),
        };

        if name.contains(char::is_whitespace) {
            return Err(anyhow!("Name contains whitespace."));
        }

        if application::app_already_running(&name)? {
            return Err(anyhow!(
                "An application with the same name is already running."
            ));
        }

        let app_dir = application::app_dir_by_name(&name)?;

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).context("Error resetting application directory.")?;
        }

        fs::create_dir_all(&app_dir).context("Error creating application directory.")?;

        let (interpreter_args, application_args) = self.create_subprocess_arguments(&file_path);

        info!("Starting application.");

        {
            let log = File::create(app_dir.join(name.clone() + ".log"))?;
            let pid_path = app_dir.join(name.clone() + ".pid");
            let work_dir = file_path.parent().unwrap().to_path_buf();

            info!("Starting daemon.");

            let daemonize = Daemonize::new()
                .pid_file(pid_path)
                .working_directory(work_dir)
                .stderr(log);

            daemonize.start()?;

            info!("Daemon started.");
        }

        let cmd: Vec<String>;
        {
            let mut i_args = interpreter_args.clone();
            let mut a_args = application_args.clone();
            i_args.append(&mut a_args);
            cmd = i_args
        }

        let app_status = ApplicationInfo {
            name,
            interpreter_args,
            application_args,
            cmd,
            profile: profile_name,
        };

        subprocess::start(app_status, stop_command, app_dir)?;

        Ok(())
    }

    fn overwrite_args(self, loaded_args: StartArgs) -> Result<StartArgs> {
        // All other fields are optional except this one, return an error if not found.
        let file_path = match self.file_path {
            Some(field) => field,
            _ => match loaded_args.file_path {
                Some(path) => path,
                None => {
                    return Err(anyhow!(
                        "Profile does not contain a file path and one wasn't specified."
                    ))
                }
            },
        };

        let overwrite_string_value = |set: Option<String>, loaded: Option<String>| match set {
            Some(field) => Some(field),
            None => match loaded {
                Some(path) => Some(path),
                None => set,
            },
        };

        let interpreter = overwrite_string_value(self.interpreter, loaded_args.interpreter);
        let name = overwrite_string_value(self.name, loaded_args.name);

        let overwrite_vec_value = |set: Option<Vec<String>>, loaded: Option<Vec<String>>| match set
        {
            Some(field) => Some(field),
            None => match loaded {
                Some(path) => Some(path),
                None => set,
            },
        };

        let interpreter_arguments = overwrite_vec_value(
            self.interpreter_arguments,
            loaded_args.interpreter_arguments,
        );

        let application_arguments = overwrite_vec_value(
            self.application_arguments,
            loaded_args.application_arguments,
        );

        Ok(StartArgs {
            file_path: Some(file_path),
            name,
            interpreter,
            interpreter_arguments,
            application_arguments,
            profile: Some(self.profile.unwrap()),
        })
    }

    fn create_subprocess_arguments(self, exec_path: &Path) -> (Vec<String>, Vec<String>) {
        let mut interpreter_args = vec![];

        if let Some(interpreter) = self.interpreter {
            interpreter_args.push(interpreter);

            if let Some(mut arguments) = self.interpreter_arguments {
                interpreter_args.append(&mut arguments);
            }
        };

        let application_args = match self.application_arguments {
            Some(args) => args,
            None => vec![],
        };

        interpreter_args.push(exec_path.to_str().unwrap().to_string());

        (interpreter_args, application_args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_utils;
    use predicates::prelude::predicate;

    #[test]
    fn unit_start_run() -> Result<()> {
        let start_command = StartArgs {
            file_path: None,
            name: Some(String::from("name with space")),
            interpreter: None,
            interpreter_arguments: None,
            application_arguments: None,
            profile: None,
        };

        let err = start_command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Executable path not provided.");

        let start_command = StartArgs {
            file_path: Some(String::from("/does/not/exist")),
            name: Some(String::from("name with space")),
            interpreter: None,
            interpreter_arguments: None,
            application_arguments: None,
            profile: None,
        };

        let err = start_command.run().unwrap_err();
        assert_eq!(
            format!("{}", err),
            "Error retrieving absolute file path: No such file or directory (os error 2)."
        );

        let start_command = StartArgs {
            file_path: Some(String::from("./tools/long_running_service.py")),
            name: Some(String::from("name with space")),
            interpreter: None,
            interpreter_arguments: None,
            application_arguments: None,
            profile: None,
        };

        let err = start_command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Name contains whitespace.");

        let name = "duplicate_app";
        test_utils::start_long_running_service(name)?;
        assert!(test_utils::check_app_is_running(name)?);

        let mut cmd = test_utils::get_base_command();

        let args = [
            "start",
            "./tools/long_running_service.py",
            "-i",
            "python3",
            "-n",
            name,
        ];

        cmd.args(args);

        cmd.assert().failure().stderr(predicate::str::contains(
            "An application with the same name is already running.",
        ));

        test_utils::shutdown_long_running_service(name)?;
        test_utils::delete_app_folder(name)?;
        Ok(())
    }
}
