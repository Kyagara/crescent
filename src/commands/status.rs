use crate::{application, util};
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

        let status = application::get_app_status(&self.name)?;

        println!("{}", "Application information:".bold().cyan());
        println!("{} {}", "Name:".white(), status.name);
        println!(
            "{} \"{}\"",
            "Interpreter arguments:".white(),
            status.interpreter_args.join(" ")
        );
        println!(
            "{} \"{}\"",
            "Application arguments:".white(),
            status.application_args.join(" ")
        );
        println!("{} {}", "Profile:".white(), status.profile);
        println!("{} {}", "crescent PID:".white(), pids[0]);
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
                status.cmd.join(" ")
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
            println!(
                "{} {}",
                "Uptime:".white(),
                util::get_uptime_from_seconds(process.run_time())
            );
        }

        Ok(())
    }
}
