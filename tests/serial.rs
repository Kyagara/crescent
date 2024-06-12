use std::thread;

use anyhow::Result;
use predicates::prelude::predicate;
use serial_test::serial;

#[test]
#[serial]
fn list_no_apps_running() -> Result<()> {
    let mut cmd = test_utils::get_base_command();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No application running."));
    Ok(())
}

#[test]
#[serial]
fn list_long_running_service() -> Result<()> {
    let name = "list_long_running_service";
    test_utils::start_long_running_service(name)?;
    assert!(test_utils::check_app_is_running(name)?);

    let mut cmd = test_utils::get_base_command();
    cmd.args(["list"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(name));

    test_utils::shutdown_long_running_service(name)?;
    test_utils::delete_app_folder(name)?;
    Ok(())
}

#[test]
#[serial]
fn start_saved_long_running_service() -> Result<()> {
    let name = "start_saved_running_service";
    test_utils::start_long_running_service(name)?;
    assert!(test_utils::check_app_is_running(name)?);

    let mut cmd = test_utils::get_base_command();
    cmd.args(["save"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Saved "));

    test_utils::shutdown_long_running_service(name)?;

    cmd = test_utils::get_base_command();
    cmd.args(["start", "--saved"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting applications."));

    // Sleeping to make sure the process started
    thread::sleep(std::time::Duration::from_secs(1));

    assert!(test_utils::check_app_is_running(name)?);
    test_utils::shutdown_long_running_service(name)?;
    test_utils::delete_app_folder(name)?;
    Ok(())
}
