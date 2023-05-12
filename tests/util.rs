use anyhow::{anyhow, Context, Result};
use assert_cmd::Command;
use predicates::{prelude::predicate, Predicate};
use std::{env, fs, path::PathBuf, str::from_utf8, thread};

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

pub fn start_long_running_service(name: &str) -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    let args = [
        "start",
        "./tools/long_running_service.py",
        "-i",
        "python3",
        "-n",
        name,
    ];

    cmd.args(args);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting daemon."));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

pub fn check_app_is_running(name: &str) -> Result<bool> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["status", name]);

    let binding = cmd.assert().success();
    let output = binding.get_output();

    let stdout = &output.stdout;

    let predicate = predicate::str::contains("Memory usage:");

    match from_utf8(stdout) {
        Ok(string) => Ok(predicate.eval(string)),
        Err(err) => Err(anyhow!("{err}")),
    }
}
