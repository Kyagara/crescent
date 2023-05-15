use anyhow::{anyhow, Context, Result};
use predicates::{prelude::predicate, Predicate};
use serial_test::serial;
use std::{env, fs, os::unix::net::UnixListener, path::PathBuf, str::from_utf8, thread};

mod util;

#[test]
#[serial]
fn list_command_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No application running."));
    Ok(())
}

#[test]
fn log_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["log", "test_app_not_available"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn send_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["send", "test_app_not_available", "command"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn signal_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["signal", "test_app_not_available", "0"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn status_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["status", "test_app_not_available"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn stop_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["stop", "test_app_not_available"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn kill_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["kill", "test_app_not_available"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn attach_no_apps_running() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["attach", "test_app_not_available"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application not running."));
    Ok(())
}

#[test]
fn start_short_lived_command() -> Result<()> {
    let name = "start_echo";
    util::start_short_lived_command(name)?;
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn log_short_lived_command() -> Result<()> {
    let name = "log_echo";
    util::start_short_lived_command(name)?;

    let mut cmd = util::get_base_command();
    cmd.args(["log", name]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">> Printed"));

    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn log_follow_short_lived_command() -> Result<()> {
    let name = "log_follow_echo";
    util::start_short_lived_command(name)?;
    let mut cmd = util::get_base_command();

    cmd.args(["log", name, "-f"])
        .timeout(std::time::Duration::from_secs(1));

    cmd.assert()
        .interrupted()
        .stdout(predicate::str::contains(">> Printed"));

    cmd.assert()
        .interrupted()
        .stdout(predicate::str::contains(">> Watching log"));

    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
#[serial]
fn list_command_long_running_service() -> Result<()> {
    let name = "list_long_running_service";
    util::start_long_running_service(name)?;
    assert!(util::check_app_is_running(name)?);

    let mut cmd = util::get_base_command();
    cmd.args(["list"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(name));

    util::shutdown_long_running_service(name)?;
    assert!(!util::check_app_is_running(name)?);
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn start_long_running_service_with_profile() -> Result<()> {
    let name = "example";
    let mut cmd = util::get_base_command();
    cmd.args(["start", "-p", "example"]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting daemon."));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(util::check_app_is_running(name)?);

    let mut cmd = util::get_base_command();
    cmd.args(["status", name]);

    let binding = cmd.assert().success();
    let output = binding.get_output();

    let stdout = &output.stdout;

    let usage_predicate =
        predicate::str::contains("arguments can be in one big element or in multiple elements");
    let name_predicate = predicate::str::contains(name);

    match from_utf8(stdout) {
        Ok(string) => usage_predicate.eval(string) && name_predicate.eval(string),
        Err(err) => return Err(anyhow!("{err}")),
    };

    util::shutdown_long_running_service(name)?;
    assert!(!util::check_app_is_running(name)?);
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn signal_long_running_service() -> Result<()> {
    let name = "signal_long_running_service";
    util::start_long_running_service(name)?;
    assert!(util::check_app_is_running(name)?);

    util::shutdown_long_running_service(name)?;
    assert!(!util::check_app_is_running(name)?);
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn send_command_socket() -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;
    let app_dir = PathBuf::from(home).join(".crescent/apps/send_socket_test");
    fs::create_dir_all(&app_dir).context("Error creating crescent directory.")?;

    let address = app_dir.join("send_socket_test.sock");

    let _socket = UnixListener::bind(address)?;

    let mut cmd = util::get_base_command();
    cmd.args(["send", "send_socket_test", "command"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Command sent."));

    util::delete_app_folder("send_socket_test")?;
    Ok(())
}

#[test]
fn attach_short_lived_command() -> Result<()> {
    let name = "attach_echo";
    util::start_short_lived_command(name)?;

    let mut cmd = util::get_base_command();
    cmd.args(["attach", name]);

    cmd.assert()
        .failure()
        .stderr("Error: Application not running.\n");

    assert!(!util::check_app_is_running(name)?);
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn attach_command_socket_not_found() -> Result<()> {
    let name = "attach_socket_not_found";
    util::start_short_lived_command(name)?;

    let home = env::var("HOME").context("Error getting HOME env.")?;
    let socket_dir = PathBuf::from(home)
        .join(".crescent/apps/attach_socket_not_found/attach_socket_not_found.sock");

    if socket_dir.exists() {
        fs::remove_file(socket_dir)?
    }

    let mut cmd = util::get_base_command();
    cmd.args(["attach", name]);

    cmd.assert()
        .failure()
        .stderr("Error: Application not running.\n");

    util::delete_app_folder(name)?;
    Ok(())
}
