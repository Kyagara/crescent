use crate::{
    directory::{app_already_exist, application_dir_by_name},
    process::Application,
};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::fs;

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

impl StartArgs {
    pub fn run(self) -> Result<()> {
        let application =
            Application::new(self.file_path, self.name, self.interpreter, self.arguments)?;

        if app_already_exist(&application.name) {
            return Err(anyhow!(
                "An application with the same name is already running."
            ));
        }

        let app_path = application_dir_by_name(&application.name)?;

        if app_path.exists() {
            fs::remove_dir_all(&app_path).context("Error reseting application directory.")?;
        }

        fs::create_dir_all(&app_path).context("Error creating application directory.")?;

        application.start()?;

        Ok(())
    }
}
