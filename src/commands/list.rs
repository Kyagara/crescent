use crate::directory;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::{fs::File, io::Read, str::FromStr, time::Duration};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tabled::{settings::Style, Table, Tabled};

#[derive(Args)]
#[command(about = "List all running applications.")]
pub struct ListArgs;

#[derive(Tabled)]
struct ApplicationInfo {
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
            .context("Error reading crescent directory.")?;

        let mut system = System::new();
        system.refresh_all();

        let mut apps = vec![];

        for app_dir in crescent_dir.flatten() {
            let path = app_dir.path();

            let name = path
                .file_stem()
                .context("Error extracting file name.")?
                .to_str()
                .context("Error converting OsStr to str.")?;

            let mut dir = path.clone();

            dir.push(name.to_owned() + ".pid");

            if !dir.exists() {
                continue;
            }

            let mut pid_file = File::open(dir).context("Error opening PID file.")?;

            let mut pid_str = String::new();

            pid_file
                .read_to_string(&mut pid_str)
                .context("Error reading PID file to string.")?;

            let pids: Vec<Pid> = pid_str
                .split('\n')
                .map(|x| Pid::from_str(x).context("Error parsing Pid.").unwrap())
                .collect();

            if let Some(process) = system.process(pids[0]) {
                let (_, cmd) = process.cmd().split_at(2);

                let app = ApplicationInfo {
                    name: name.to_string(),
                    pid: pids[0].to_string(),
                    command: cmd.join(" "),
                    cwd: process.cwd().display().to_string(),
                    uptime: format!("{:?}", Duration::from_secs(process.run_time())),
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

        Err(anyhow!("No application running."))
    }
}
