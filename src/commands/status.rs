use std::process::Command;

use crate::{
    application::Application,
    service::{InitSystem, Service, StatusOutput},
    util,
};

use anyhow::{anyhow, Result};
use clap::Args;
use sysinfo::{Pid, System};

#[derive(Args)]
#[command(about = "Get information about a service")]
pub struct StatusArgs {
    #[arg(help = "Service name")]
    pub name: String,

    #[arg(
        help = "Prints the output of the `status` command without any modification",
        short,
        long
    )]
    pub raw: bool,
}

impl StatusArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(&self.name);
        application.exists()?;

        let init_system = Service::get(Some(&application.name));

        let status = init_system.status(self.raw)?;

        match status {
            StatusOutput::Raw(output) => print!("{output}"),
            StatusOutput::Pretty(status) => {
                let pid = Pid::from_u32(status.pid);

                let mut system = System::new();
                system.refresh_process(pid);
                system.refresh_memory();

                let enabled = init_system.is_enabled()?;

                util::print_title_cyan("Application information");

                util::println_field_white("Name", application.name);
                util::println_field_white("Status", status.active);
                util::println_field_white("Script", status.script);
                util::println_field_white("Stdin", status.stdin);
                util::println_field_white("Enabled", enabled);

                println!();

                util::print_title_cyan("Service information");
                util::println_field_white("PID", pid);
                util::println_field_white("CMD", status.cmd);

                match system.process(pid) {
                    Some(process) => {
                        // Using date to convert the start time into a human readable format
                        let output = Command::new("date")
                            .arg("-d")
                            .arg(format!("@{}", process.start_time()))
                            .arg("+%Y-%m-%d %H:%M:%S")
                            .output()
                            .expect("Failed to execute command");

                        let started = String::from_utf8(output.stdout).unwrap();

                        util::println_field_white("Started", started);

                        util::println_field_white(
                            "Uptime",
                            util::get_uptime_from_seconds(process.run_time()),
                        );

                        let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;
                        let cpu_count = system.physical_core_count().unwrap() as f32;

                        util::println_field_white(
                            "CPU",
                            format!("{:.2}", process.cpu_usage() / cpu_count),
                        );

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
            }
        }

        Ok(())
    }
}
