use crate::{
    service::{InitSystem, Service},
    util,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::Args;
use sysinfo::{Pid, System};

#[derive(Args)]
#[command(about = "Request and print the status of a service")]
pub struct StatusArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(
        short,
        long,
        help = "Prints the output of the 'status' command without any modification"
    )]
    pub raw: bool,
}

impl StatusArgs {
    pub fn run(self) -> Result<()> {
        let mut init_system = Service::get_init_system();
        init_system.set_service_name(&self.name);
        let status = init_system.status(self.raw)?;
        if self.raw {
            return Ok(());
        }

        if !status.active.starts_with("active") {
            println!("Script: {}", status.script);
            println!("Status: {}", status.active);
            return Err(anyhow!(
                "Service 'cres.{}.service' is not running",
                self.name
            ));
        }

        let pid = Pid::from_u32(status.pid);

        let mut system = System::new();
        system.refresh_process(pid);
        system.refresh_memory();

        util::print_title_cyan("Application information");

        util::println_field_white("Name", self.name);
        util::println_field_white("Status", status.active);
        util::println_field_white("Script", status.script);
        util::println_field_white("Stdin", status.stdin);

        println!();

        util::print_title_cyan("Service information");
        util::println_field_white("PID", pid);
        util::println_field_white("CMD", status.cmd);

        match system.process(pid) {
            Some(process) => {
                let utc = Utc
                    .timestamp_opt(process.start_time().try_into().unwrap(), 0)
                    .unwrap();
                let start_time: DateTime<Local> = DateTime::from(utc);

                util::println_field_white("Started", start_time);

                util::println_field_white(
                    "Uptime",
                    util::get_uptime_from_seconds(process.run_time()),
                );

                let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;
                let cpu_count = system.physical_core_count().unwrap() as f32;

                util::println_field_white("CPU", format!("{:.2}", process.cpu_usage() / cpu_count));

                util::println_field_white(
                    "Memory",
                    format!("{:.2}% ({} Mb)", memory, process.memory() / 1024 / 1024),
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
