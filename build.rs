use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    // Only systemd is supported for now so just check if it is running.
    // Ideally we would check if one of the supported init and logging system is running and panic if not.

    if !run_command("ps", &["-p", "1", "-o", "comm="]) {
        panic!("systemd is not running.");
    }

    // Create directories if they don't exist.

    let profile_path = env::var("HOME").expect("Error retrieving HOME directory.");
    let profile_path = PathBuf::from(profile_path + "/.crescent/profiles/");
    fs::create_dir_all(profile_path).unwrap();

    let app_path = env::var("HOME").expect("Error retrieving HOME directory.");
    let app_path = PathBuf::from(app_path + "/.crescent/apps/");
    fs::create_dir_all(app_path).unwrap();
}

fn run_command(binary: &str, args: &[&str]) -> bool {
    Command::new(binary)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
