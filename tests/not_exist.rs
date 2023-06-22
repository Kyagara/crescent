use anyhow::Result;

#[test]
fn log_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["log", "test_app_not_exist"])
}

#[test]
fn send_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["send", "test_app_not_exist", "command"])
}

#[test]
fn signal_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["signal", "test_app_not_exist", "0"])
}

#[test]
fn status_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["status", "test_app_not_exist"])
}

#[test]
fn stop_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["stop", "test_app_not_exist", "-f"])
}

#[test]
fn kill_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["kill", "test_app_not_available"])
}

#[test]
fn attach_no_apps_running() -> Result<()> {
    test_utils::execute_against_app_not_exist(vec!["attach", "test_app_not_available"])
}
