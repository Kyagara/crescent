use std::vec;

use crate::{
    service::{InitSystem, Service},
    util,
};

use anyhow::Result;
use clap::Args;
use sysinfo::{Pid, System};
use tabled::{Table, Tabled};

#[derive(Args)]
#[command(about = "List services created")]
pub struct ListArgs;

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "PID")]
    pid: u32,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

impl ListArgs {
    pub fn run() -> Result<()> {
        let mut init_system = Service::get_init_system();
        let list = init_system.list()?;

        let mut system = System::new();
        system.refresh_processes();

        let mut services = vec![];

        for service in list {
            let service = service
                .trim_start_matches("cres.")
                .trim_end_matches(".service")
                .to_string();

            let mut app = Row {
                name: service.clone(),
                pid: 0,
                status: String::from("N/A"),
                uptime: String::from("N/A"),
            };

            init_system.set_service_name(&service);
            let status = init_system.status(false)?;

            let process = system.process(Pid::from_u32(status.pid));

            let uptime = match process {
                Some(process) => util::get_uptime_from_seconds(process.run_time()),
                None => String::from("N/A"),
            };

            app.pid = status.pid;
            app.status = status.active;
            app.uptime = uptime;

            services.push(app);
        }

        let table = Table::new(services);
        println!("{table}");
        Ok(())
    }
}
