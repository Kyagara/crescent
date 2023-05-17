use crate::{application, subprocess};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::Args;
use crossterm::style::Stylize;
use std::println;
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Args)]
#[command(about = "Print information about an application.")]
pub struct StatusArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

impl StatusArgs {
    pub fn run(self) -> Result<()> {
        let app_dir = application::app_dir_by_name(&self.name)?;

        if !app_dir.exists() {
            return Err(anyhow!("Application does not exist."));
        }

        let pids = application::app_pids_by_name(&self.name)?;

        if pids.len() < 2 {
            println!("Application not running.");
            return Ok(());
        }

        let subprocess_pid = pids[1];

        let mut system = System::new();
        system.refresh_process(subprocess_pid);
        system.refresh_memory();

        if let Some((name, interpreter_args, application_args, profile)) =
            subprocess::get_app_process_envs(&subprocess_pid)?
        {
            println!("{}", "Application information:".bold().cyan());
            println!("{} {}", "Name:".white(), name);
            println!(
                "{} \"{}\"",
                "Interpreter arguments:".white(),
                interpreter_args
            );
            println!(
                "{} \"{}\"",
                "Application arguments:".white(),
                application_args
            );
            println!("{} \"{}\"", "Profile:".white(), profile);
            println!("{} {}", "Crescent PID:".white(), pids[0]);
            println!();

            if let Some(process) = system.process(subprocess_pid) {
                let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;
                let cpu_count = system.physical_core_count().unwrap() as f32;
                let utc = Utc
                    .timestamp_opt(process.start_time().try_into().unwrap(), 0)
                    .unwrap();
                let start_time: DateTime<Local> = DateTime::from(utc);

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

            return Ok(());
        }

        println!("Application not running.");

        Ok(())
    }
}
