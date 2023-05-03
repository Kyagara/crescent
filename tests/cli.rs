use anyhow::Result;
use assert_cmd::Command;

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
fn send_no_apps_running() -> Result<()> {
    let mut cmd = Command::cargo_bin("cres")?;

    cmd.arg("send").arg("test_app_not_available").arg("command");

    cmd.assert()
        .failure()
        .stderr("Error: Application does not exist.\n");

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
