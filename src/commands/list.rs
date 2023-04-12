use crate::process::{self, process_pid_by_name};
use clap::Args;
use sysinfo::{ProcessExt, System, SystemExt};
use tabled::{settings::Style, Table, Tabled};

#[derive(Args)]
#[command(about = "List all running applications.")]
pub struct ListArgs {}

#[derive(Tabled)]
struct Application {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "PID")]
    pid: String,
    #[tabled(rename = "Command")]
    command: String,
    #[tabled(rename = "CWD")]
    cwd: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

impl ListArgs {
    pub fn run() {
        let temp_dir = process::crescent_temp_dir();

        let mut system = System::new();
        system.refresh_all();

        let mut apps = vec![];

        for app_dir in temp_dir.read_dir().unwrap().flatten() {
            let name = app_dir.file_name().to_str().unwrap().to_string();

            let pid = process_pid_by_name(name.clone());

            if pid.is_none() {
                continue;
            }

            if let Some(process) = system.process(pid.unwrap()) {
                let app = Application {
                    name,
                    pid: pid.unwrap().to_string(),
                    command: process.cmd().join(" "),
                    cwd: process.cwd().join(" ").to_str().unwrap().to_string(),
                    uptime: process.run_time().to_string() + "s",
                };

                apps.push(app);
            }
        }

        if !apps.is_empty() {
            let mut table = Table::new(apps);
            table.with(Style::modern());
            println!("{table}");
        }
    }
}
