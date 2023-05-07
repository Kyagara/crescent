use anyhow::{Context, Result};
use assert_cmd::Command;
use predicates::prelude::predicate;
use std::{env, fs, os::unix::net::UnixListener, path::PathBuf};

fn delete_app_folder(name: &str) -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;
    let mut crescent_dir = PathBuf::from(home);
    crescent_dir.push(".crescent");
    crescent_dir.push("apps");

    if !crescent_dir.exists() {
        fs::create_dir_all(&crescent_dir)?;
    }

    crescent_dir.push(name);

    if crescent_dir.exists() {
        fs::remove_dir_all(&crescent_dir)?;
    }

    Ok(())
}

#[test]
fn list_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("list");

    cmd.assert()
        .failure()
        .stderr("Error: No application running.\n");

    Ok(())
}

#[test]
fn log_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "test_app_not_available"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}

#[test]
fn send_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "test_app_not_available", "command"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}

#[test]
fn attach_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["attach", "test_app_not_available"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}

#[test]
fn start_short_lived_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/ls", "-n", "start_ls"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    delete_app_folder("start_ls")?;

    Ok(())
}

#[test]
fn log_short_lived_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-n", "log_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "log_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">> Printed 200 lines"));

    delete_app_folder("log_echo")?;

    Ok(())
}

#[test]
fn log_follow_short_lived_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-n", "log_follow_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "log_follow_echo", "-f"])
        .timeout(std::time::Duration::from_secs(1));

    cmd.assert()
        .interrupted()
        .stdout(predicate::str::contains(">> Watching log"));

    delete_app_folder("log_follow_echo")?;

    Ok(())
}

#[test]
fn list_long_running_service() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    let args = [
        "start",
        "./tools/long_running_service.py",
        "-i",
        "python3",
        "-n",
        "list_long_running",
    ];

    cmd.args(args);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("list_long_running"));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "list_long_running", "stop"]);

    cmd.assert().success().stdout("Command sent.\n");

    delete_app_folder("list_long_running")?;

    Ok(())
}

#[test]
fn send_command_socket() -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent");
    crescent_dir.push("apps");
    crescent_dir.push("send_socket_test");

    fs::create_dir_all(&crescent_dir).context("Error creating crescent directory.")?;

    let address = crescent_dir.join("send_socket_test.sock");

    let _socket = UnixListener::bind(address)?;

    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "send_socket_test", "command"]);

    cmd.assert().success().stdout("Command sent.\n");

    delete_app_folder("send_socket_test")?;

    Ok(())
}

#[test]
fn attach_command_socket_not_found() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-n", "attach_socket_not_found"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["attach", "attach_socket_not_found"]);

    cmd.assert()
        .failure()
        .stderr("Error: Socket file does not exist.\n");

    delete_app_folder("attach_socket_not_found")?;

    Ok(())
}
