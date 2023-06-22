use anyhow::{anyhow, Context, Result};
use predicates::{prelude::predicate, Predicate};
use serial_test::serial;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    str::from_utf8,
    thread,
};

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
    cmd.args(["stop", "test_app_not_available", "-f"]);
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
        .stderr(predicate::str::contains("Application does not exist."));
    Ok(())
}

#[test]
fn stop_force_long_running_service() -> Result<()> {
    let name = "stop_force_long_running_service";
    util::start_long_running_service(name)?;
    assert!(util::check_app_is_running(name)?);

    let mut cmd = util::get_base_command();
    cmd.args(["stop", name, "-f"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Signal sent."));

    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn stop_long_running_service() -> Result<()> {
    let name = "stop_long_running_service";

    let mut cmd = util::get_base_command();
    cmd.args(["start", "-n", name, "-p", "example"]);
    cmd.assert().success();

    assert!(util::check_app_is_running(name)?);

    let mut cmd = util::get_base_command();
    cmd.args(["stop", name]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Stop command sent."));

    // Sleeping to make sure the process shutdown
    thread::sleep(std::time::Duration::from_secs(1));

    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn profile_example_profile() -> Result<()> {
    let mut cmd = util::get_base_command();
    cmd.args(["profile", "example"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tools"));
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
fn start_python_without_interpreter() -> Result<()> {
    let name = "start_python_without_interpreter";

    let mut cmd = util::get_base_command();
    cmd.args([
        "start",
        "./tools/long_running_service.py",
        "-i",
        "",
        "-n",
        name,
    ]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting"));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    let home = env::var("HOME").context("Error getting HOME env.")?;

    let pid_path = PathBuf::from(
        home
            + "/.crescent/apps/start_python_without_interpreter/start_python_without_interpreter.pid",
    );

    assert!(pid_path.exists());

    let mut pid_str = String::new();
    File::open(pid_path)?.read_to_string(&mut pid_str)?;
    let pids: Vec<&str> = pid_str.lines().collect();
    assert!(pids.len() == 1);

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
fn log_empty_file() -> Result<()> {
    let name = "log_empty_file";
    util::start_short_lived_command(name)?;

    let home = env::var("HOME").context("Error getting HOME env.")?;
    let log_dir = PathBuf::from(home).join(".crescent/apps/log_empty_file/log_empty_file.log");
    fs::remove_file(&log_dir)?;
    File::create(log_dir)?;

    let mut cmd = util::get_base_command();
    cmd.args(["log", name]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Log is empty at the moment."));

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

    let binding = cmd.assert().interrupted();
    let output = binding.get_output();

    let stdout = &output.stdout;

    let print_predicate = predicate::str::contains(">> Printed");
    let watching_predicate = predicate::str::contains(">> Watching log");

    match from_utf8(stdout) {
        Ok(string) => print_predicate.eval(string) && watching_predicate.eval(string),
        Err(err) => return Err(anyhow!("{err}")),
    };

    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn log_flush_command() -> Result<()> {
    let name = "log_flush_command";
    util::start_short_lived_command(name)?;

    let mut cmd = util::get_base_command();
    cmd.args(["log", name, "--flush"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Flushed 'log_flush_command' log file.",
    ));

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
        .stderr(predicate::str::contains("Starting"));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(util::check_app_is_running(name)?);

    let mut cmd = util::get_base_command();
    cmd.args(["status", name]);

    let binding = cmd.assert().success();
    let output = binding.get_output();

    let stdout = &output.stdout;

    let usage_predicate = predicate::str::contains("arguments must be in multiple elements");
    let name_predicate = predicate::str::contains(name);

    match from_utf8(stdout) {
        Ok(string) => usage_predicate.eval(string) && name_predicate.eval(string),
        Err(err) => return Err(anyhow!("{err}")),
    };

    util::shutdown_long_running_service(name)?;
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn signal_long_running_service() -> Result<()> {
    let name = "signal_long_running_service";
    util::start_long_running_service(name)?;
    assert!(util::check_app_is_running(name)?);

    util::shutdown_long_running_service(name)?;
    util::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn send_command_socket() -> Result<()> {
    let name = "send_socket_test";
    util::start_long_running_service(name)?;
    assert!(util::check_app_is_running(name)?);

    let mut cmd = util::get_base_command();
    cmd.args(["send", name, "ping"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Command sent."));

    util::shutdown_long_running_service(name)?;
    util::delete_app_folder(name)?;
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
        .stderr(predicate::str::contains("Application not running."));

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
        .stderr(predicate::str::contains("Application not running."));

    util::delete_app_folder(name)?;
    Ok(())
}
