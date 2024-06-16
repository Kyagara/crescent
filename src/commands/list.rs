use std::vec;

use crate::{
    service::{InitSystem, Service},
    util,
};

use anyhow::Result;
use clap::Args;
use sysinfo::{Pid, System};
use tabled::builder::Builder;

#[derive(Args)]
#[command(about = "List services created")]
pub struct ListArgs;

impl ListArgs {
    pub fn run() -> Result<()> {
        let mut init_system = Service::get_init_system();
        let list = init_system.list()?;

        let mut system = System::new();
        system.refresh_processes();

        let mut builder = Builder::default();
        builder.push_record(["Name", "PID", "Status", "Uptime"]);

        for service in list {
            let service = service
                .trim_start_matches("cres.")
                .trim_end_matches(".service")
                .to_string();

            let mut row = vec![
                service.clone(),
                String::from("N/A"),
                String::from("N/A"),
                String::from("N/A"),
            ];

            init_system.set_service_name(&service);
            let status = init_system.status(false)?;

            let process = system.process(Pid::from_u32(status.pid));

            let uptime = match process {
                Some(process) => util::get_uptime_from_seconds(process.run_time()),
                None => String::from("N/A"),
            };

            row[1] = status.pid.to_string();
            row[2] = status.active;
            row[3] = uptime;

            builder.push_record(row);
        }

        let table = builder.build();
        println!("{table}");
        Ok(())
    }
}
