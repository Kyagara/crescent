use crate::{
    directory::{app_already_exist, application_dir_by_name},
    process::Application,
};
use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Starts an application from the file path provided.")]
pub struct StartArgs {
    pub file_path: String,
    #[arg(short = 'n', long = "name", help = "Application name")]
    pub name: Option<String>,
    #[arg(
        short = 'c',
        long = "command",
        help = "Example: node, python, java (will add a -jar argument automatically)."
    )]
    pub command: Option<String>,
    #[arg(
        short = 'a',
        long = "arguments",
        help = "Arguments for the application. Example: -a \"-Xms10G -Xmx10G\"",
        allow_hyphen_values = true
    )]
    pub arguments: Option<String>,
}

impl StartArgs {
    pub fn run(
        file_path: String,
        name: Option<String>,
        command: Option<String>,
        arguments: Option<String>,
    ) -> Result<()> {
        let application = Application::new(file_path, name, command, arguments)?;

        if app_already_exist(application.name.clone()) {
            return Err(anyhow!(
                "An application with the same name is already running."
            ));
        }

        application_dir_by_name(application.name.clone())?;

        application.start()?;

        Ok(())
    }
}
