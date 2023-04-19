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
        let crescent_dir = process::crescent_temp_dir()
            .read_dir()
            .expect("should have the crescent's home directory");

        let mut system = System::new();
        system.refresh_all();

        let mut apps = vec![];

        for app_dir in crescent_dir.flatten() {
            let name = app_dir
                .file_name()
                .into_string()
                .expect("should be a string containing the file name");

            let pid = process_pid_by_name(name.clone());

            let pid = match pid {
                Some(pid) => pid,
                None => continue,
            };

            if let Some(process) = system.process(pid) {
                let (_, cmd) = process.cmd().split_at(2);

                let app = Application {
                    name,
                    pid: pid.to_string(),
                    command: cmd.join(" "),
                    cwd: process.cwd().display().to_string(),
                    uptime: process.run_time().to_string() + "s",
                };

                apps.push(app);
            }
        }

        if !apps.is_empty() {
            let mut table = Table::new(apps);
            table.with(Style::modern());

            println!("{table}");

            return;
        }

        println!("No application running.");
    }
}
