use std::vec;

use crate::{
    service::{InitSystem, Service, StatusOutput},
    util,
};

use anyhow::Result;
use clap::Args;
use sysinfo::{Pid, System};

#[derive(Args)]
#[command(about = "List services created with basic information")]
pub struct ListArgs;

impl ListArgs {
    pub fn run() -> Result<()> {
        let mut init_system = Service::get(None);
        let list = init_system.list()?;

        let mut system = System::new();
        system.refresh_processes();

        let mut services = vec![];

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
                String::from("N/A"),
            ];

            init_system.update_application_name(&service);
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

            row[4] = init_system.is_enabled()?.to_string();
            services.push(row);
        }

        if services.is_empty() {
            println!("No services found.");
            return Ok(());
        }

        let headers = ["Name", "PID", "Status", "Uptime", "Enabled"];
        let mut max_widths: Vec<usize> = vec![4, 3, 6, 6, 7];

        // Get the max width for each column
        for service in &services {
            for (i, value) in service.iter().enumerate() {
                if value.len() > max_widths[i] {
                    max_widths[i] = value.len();
                }
            }
        }

        // Print the headers
        for (i, header) in headers.iter().enumerate() {
            print!("{:<width$}   ", header, width = max_widths[i]);
        }

        println!();

        // Print the separator
        for max_width in &max_widths {
            print!("{:-<width$} | ", "", width = max_width);
        }

        println!();

        // Print the rows
        for service in services {
            for (i, value) in service.iter().enumerate() {
                print!("{:<width$}   ", value, width = max_widths[i]);
            }
            println!();
        }

        Ok(())
    }
}
