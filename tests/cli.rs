use anyhow::{Context, Result};
use assert_cmd::Command;
use predicates::prelude::predicate;
use std::{env, fs, os::unix::net::UnixListener, path::PathBuf};

fn reset_apps_folder() -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;
    let mut crescent_dir = PathBuf::from(home);
    crescent_dir.push(".crescent");
    crescent_dir.push("apps");

    if crescent_dir.exists() {
        fs::remove_dir_all(&crescent_dir)?;
    }

    fs::create_dir_all(crescent_dir)?;

    Ok(())
}

#[test]
fn start_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/ls"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn list_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("list");

    cmd.assert()
        .failure()
        .stderr("Error: No application running.\n");

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn list_app() -> Result<()> {
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

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn log_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "test_app_not_available"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn log_echo() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-a", "command", "-n", "log_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "log_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("command"));

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn send_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "test_app_not_available", "command"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn send_command() -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent");
    crescent_dir.push("apps");
    crescent_dir.push("socket_test");

    fs::create_dir_all(&crescent_dir).context("Error creating crescent directory.")?;

    let address = crescent_dir.join("socket_test.sock");

    let _socket = UnixListener::bind(address)?;

    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "socket_test", "command"]);

    cmd.assert().success().stdout("Command sent.\n");

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn attach_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["attach", "test_app_not_available"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn attach_echo_socket_not_found() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args([
        "start",
        "/bin/echo",
        "-a",
        "command",
        "-n",
        "socket_not_found",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["attach", "socket_not_found"]);

    cmd.assert()
        .failure()
        .stderr("Error: Socket file does not exist.\n");

    reset_apps_folder()?;

    Ok(())
}

#[test]
fn log_echo_follow() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-a", "command"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "echo", "-f"])
        .timeout(std::time::Duration::from_secs(1));

    cmd.assert()
        .stdout(predicate::str::contains(">> Watching log"));

    reset_apps_folder()?;

    Ok(())
}
