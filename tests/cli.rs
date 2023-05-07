use std::{env, fs, os::unix::net::UnixListener, path::PathBuf};

use anyhow::{Context, Result};
use assert_cmd::Command;

#[test]
fn start_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("start").arg("/bin/ls");

    cmd.assert().success().stdout("Starting daemon\n");

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

    cmd.arg("log").arg("test_app_not_available");

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}

#[test]
fn log_echo() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("start").arg("/bin/echo").arg("-a").arg("command");

    cmd.assert().success().stdout("Starting daemon\n");

    cmd = Command::cargo_bin("cres")?;

    cmd.arg("log").arg("echo");

    cmd.assert()
        .success()
        .stdout("command\n>> Printed 200 lines\n");

    Ok(())
}

#[test]
fn send_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("send").arg("test_app_not_available").arg("command");

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

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

    let _socket = UnixListener::bind(&address)?;

    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("send").arg("socket_test").arg("command");

    cmd.assert().success().stdout("Command sent.\n");

    fs::remove_file(address).unwrap();

    Ok(())
}

#[test]
fn attach_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("attach").arg("test_app_not_available");

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}
