use crate::{application, subprocess};
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use clap::Args;
use crossterm::style::Stylize;
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Args)]
#[command(about = "Prints information about an application.")]
pub struct StatusArgs {
    #[arg(help = "The application name.")]
    pub name: String,
}

impl StatusArgs {
    pub fn run(self) -> Result<()> {
        let app_dir = application::app_dir_by_name(&self.name)?;

        if !app_dir.exists() {
            return Err(anyhow!("Application does not exist."));
        }

        let pids = application::app_pids_by_name(&self.name)?;

        let subprocess_pid = pids[1];

        let mut system = System::new();
        system.refresh_process(subprocess_pid);
        system.refresh_memory();

        if let Some((name, args)) = subprocess::get_app_process_envs(&subprocess_pid)? {
            println!("{}", "Application information:".bold().cyan());
            println!("{} {}", "Name:".white(), name);
            println!("{} \"{}\"", "Arguments:".white(), args);
            println!("{} {}", "Crescent PID:".white(), pids[0]);

            match system.process(subprocess_pid) {
                Some(process) => {
                    let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;
                    let start_time = NaiveDateTime::from_timestamp_opt(
                        process.start_time().try_into().unwrap(),
                        0,
                    )
                    .unwrap();
                    let cpu_count = system.physical_core_count().unwrap() as f32;

                    println!();
                    println!("{}", "Subprocess information:".bold().cyan());

                    println!("{} {}", "Subprocess PID:".white(), subprocess_pid);
                    println!("{} {:?}", "CWD:".white(), process.cwd());
                    println!(
                        "{} \"{}\"",
                        "Full command line:".white(),
                        process.cmd().join(" ")
                    );

                    println!(
                        "{} {:.2}%",
                        "CPU usage:".white(),
                        process.cpu_usage() / cpu_count,
                    );
                    println!(
                        "{} {:.2}% ({} Mb)",
                        "Memory usage:".white(),
                        memory,
                        process.memory() / 1024 / 1024
                    );

                    println!("{} {}", "Started at:".white(), start_time);
                    println!("{} {}s", "Uptime:".white(), process.run_time());
                }
                None => return Err(anyhow!("Process does not exist.")),
            }

            return Ok(());
        }

        println!("Application not running.");

        Ok(())
    }
}
