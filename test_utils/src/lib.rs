use anyhow::{anyhow, Context, Result};
use assert_cmd::Command;
use predicates::{prelude::predicate, Predicate};
use std::{env, fs, path::PathBuf, str::from_utf8, thread};

pub fn start_short_lived_command(name: &str) -> Result<()> {
    let mut cmd = get_base_command();
    cmd.args(["start", "/bin/echo", "-n", name]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting"));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

pub fn start_long_running_service(name: &str) -> Result<()> {
    let mut cmd = get_base_command();

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
        .stderr(predicate::str::contains("Starting"));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

pub fn check_app_is_running(name: &str) -> Result<bool> {
    let mut cmd = get_base_command();
    cmd.args(["status", name]);

    let binding = cmd.assert().success();
    let output = binding.get_output();

    let stdout = &output.stdout;

    let usage_predicate = predicate::str::contains("Memory usage");
    let name_predicate = predicate::str::contains(name);

    match from_utf8(stdout) {
        Ok(string) => Ok(usage_predicate.eval(string) && name_predicate.eval(string)),
        Err(err) => Err(anyhow!("{err}")),
    }
}

pub fn shutdown_long_running_service(name: &str) -> Result<()> {
    let mut cmd = get_base_command();

    cmd.args(["signal", name, "15"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Signal sent."));

    // Sleeping to make sure the process exited
    thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

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

// Solves issues with assert_cmd not finding the binary when using cross.
// https://github.com/assert-rs/assert_cmd/issues/139#issuecomment-1200146157
pub fn get_base_command() -> Command {
    let mut cmd;
    let path = assert_cmd::cargo::cargo_bin("cres");
    if let Some(runner) = find_runner() {
        let mut runner = runner.split_whitespace();
        cmd = Command::new(runner.next().unwrap());
        for arg in runner {
            cmd.arg(arg);
        }
        cmd.arg(path);
    } else {
        cmd = Command::new(path);
    }
    cmd
}

// https://github.com/assert-rs/assert_cmd/issues/139#issuecomment-1200146157
fn find_runner() -> Option<String> {
    for (key, value) in std::env::vars() {
        if key.starts_with("CARGO_TARGET_") && key.ends_with("_RUNNER") && !value.is_empty() {
            return Some(value);
        }
    }
    None
}

pub fn execute_against_app_not_exist(command: Vec<&str>) -> Result<()> {
    let mut cmd = get_base_command();
    cmd.args(command);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}
