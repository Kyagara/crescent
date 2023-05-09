use crate::{
    directory::{app_already_exist, application_dir_by_name},
    process::Application,
};
use anyhow::{anyhow, Context, Result};
use chrono::Local;
use clap::Args;
use daemonize::Daemonize;
use log::{info, Level, LevelFilter, Metadata, Record};
use std::fs::{self, File};

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

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "[{}] [crescent] {} - {}",
                Local::now().time().format("%H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

impl StartArgs {
    pub fn run(self) -> Result<()> {
        let application =
            Application::new(self.file_path, self.name, self.interpreter, self.arguments)?;

        if app_already_exist(&application.name)? {
            return Err(anyhow!(
                "An application with the same name is already running."
            ));
        }

        let app_path = application_dir_by_name(&application.name)?;

        if app_path.exists() {
            fs::remove_dir_all(&app_path).context("Error reseting application directory.")?;
        }

        fs::create_dir_all(&app_path).context("Error creating application directory.")?;

        drop(app_path);

        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info);

        println!("Starting application.");

        {
            let log = File::create(application.app_dir.join(application.name.clone() + ".log"))?;

            println!("Starting daemon.");

            let daemonize = Daemonize::new()
                .pid_file(application.app_dir.join(application.name.clone() + ".pid"))
                .working_directory(&application.work_dir)
                .stderr(log);

            daemonize.start()?;

            info!("Daemon started.");
        }

        application.start()?;

        Ok(())
    }
}
