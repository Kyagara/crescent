use anyhow::{anyhow, Context, Result};
use predicates::{prelude::predicate, Predicate};
use std::{env, fs::File, io::Read, path::PathBuf, str::from_utf8, thread};

#[test]
fn stop_force_long_running_service() -> Result<()> {
    let name = "stop_force_long_running_service";
    test_utils::start_long_running_service(name)?;
    assert!(test_utils::check_app_is_running(name)?);

    let mut cmd = test_utils::get_base_command();
    cmd.args(["stop", name, "-f"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Signal sent."));

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn stop_long_running_service() -> Result<()> {
    let name = "stop_long_running_service";

    let mut cmd = test_utils::get_base_command();
    cmd.args(["start", "-n", name, "-p", "example"]);
    cmd.assert().success();

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(test_utils::check_app_is_running(name)?);

    let mut cmd = test_utils::get_base_command();
    cmd.args(["stop", name]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Stop command sent."));

    // Sleeping to make sure the process shutdown
    thread::sleep(std::time::Duration::from_secs(1));

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn profile_example_profile() -> Result<()> {
    let mut cmd = test_utils::get_base_command();
    cmd.args(["profile", "example"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tools"));

    cmd = test_utils::get_base_command();
    cmd.args(["profile", "example", "--json"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"interpreter_arguments\""));

    Ok(())
}

#[test]
fn start_python_without_interpreter() -> Result<()> {
    let name = "start_python_without_interpreter";

    let mut cmd = test_utils::get_base_command();
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

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn start_long_running_service_with_profile() -> Result<()> {
    let name = "example";
    let mut cmd = test_utils::get_base_command();
    cmd.args(["start", "-p", "example"]);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting"));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(test_utils::check_app_is_running(name)?);

    let mut cmd = test_utils::get_base_command();
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

    test_utils::shutdown_long_running_service(name)?;
    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn signal_long_running_service() -> Result<()> {
    let name = "signal_long_running_service";
    test_utils::start_long_running_service(name)?;
    assert!(test_utils::check_app_is_running(name)?);

    test_utils::shutdown_long_running_service(name)?;
    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn send_command_socket() -> Result<()> {
    let name = "send_socket_test";
    test_utils::start_long_running_service(name)?;
    assert!(test_utils::check_app_is_running(name)?);

    let mut cmd = test_utils::get_base_command();
    cmd.args(["send", name, "ping"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Command sent."));

    test_utils::shutdown_long_running_service(name)?;
    test_utils::delete_app_folder(name)?;
    Ok(())
}
