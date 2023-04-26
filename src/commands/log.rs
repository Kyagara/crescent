use crate::{directory, tail::Tail};
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Outputs the `.log` file from an application.")]
pub struct LogArgs {
    #[arg(help = "The application name")]
    pub name: String,
    #[arg(short, long, help = "Lines to print. Defaults to 200")]
    pub lines: Option<usize>,
}

impl LogArgs {
    pub fn run(name: String, lines: Option<usize>) -> Result<()> {
        let mut app_dir = directory::application_dir_by_name(name.clone())?;

        app_dir.push(name + ".log");

        let read_lines = lines.unwrap_or(200);

        let mut log = Tail::new(app_dir)?;

        if log.length == 0 {
            println!("Log is empty at the moment.")
        } else {
            for line in log.read_lines(read_lines)? {
                println!("{line}")
            }
        }

        println!("Watching log");

        log.watch()?;

        Ok(())
    }
}
