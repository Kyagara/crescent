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

#[derive(Args, Deserialize)]
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
        help = "node, python3, java (will add a -jar argument automatically). Not needed if file path is an executable."
    )]
    pub interpreter: Option<String>,
    #[arg(
        short = 'a',
        long = "arguments",
        help = "Arguments for the subprocess. Example: -a \"-Xms10G -Xmx10G.\"",
        allow_hyphen_values = true
    )]
    pub arguments: Option<Vec<String>>,
    #[arg(
        short = 'p',
        long = "profile",
        help = "Name of the profile to load fields from."
    )]
    pub profile: Option<String>,
}

static LOGGER: logger::Logger = logger::Logger;

impl StartArgs {
    pub fn run(mut self) -> Result<()> {
        let mut profile_path = PathBuf::new();

        if let Some(profile) = self.profile {
            profile_path = crescent::get_profile_path(profile)?;
            let string = fs::read_to_string(&profile_path)?;
            let args: StartArgs = serde_json::from_str(&string)?;
            self = args;
        }

        let file_path = fs::canonicalize(self.file_path.unwrap())?;

        if !file_path.exists() {
            return Err(anyhow!(format!(
                "File '{}' not found.",
                &file_path.to_string_lossy()
            )));
        }

        let name = match self.name {
            Some(name) => name,
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
            fs::remove_dir_all(&app_dir).context("Error reseting application directory.")?;
        }

        fs::create_dir_all(&app_dir).context("Error creating application directory.")?;

        let mut interpreter = self.interpreter.unwrap_or(String::new());

        let mut args = check_interpreter_and_executable(&mut interpreter, &file_path);

        let command_args = match &self.arguments {
            Some(arguments) => {
                args.push(arguments.join(" "));
                arguments.join(" ")
            }
            None => String::new(),
        };

        // The subprocess inherits all environment variables
        env::set_var("CRESCENT_APP_NAME", &name);
        env::set_var("CRESCENT_APP_ARGS", &command_args);
        env::set_var("CRESCENT_APP_PROFILE", &profile_path);

        drop(command_args);
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

        subprocess::start(name, interpreter, args, app_dir)?;

        Ok(())
    }
}

fn check_interpreter_and_executable(interpreter: &mut String, exec_path: &Path) -> Vec<String> {
    let exec_path_str = exec_path.to_str().unwrap().to_string();

    let mut args = vec![];

    match interpreter.as_str() {
        "java" => {
            *interpreter = "java".to_string();
            args.push(String::from("-jar"));
            args.push(exec_path_str)
        }
        "" => *interpreter = exec_path_str,
        _ => args.push(exec_path_str),
    }

    args
}
