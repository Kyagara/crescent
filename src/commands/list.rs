use crate::{application, crescent};
use anyhow::{Context, Result};
use clap::Args;
use std::{fs::ReadDir, iter::Flatten, vec};
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
    subprocess_pid: String,
    #[tabled(rename = "CWD")]
    cwd: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

impl ListArgs {
    pub fn run() -> Result<()> {
        let mut crescent_pathbuf = crescent::crescent_dir()?;

        crescent_pathbuf.push("apps");

        let crescent_dir = crescent_pathbuf
            .read_dir()
            .context("Error reading crescent directory.")?
            .flatten();

        let apps = get_applications_info(crescent_dir)?;

        if apps.is_empty() {
            println!("No application running.");
            return Ok(());
        }

        let table = create_table(apps)?;
        println!("{table}");
        Ok(())
    }
}

fn create_table(apps: Vec<ApplicationInfo>) -> Result<Table> {
    let mut table = Table::new(apps);
    table.with(Style::modern());
    Ok(table)
}

fn get_applications_info(crescent_dir: Flatten<ReadDir>) -> Result<Vec<ApplicationInfo>> {
    let mut system = System::new();
    system.refresh_processes();

    let mut apps = vec![];

    for app_dir in crescent_dir {
        let app_name = app_dir
            .file_name()
            .to_str()
            .context("Error converting OsStr to str.")?
            .to_string();

        if !application::app_already_running(&app_name)? {
            continue;
        }

        let pids = application::app_pids_by_name(&app_name)?;

        if pids.is_empty() {
            continue;
        }

        let subprocess_pid = if pids.len() == 2 {
            pids[1].to_string()
        } else {
            String::from("Not running.")
        };

        if let Some(process) = system.process(pids[0]) {
            let app = ApplicationInfo {
                name: app_name,
                crescent_pid: pids[0],
                subprocess_pid,
                cwd: process.cwd().display().to_string(),
                uptime: format!("{}s", process.run_time()),
            };

            apps.push(app);
        }
    }

    Ok(apps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::util;
    use anyhow::Context;
    use serial_test::serial;
    use std::assert_eq;

    #[test]
    #[serial]
    fn unit_list_command_functions() -> Result<()> {
        let name = "list_command_application_info";
        util::start_long_running_service(name)?;
        assert!(util::check_app_is_running(name)?);

        let mut crescent_pathbuf = crescent::crescent_dir()?;

        crescent_pathbuf.push("apps");

        let crescent_dir = crescent_pathbuf
            .read_dir()
            .context("Error reading crescent directory.")?
            .flatten();

        let apps = get_applications_info(crescent_dir)?;
        assert_eq!(apps.len(), 1);
        let app = &apps[0];
        assert_eq!(&app.name, &name);

        let table = create_table(apps)?;
        assert!(!table.is_empty());
        assert_eq!(table.shape(), (2, 5));

        util::shutdown_long_running_service(name)?;
        assert!(!util::check_app_is_running(name)?);
        util::delete_app_folder(name)?;
        Ok(())
    }
}
