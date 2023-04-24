use std::{fs::File, io::Read};

use crate::directory;
use anyhow::{Context, Result};
use clap::Args;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
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
    pub fn run() -> Result<()> {
        let mut crescent_pathbuf = directory::crescent_dir()?;

        crescent_pathbuf.push("apps");

        let crescent_dir = crescent_pathbuf
            .read_dir()
            .context("Should have the crescent's home directory");

        let mut system = System::new();
        system.refresh_all();

        let mut apps = vec![];

        for app_dir in crescent_dir?.flatten() {
            let path = app_dir.path();

            let name = path
                .file_stem()
                .context("Should contain the directory name as string")?
                .to_str()
                .context("Should be a valid string")?;

            let mut dir = path.clone();

            dir.push(name.to_owned() + ".pid");

            let mut pid_file = File::open(dir).context("Error opening PID file")?;

            let mut pid_str = String::new();

            pid_file
                .read_to_string(&mut pid_str)
                .context("Should have pid inside file")?;

            pid_str = pid_str.trim().to_string();

            let pid: usize = pid_str.parse().context("Should be a valid i32")?;

            if let Some(process) = system.process(Pid::from(pid)) {
                let (_, cmd) = process.cmd().split_at(2);

                let app = Application {
                    name: name.to_string(),
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

            return Ok(());
        }

        println!("No application running.");

        Ok(())
    }
}
