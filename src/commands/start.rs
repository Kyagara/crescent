use crate::{application, logger::Logger, subprocess};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use daemonize::Daemonize;
use log::{info, LevelFilter};
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[derive(Args)]
#[command(about = "Starts an application from the file path provided.")]
pub struct StartArgs {
    pub file_path: String,
    #[arg(short = 'n', long = "name", help = "The application name")]
    pub name: Option<String>,
    #[arg(
        short = 'i',
        long = "interpreter",
        help = "Example: node, python3, java (will add a -jar argument automatically)."
    )]
    pub interpreter: Option<String>,
    #[arg(
        short = 'a',
        long = "arguments",
        help = "Arguments for the application. Example: -a \"-Xms10G -Xmx10G\"",
        allow_hyphen_values = true
    )]
    pub arguments: Option<String>,
}

static LOGGER: Logger = Logger;

impl StartArgs {
    pub fn run(self) -> Result<()> {
        let file_path = fs::canonicalize(&self.file_path)?;

        if !file_path.exists() {
            return Err(anyhow!(format!(
                "File '{}' not found",
                &file_path.to_string_lossy()
            )));
        }

        let name = match self.name {
            Some(name) => name,
            None => file_path.file_stem().unwrap().to_str().unwrap().to_string(),
        };

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

        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info);

        println!("Starting application.");

        {
            let log = File::create(app_dir.join(name.clone() + ".log"))?;
            let pid_path = app_dir.join(name.clone() + ".pid");
            let work_dir = file_path.parent().unwrap().to_path_buf();

            println!("Starting daemon.");

            let daemonize = Daemonize::new()
                .pid_file(pid_path)
                .working_directory(work_dir)
                .stderr(log);

            daemonize.start()?;

            info!("Daemon started.");
        }

        let mut interpreter = self.interpreter.unwrap_or(String::new());

        let args = get_args(&mut interpreter, file_path);

        drop(app_dir);

        subprocess::start(&name, &interpreter, &args)?;

        Ok(())
    }
}

fn get_args(interpreter: &mut String, file_path: PathBuf) -> Vec<String> {
    let file_path_str = file_path.to_str().unwrap().to_string();

    let mut args = vec![];

    match interpreter.as_str() {
        "java" => {
            *interpreter = "java".to_string();
            args.push(String::from("-jar"));
            args.push(file_path_str)
        }
        "" => *interpreter = file_path_str,
        _ => args.push(file_path_str),
    }

    args
}
