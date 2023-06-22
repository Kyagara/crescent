use anyhow::{anyhow, Context, Result};
use predicates::{prelude::predicate, Predicate};
use std::{env, fs, path::PathBuf, str::from_utf8};

#[test]
fn start_short_lived() -> Result<()> {
    let name = "start_short_lived";
    test_utils::start_short_lived_command(name)?;
    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn attach_short_lived() -> Result<()> {
    let name = "attach_short_lived";
    test_utils::start_short_lived_command(name)?;

    let mut cmd = test_utils::get_base_command();
    cmd.args(["attach", name]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application not running."));

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn attach_socket_not_found_short_lived() -> Result<()> {
    let name = "attach_socket_not_found";
    test_utils::start_short_lived_command(name)?;

    let home = env::var("HOME").context("Error getting HOME env.")?;
    let socket_dir = PathBuf::from(home)
        .join(".crescent/apps/attach_socket_not_found/attach_socket_not_found.sock");

    if socket_dir.exists() {
        fs::remove_file(socket_dir)?
    }

    let mut cmd = test_utils::get_base_command();
    cmd.args(["attach", name]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Application not running."));

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn log_short_lived() -> Result<()> {
    let name = "log_short_lived";
    test_utils::start_short_lived_command(name)?;

    let mut cmd = test_utils::get_base_command();
    cmd.args(["log", name]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(">> Printed"));

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn log_flush_short_lived() -> Result<()> {
    let name = "log_flush_short_lived";
    test_utils::start_short_lived_command(name)?;

    let mut cmd = test_utils::get_base_command();
    cmd.args(["log", name, "--flush"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Flushed 'log_flush_short_lived' log file.",
    ));

    let mut cmd = test_utils::get_base_command();
    cmd.args(["log", name]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Log is empty at the moment."));

    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
fn log_follow_short_lived() -> Result<()> {
    let name = "log_follow_short_lived";
    test_utils::start_short_lived_command(name)?;

    let mut cmd = test_utils::get_base_command();

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

    test_utils::delete_app_folder(name)?;
    Ok(())
}
