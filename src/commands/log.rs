use crate::process;
use clap::Args;
use std::fs;

#[derive(Args)]
#[command(about = "Outputs the `.log` file from an application.")]
pub struct LogArgs {
    #[arg(help = "The application name")]
    pub name: String,
}

impl LogArgs {
    pub fn run(name: String) {
        let mut temp_dir = process::application_temp_dir_by_name(name.clone());

        temp_dir.push(name + ".log");

        let log = fs::read_to_string(temp_dir).unwrap();

        println!("{log}");
    }
}
