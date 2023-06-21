use crate::{application, crescent, util};
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
    #[tabled(rename = "crescent PID")]
    crescent_pid: Pid,
    #[tabled(rename = "Subprocess PID")]
    subprocess_pid: String,
    #[tabled(rename = "CWD")]
    cwd: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
}

impl ListArgs {
    pub fn run(self) -> Result<()> {
        let apps_dir = crescent::get_apps_dir()?;

        let dirs = apps_dir
            .read_dir()
            .context("Error reading apps directory.")?
            .flatten();

        let apps = self.get_applications_info(dirs)?;

        if apps.is_empty() {
            println!("No application running.");
            return Ok(());
        }

        let table = self.create_table(apps)?;
        println!("{table}");
        Ok(())
    }

    fn create_table(&self, apps: Vec<ApplicationInfo>) -> Result<Table> {
        let mut table = Table::new(apps);
        table.with(Style::modern());
        Ok(table)
    }

    fn get_applications_info(
        &self,
        crescent_dir: Flatten<ReadDir>,
    ) -> Result<Vec<ApplicationInfo>> {
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
                    uptime: util::get_uptime_from_seconds(process.run_time()),
                };

                apps.push(app);
            }
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_utils;
    use anyhow::Context;
    use serial_test::serial;
    use std::assert_eq;

    #[test]
    fn unit_list_run() -> Result<()> {
        let command = ListArgs {};
        command.run()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn unit_list_command_functions() -> Result<()> {
        let name = "list_command_application_info";
        test_utils::start_long_running_service(name)?;
        assert!(test_utils::check_app_is_running(name)?);

        let apps_dir = crescent::get_apps_dir()?;

        let dirs = apps_dir
            .read_dir()
            .context("Error reading crescent directory.")?
            .flatten();

        let list_command = ListArgs {};

        let apps = list_command.get_applications_info(dirs)?;
        let app = apps.into_iter().find(|app| app.name == name).unwrap();

        assert_eq!(&app.name, &name);

        let table = list_command.create_table(vec![app])?;
        assert!(!table.is_empty());
        assert_eq!(table.shape(), (2, 5));

        test_utils::shutdown_long_running_service(name)?;
        test_utils::delete_app_folder(name)?;
        Ok(())
    }
}
