use std::io::{BufRead, BufReader};

use crate::{
    application::Application,
    logger::{LogSystem, Logger},
};

use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Print or follow the logs from a service")]
pub struct LogArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(help = "Lines to print", short, long, default_value_t = 200)]
    pub lines: u64,

    #[arg(help = "Follow the log for any new lines", short, long)]
    pub follow: bool,
}

impl LogArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(&self.name);
        application.exists()?;

        let logger = Logger::get(application.name);

        if self.follow {
            let process = logger.follow()?;
            let stdout = process.stdout.expect("Failed to capture stdout");

            let reader = BufReader::new(stdout);

            for line in reader.lines().map_while(Result::ok) {
                println!("{line}");
            }

            return Ok(());
        }

        let stdout = logger.log(self.lines)?;
        print!("{stdout}");
        Ok(())
    }
}
