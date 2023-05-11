use anyhow::{Context, Result};
use assert_cmd::Command;
use predicates::prelude::predicate;
use std::{env, fs, path::PathBuf};

pub fn delete_app_folder(name: &str) -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;
    let mut crescent_dir = PathBuf::from(home);
    crescent_dir.push(".crescent/apps");

    if !crescent_dir.exists() {
        fs::create_dir_all(&crescent_dir)?;
    }

    crescent_dir.push(name);

    if crescent_dir.exists() {
        fs::remove_dir_all(&crescent_dir)?;
    }

    Ok(())
}

pub fn start_long_running_service() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    let args = [
        "start",
        "./tools/long_running_service.py",
        "-i",
        "python3",
        "-n",
        "long_running_service",
    ];

    cmd.args(args);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    Ok(())
}

pub fn check_list_contains_app_name(name: &str) -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(name));

    Ok(())
}

pub fn list_has_no_apps() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("list");

    cmd.assert().success().stdout("No application running.\n");

    Ok(())
}
