use super::save::SaveFile;
use crate::{
    application::{self, Application},
    crescent::{self, Profile},
    logger, subprocess,
    util::{self, print_title_cyan},
};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use daemonize::Daemonize;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::Path,
};

#[derive(Args, Clone, Serialize, Deserialize, Default)]
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

    #[arg(short, long, help = "Start all saved apps.")]
    pub saved: bool,
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
            saved: false,
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

        if self.saved {
            return start_saved();
        }

        let stop_command = match &self.profile {
            Some(profile_str) => {
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

        let start_args = self.clone();

        let (interpreter_args, application_args) = self.create_subprocess_arguments(&file_path);

        let cmd: Vec<String>;
        {
            let mut i_args = interpreter_args;
            let mut a_args = application_args;
            i_args.append(&mut a_args);
            cmd = i_args
        }

        let app_info = Application {
            name,
            cmd,
            file_path,
            stop_command,
            start_args,
        };

        start(app_info)
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
            profile: None,
            saved: false,
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

fn start_saved() -> Result<()> {
    let mut save_dir = crescent::crescent_dir()?;
    save_dir.push("apps.json");
    let file = File::open(save_dir)?;
    let save_file: SaveFile = serde_json::from_reader(file)?;

    if save_file.apps.is_empty() {
        return Err(anyhow!("List of apps is empty."));
    }

    print_title_cyan("Starting applications.");

    for app_info in save_file.apps {
        if application::app_already_running(&app_info.name)? {
            println!(
                "An application with the name `{}` is already running. Skipping.",
                app_info.name
            );

            continue;
        }

        let exec_path = util::get_exec_path();
        let mut cmd = util::get_base_command(exec_path);
        let mut cmd_args = vec![];

        cmd_args.push("start".to_string());
        cmd_args.push(app_info.file_path.to_str().unwrap().to_string());

        if let Some(name) = app_info.start_args.name {
            cmd_args.push("--name".to_string());
            cmd_args.push(name);
        }

        if let Some(interpreter) = app_info.start_args.interpreter {
            cmd_args.push("--interpreter".to_string());
            cmd_args.push(interpreter);
        }

        if let Some(args) = app_info.start_args.interpreter_arguments {
            cmd_args.push("--interpreter-args".to_string());
            cmd_args.push(args.join(" "));
        }

        if let Some(args) = app_info.start_args.application_arguments {
            cmd_args.push("--arguments".to_string());
            cmd_args.push(args.join(" "));
        }

        if let Some(profile) = app_info.start_args.profile {
            cmd_args.push("--profile".to_string());
            cmd_args.push(profile);
        }

        cmd.args(cmd_args).spawn()?;
    }

    Ok(())
}

pub fn start(app_info: Application) -> Result<()> {
    let app_dir = application::app_dir_by_name(&app_info.name)?;

    if app_dir.exists() {
        fs::remove_dir_all(&app_dir).context("Error resetting application directory.")?;
    }

    fs::create_dir_all(&app_dir).context("Error creating application directory.")?;

    eprintln!("Starting `{}` application.", app_info.name);

    {
        let log = File::create(app_dir.join(app_info.name.clone() + ".log"))?;
        let pid_path = app_dir.join(app_info.name.clone() + ".pid");
        let work_dir = app_info.file_path.parent().unwrap().to_path_buf();

        let daemonize = Daemonize::new()
            .pid_file(pid_path)
            .working_directory(work_dir)
            .stderr(log);

        daemonize.start()?;
    }

    subprocess::start(app_info, app_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test_utils;
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
            saved: false,
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
            saved: false,
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
            saved: false,
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
