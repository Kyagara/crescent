use std::{
    env::{self},
    fs,
    path::PathBuf,
};

use anyhow::{Context, Result};
use sysinfo::{System, SystemExt};

use crate::process::process_pid_by_name;

pub fn crescent_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("Error getting directory path")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent");

    if !crescent_dir.is_dir() {
        fs::create_dir_all(&crescent_dir).context("Couldn't create crescent directory.")?;
    }

    Ok(crescent_dir)
}

pub fn application_dir_by_name(name: String) -> Result<PathBuf> {
    let mut crescent_dir = crescent_dir()?;

    crescent_dir.push("apps");

    crescent_dir.push(name);

    Ok(crescent_dir)
}

pub fn app_already_exist(name: String) -> bool {
    if let Ok(pid) = process_pid_by_name(name) {
        let mut system = System::new();
        system.refresh_all();

        system.process(pid).is_some()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_crescent_dir() {
        let home = env::var("HOME").unwrap();
        let mut home_path = PathBuf::from(home);
        home_path.push(".crescent");

        assert_eq!(crescent_dir().unwrap(), home_path);
    }

    #[test]
    fn test_application_dir_by_name() {
        let mut home_path = crescent_dir().unwrap();
        home_path.push("apps");
        home_path.push("app");

        assert_eq!(
            application_dir_by_name(String::from("app")).unwrap(),
            home_path
        );
    }
}
