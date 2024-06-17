use std::vec;

use crate::{
    service::{InitSystem, Service, StatusOutput},
    util,
};

use anyhow::Result;
use clap::Args;
use sysinfo::{Pid, System};
use tabled::builder::Builder;

#[derive(Args)]
#[command(about = "List services created with basic information")]
pub struct ListArgs;

impl ListArgs {
    pub fn run() -> Result<()> {
        let mut init_system = Service::get_init_system();
        let list = init_system.list()?;

        let mut system = System::new();
        system.refresh_processes();

        let mut builder = Builder::default();
        builder.push_record(["Name", "PID", "Status", "Uptime"]);

        let mut index = 0;
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

            // No need to check other types of output.
            if let StatusOutput::Pretty(status) = status {
                let process = system.process(Pid::from_u32(status.pid));

                let uptime = match process {
                    Some(process) => util::get_uptime_from_seconds(process.run_time()),
                    None => String::from("N/A"),
                };

                row[1] = status.pid.to_string();
                row[2] = status.active;
                row[3] = uptime;
            }

            builder.push_record(row);
            index += 1;
        }

        if index == 0 {
            println!("No services found.");
            return Ok(());
        }

        let table = builder.build();
        println!("{table}");
        Ok(())
    }
}
