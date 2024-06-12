use std::println;

use crate::{application, util};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::Args;
use sysinfo::System;

#[derive(Args)]
#[command(about = "Print information about an application.")]
pub struct StatusArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

impl StatusArgs {
    pub fn run(self) -> Result<()> {
        application::check_app_exists(&self.name)?;

        let pids = application::app_pids_by_name(&self.name)?;

        let status = application::get_app_info(&self.name)?;

        let subprocess_pid = pids[1];

        let mut system = System::new();
        system.refresh_process(subprocess_pid);
        system.refresh_memory();

        let i_args = match status.start_args.interpreter_arguments {
            Some(args) => args.join(" "),
            None => String::new(),
        };

        let a_args = match status.start_args.application_arguments {
            Some(args) => args.join(" "),
            None => String::new(),
        };

        util::print_title_cyan("Application information");

        util::println_field_white("crescent PID", pids[0]);
        util::println_field_white("Name", status.name);
        util::println_field_white("Interpreter arguments", i_args);
        util::println_field_white("Application arguments", a_args);
        util::println_field_white(
            "Profile",
            status.start_args.profile.unwrap_or(String::new()),
        );

        println!();

        util::print_title_cyan("Subprocess information");
        util::println_field_white("PID", subprocess_pid);
        util::println_field_white("Full command line", status.cmd.join(" "));

        match system.process(subprocess_pid) {
            Some(process) => {
                let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;
                let cpu_count = system.physical_core_count().unwrap() as f32;
                let utc = Utc
                    .timestamp_opt(process.start_time().try_into().unwrap(), 0)
                    .unwrap();
                let start_time: DateTime<Local> = DateTime::from(utc);

                let cwd = match process.cwd() {
                    Some(cwd) => cwd.to_string_lossy().to_string(),
                    None => String::from("N/A"),
                };

                util::println_field_white("CWD", cwd);
                util::println_field_white(
                    "CPU usage",
                    format!("{:.2}", process.cpu_usage() / cpu_count),
                );
                util::println_field_white(
                    "Memory usage",
                    format!("{:.2}% ({} Mb)", memory, process.memory() / 1024 / 1024),
                );
                util::println_field_white("Started at", start_time);
                util::println_field_white(
                    "Uptime",
                    util::get_uptime_from_seconds(process.run_time()),
                );
            }
            None => {
                return Err(anyhow!(
                    "Error retrieving subprocess information, process does not exist."
                ))
            }
        }

        Ok(())
    }
}
