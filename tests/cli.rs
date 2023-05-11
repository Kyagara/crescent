use anyhow::{Context, Result};
use assert_cmd::Command;
use predicates::prelude::predicate;
use std::{env, fs, os::unix::net::UnixListener, path::PathBuf, thread};

mod util;

#[test]
fn list_no_apps_running() -> Result<()> {
    util::list_has_no_apps()?;

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
fn signal_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["signal", "test_app_not_available", "0"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}

#[test]
fn stop_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["stop", "test_app_not_available"]);

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

    Ok(())
}

#[test]
fn kill_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["kill", "test_app_not_available"]);

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

    // Sleeping to make sure the process exited
    thread::sleep(std::time::Duration::from_millis(500));

    util::delete_app_folder("start_ls")?;

    Ok(())
}

#[test]
fn log_short_lived_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-n", "log_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    // Sleeping to make sure the process exited
    thread::sleep(std::time::Duration::from_millis(500));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "log_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">> Printed 7 lines"));

    util::delete_app_folder("log_echo")?;

    Ok(())
}

#[test]
fn log_follow_short_lived_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-n", "log_follow_echo"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    // Sleeping to make sure the process exited
    thread::sleep(std::time::Duration::from_millis(500));

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["log", "log_follow_echo", "-f"])
        .timeout(std::time::Duration::from_secs(1));

    cmd.assert()
        .interrupted()
        .stdout(predicate::str::contains(">> Printed 7 lines"));

    cmd.assert()
        .interrupted()
        .stdout(predicate::str::contains(">> Watching log"));

    util::delete_app_folder("log_follow_echo")?;

    Ok(())
}

#[test]
fn list_long_running_service() -> Result<()> {
    util::start_long_running_service()?;

    util::check_list_contains_app_name("long_running_service")?;

    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "long_running_service", "stop"]);

    cmd.assert().success().stdout("Command sent.\n");

    util::delete_app_folder("long_running_service")?;

    Ok(())
}

#[test]
fn signal_long_running_service() -> Result<()> {
    util::start_long_running_service()?;

    util::check_list_contains_app_name("long_running_service")?;

    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["signal", "long_running_service", "15"]);

    cmd.assert().success().stdout("Signal sent.\n");

    util::list_has_no_apps()?;

    util::delete_app_folder("long_running_service")?;

    Ok(())
}

#[test]
fn send_command_socket() -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent/apps/send_socket_test");

    fs::create_dir_all(&crescent_dir).context("Error creating crescent directory.")?;

    let address = crescent_dir.join("send_socket_test.sock");

    let _socket = UnixListener::bind(address)?;

    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["send", "send_socket_test", "command"]);

    cmd.assert().success().stdout("Command sent.\n");

    util::delete_app_folder("send_socket_test")?;

    Ok(())
}

#[test]
fn attach_command_socket_not_found() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.args(["start", "/bin/echo", "-n", "attach_socket_not_found"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting daemon."));

    // Sleeping to make sure the process exited
    thread::sleep(std::time::Duration::from_secs(1));

    let home = env::var("HOME").context("Error getting HOME env.")?;
    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent/apps/attach_socket_not_found.sock");

    if crescent_dir.exists() {
        fs::remove_file(crescent_dir)?
    }

    cmd = Command::cargo_bin("cres")?;

    cmd.args(["attach", "attach_socket_not_found"]);

    cmd.assert()
        .failure()
        .stderr("Error: Socket file does not exist.\n");

    util::delete_app_folder("attach_socket_not_found")?;

    Ok(())
}
