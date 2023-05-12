use crate::{crescent, subprocess};
use anyhow::{Context, Result};
use std::{fs, path::PathBuf, str::FromStr};
use sysinfo::Pid;

pub fn app_dir_by_name(name: &String) -> Result<PathBuf> {
    let mut crescent_dir = crescent::crescent_dir()?;

    crescent_dir.push("apps");

    crescent_dir.push(name);

    Ok(crescent_dir)
}

pub fn app_pids_by_name(name: &String) -> Result<Vec<Pid>> {
    let application_path = app_dir_by_name(name)?;

    let app_name = application_path
        .file_name()
        .context("Error extracting file name.")?
        .to_str()
        .context("Error converting OsStr to str.")?
        .to_string();

    let mut pid_path = application_path;
    pid_path.push(app_name + ".pid");

    if !pid_path.exists() {
        return Ok(vec![]);
    }

    let pid_file = fs::read_to_string(pid_path).context("Error reading PID file to string.")?;

    let mut pid_strs: Vec<&str> = pid_file.split('\n').collect();
    pid_strs.retain(|&x| !x.is_empty());

    if pid_strs.is_empty() {
        return Ok(vec![]);
    }

    let cres_pid =
        Pid::from_str(pid_strs[0]).with_context(|| format!("Error parsing PID {}", pid_strs[0]))?;

    if pid_strs.len() == 1 {
        return Ok(vec![cres_pid]);
    }

    let app_pid =
        Pid::from_str(pid_strs[1]).with_context(|| format!("Error parsing PID {}", pid_strs[1]))?;

    let pids: Vec<Pid> = vec![cres_pid, app_pid];

    Ok(pids)
}

pub fn app_already_running(name: &String) -> Result<bool> {
    match app_pids_by_name(name) {
        Ok(pids) => match subprocess::get_app_process_envs(&pids[1])? {
            Some(_) => Ok(true),
            None => Ok(false),
        },
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crescent::crescent_dir;
    use std::fs::remove_dir_all;

    #[test]
    fn unit_application_dir_created() -> Result<()> {
        let mut home_path = crescent_dir()?;
        home_path.push("apps/test_app");
        let app_name = String::from("test_app");

        fs::create_dir_all(home_path.clone())?;

        assert_eq!(app_dir_by_name(&app_name)?, home_path);

        remove_dir_all(home_path)?;

        Ok(())
    }
}
