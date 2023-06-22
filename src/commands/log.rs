use crate::{application, tail};
use anyhow::{anyhow, Result};
use clap::Args;
use crossbeam::channel::unbounded;
use std::{fs::OpenOptions, thread};

#[derive(Args)]
#[command(about = "Print, watch or flush the log file from an application.")]
pub struct LogArgs {
    #[arg(help = "Application name.")]
    pub name: String,

    #[arg(
        short,
        long,
        help = "Lines to print. Defaults to 200.",
        default_value_t = 200
    )]
    pub lines: usize,

    #[arg(short, long, help = "Keep watching the log for any new lines.")]
    pub follow: bool,

    #[arg(long, help = "Flush the application log.")]
    pub flush: bool,
}

impl LogArgs {
    pub fn run(self) -> Result<()> {
        application::check_app_exists(&self.name)?;

        let mut app_dir = application::app_dir_by_name(&self.name)?;

        app_dir.push(format!("{}.log", self.name));

        if !app_dir.is_file() {
            return Err(anyhow!("Log file does not exist."));
        }

        if self.flush {
            match OpenOptions::new().write(true).truncate(true).open(app_dir) {
                Ok(_) => {
                    println!("Flushed '{}' log file.", self.name);
                    return Ok(());
                }
                Err(err) => {
                    return Err(anyhow!("Error flushing log file: {err:}"));
                }
            }
        }

        let mut log = tail::Tail::new(app_dir)?;

        if log.length == 0 {
            println!("Log is empty at the moment.")
        } else {
            let mut count = 0;

            log.read_lines(self.lines)?
                .into_iter()
                .enumerate()
                .for_each(|(i, line)| {
                    println!("{line}");
                    count = i;
                });

            println!(">> Printed {} lines", count)
        }

        if !self.follow {
            return Ok(());
        }

        println!(">> Watching log");

        let (sender, receiver) = unbounded();

        thread::spawn(move || log.watch(&sender));

        for content in receiver {
            print!("{content}")
        }

        Ok(())
    }
}
