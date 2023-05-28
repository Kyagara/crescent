use crate::{application, tail};
use anyhow::{anyhow, Result};
use clap::Args;
use crossbeam::channel::unbounded;
use std::thread;

#[derive(Args)]
#[command(about = "Print or watch the '.log' file from an application.")]
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
}

impl LogArgs {
    pub fn run(self) -> Result<()> {
        let mut app_dir = application::app_dir_by_name(&self.name)?;

        if !app_dir.exists() {
            return Err(anyhow!("Application does not exist."));
        }

        app_dir.push(self.name + ".log");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_log_run() -> Result<()> {
        let command = LogArgs {
            name: "log_run".to_string(),
            lines: 200,
            follow: false,
        };

        assert_eq!(command.name, "log_run");
        assert!(!command.follow);
        assert_eq!(command.lines, 200);
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");

        Ok(())
    }
}
