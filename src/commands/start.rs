use crate::{application, crescent, logger, subprocess};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use daemonize::Daemonize;
use log::{info, LevelFilter};
use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

#[derive(Args, Deserialize, Clone)]
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

static LOGGER: logger::Logger = logger::Logger;

impl StartArgs {
    pub fn run(mut self) -> Result<()> {
        let mut profile_path = PathBuf::new();

        if let Some(profile) = &self.profile {
            profile_path = crescent::get_profile_path(profile.to_string())?;
            let string = fs::read_to_string(&profile_path)?;
            let args: StartArgs = serde_json::from_str(&string)?;
            self = self.overwrite_args(args)?;
        }

        let file_path = fs::canonicalize(self.file_path.clone().unwrap())?;

        if !file_path.exists() && file_path.is_file() {
            return Err(anyhow!(format!(
                "File '{}' not found.",
                &file_path.to_string_lossy()
            )));
        }

        let name = match &self.name {
            Some(name) => name.to_string(),
            None => file_path.file_stem().unwrap().to_str().unwrap().to_string(),
        };

        if name.contains(char::is_whitespace) {
            return Err(anyhow!("Name can't contain whitespace."));
        }

        if application::app_already_running(&name)? {
            return Err(anyhow!(
                "An application with the same name is already running."
            ));
        }

        let app_dir = application::app_dir_by_name(&name)?;

        if app_dir.exists() {
            fs::remove_dir_all(&app_dir).context("Error reseting application directory.")?;
        }

        fs::create_dir_all(&app_dir).context("Error creating application directory.")?;

        let (mut interpreter_args, mut application_args) =
            self.create_subprocess_arguments(&file_path);

        // The subprocess inherits all environment variables
        env::set_var("CRESCENT_APP_NAME", &name);
        env::set_var("CRESCENT_APP_INTERPRETER_ARGS", interpreter_args.join(" "));
        env::set_var("CRESCENT_APP_ARGS", application_args.join(" "));
        env::set_var("CRESCENT_APP_PROFILE", &profile_path);

        drop(profile_path);

        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info);

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

        interpreter_args.append(&mut application_args);

        subprocess::start(name, interpreter_args, app_dir)?;

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
