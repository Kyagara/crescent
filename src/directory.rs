use crate::process::process_pid_by_name;
use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};
use sysinfo::{System, SystemExt};

pub fn crescent_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("Error getting HOME env.")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent");

    if !crescent_dir.exists() {
        fs::create_dir_all(&crescent_dir).context("Error creating crescent directory.")?;
    }

    Ok(crescent_dir)
}

pub fn application_dir_by_name(name: &String) -> Result<PathBuf> {
    let mut crescent_dir = crescent_dir()?;

    crescent_dir.push("apps");

    crescent_dir.push(name);

    Ok(crescent_dir)
}

pub fn app_already_exist(name: &String) -> bool {
    if let Ok(pid) = process_pid_by_name(name) {
        let mut system = System::new();
        system.refresh_all();

        // First PID is always the crescent process.
        system.process(pid[0]).is_some()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn crescent_dir_created() -> Result<()> {
        let home = env::var("HOME")?;
        let mut home_path = PathBuf::from(home);
        home_path.push(".crescent");

        assert_eq!(crescent_dir()?, home_path);

        Ok(())
    }

    #[test]
    fn application_dir_created() -> Result<()> {
        let mut home_path = crescent_dir()?;
        home_path.push("apps");
        home_path.push("test_app");
        let app_name = String::from("test_app");

        fs::create_dir_all(home_path.clone())?;

        assert_eq!(application_dir_by_name(&app_name)?, home_path);

        Ok(())
    }
}
