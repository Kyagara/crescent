use crate::{application, tail::Tail};
use anyhow::{anyhow, Result};
use clap::Args;
use crossbeam::channel::unbounded;
use std::thread;

#[derive(Args)]
#[command(about = "Print or watch the '.log' file from an application.")]
pub struct LogArgs {
    #[arg(help = "The application name")]
    pub name: String,
    #[arg(short, long, help = "Lines to print. Defaults to 200")]
    pub lines: Option<usize>,
    #[arg(
        short,
        long,
        help = "Watches the file for any modification, printing new lines."
    )]
    pub follow: bool,
}

impl LogArgs {
    pub fn run(self) -> Result<()> {
        let mut app_dir = application::app_dir_by_name(&self.name)?;

        if !app_dir.exists() {
            return Err(anyhow!("Application does not exist."));
        }

        app_dir.push(self.name + ".log");

        let read_lines = self.lines.unwrap_or(200);

        let mut log = Tail::new(app_dir)?;

        if log.length == 0 {
            println!("Log is empty at the moment.")
        } else {
            let mut count = 0;

            log.read_lines(read_lines)?
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

        thread::spawn(move || {
            log.watch(&sender).unwrap();
        });

        for content in receiver {
            print!("{content}")
        }

        Ok(())
    }
}
