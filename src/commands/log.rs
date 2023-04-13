use crate::process;
use clap::Args;
use std::fs;

#[derive(Args)]
#[command(about = "Outputs the `.log` file from an application.")]
pub struct LogArgs {
    #[arg(help = "The application name")]
    pub name: String,
    #[arg(short, long, help = "Lines to print. Defaults to 200")]
    pub lines: Option<usize>,
}

impl LogArgs {
    pub fn run(name: String, lines: Option<usize>) {
        let mut temp_dir = process::application_temp_dir_by_name(name.clone());

        temp_dir.push(name + ".log");

        let log_file = fs::read_to_string(temp_dir).unwrap();

        let read_lines = lines.unwrap_or(200);

        let mut log = vec![];

        for (index, line) in log_file.lines().rev().enumerate() {
            if index >= read_lines {
                break;
            }

            log.insert(0, line);
        }

        for line in log {
            println!("{line}");
        }
    }
}
