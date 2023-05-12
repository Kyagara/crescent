use crate::{application, crescent};
use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tabled::{settings::Style, Table, Tabled};

#[derive(Args)]
#[command(about = "List all running applications.")]
pub struct ListArgs;

#[derive(Tabled)]
struct ApplicationInfo {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Crescent PID")]
    crescent_pid: Pid,
    #[tabled(rename = "Subprocess PID")]
    subprocess_pid: Pid,
    #[tabled(rename = "CWD")]
    cwd: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

impl ListArgs {
    pub fn run() -> Result<()> {
        let mut crescent_pathbuf = crescent::crescent_dir()?;

        crescent_pathbuf.push("apps");

        if !crescent_pathbuf.exists() {
            fs::create_dir_all(&crescent_pathbuf)?;
        }

        let crescent_dir = crescent_pathbuf
            .read_dir()
            .context("Error reading crescent directory.")?;

        let mut system = System::new();
        system.refresh_processes();

        let mut apps = vec![];

        for app_dir in crescent_dir.flatten() {
            let path = app_dir.path();

            let app_name = path
                .file_stem()
                .context("Error extracting file name.")?
                .to_str()
                .context("Error converting OsStr to str.")?
                .to_string();

            let pids = application::app_pids_by_name(&app_name)?;

            if let Some(process) = system.process(pids[0]) {
                let app = ApplicationInfo {
                    name: app_name,
                    crescent_pid: pids[0],
                    subprocess_pid: pids[1],
                    cwd: process.cwd().display().to_string(),
                    uptime: format!("{}s", process.run_time()),
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
