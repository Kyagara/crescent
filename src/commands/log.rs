use crate::logger::{LogSystem, Logger};

use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Print or follow the logs from a service")]
pub struct LogArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(short, long, help = "Lines to print", default_value_t = 200)]
    pub lines: u64,

    #[arg(short, long, help = "Follow the log for any new lines")]
    pub follow: bool,
}

impl LogArgs {
    pub fn run(self) -> Result<()> {
        let mut logger = Logger::get_log_system();
        logger.set_service_name(&self.name);

        if self.follow {
            logger.follow()?;
            return Ok(());
        }

        let lines = logger.log(self.lines)?;

        for line in lines {
            eprintln!("{line}");
        }

        Ok(())
    }
}
