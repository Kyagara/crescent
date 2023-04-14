use crate::process::{app_already_exist, Application};
use clap::Args;
use std::{fs, process::exit};

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
    ) {
        let application = Application::new(file_path, name, command, arguments);

        if app_already_exist(application.name.clone()) {
            eprintln!("An application with the same name is already running.");
            exit(1)
        }

        match application.temp_dir.is_dir() {
            true => fs::remove_dir_all(&application.temp_dir).unwrap(),
            false => fs::create_dir_all(&application.temp_dir).unwrap(),
        }

        fs::create_dir_all(&application.temp_dir).unwrap();

        application.start();
    }
}
