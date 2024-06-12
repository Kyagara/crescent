use std::{env, path::PathBuf, process::Command};

use crossterm::style::Stylize;

pub fn get_uptime_from_seconds(secs: u64) -> String {
    // Laughing while writing this
    // It works at least
    let check_plural = |number: u64, str: &str| {
        if number > 1 {
            return str.to_owned() + "s";
        }
        str.to_owned()
    };

    if secs <= 59 {
        return format!("{secs} {}", check_plural(secs, "second"));
    }

    let mins = secs / 60;

    if mins <= 59 {
        return format!("{mins} {}", check_plural(mins, "minute"));
    }

    let hours = mins / 60;

    if hours <= 23 {
        return format!("{hours} {}", check_plural(hours, "hour"));
    }

    let days = hours / 24;

    format!("{days} {}", check_plural(days, "day"))
}

pub fn print_title_cyan(title: &str) {
    println!("{}", title.bold().cyan())
}

pub fn println_field_white<T: std::fmt::Display>(name: &str, value: T) {
    println!("{}: {value}", name.white())
}

pub fn get_exec_path() -> PathBuf {
    let exe_path = env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap();

    std::env::var_os("CARGO_BIN_EXE_cres")
        .map(|p| p.into())
        .unwrap_or_else(|| exe_path.join("cres"))
}

// Solves issues with assert_cmd not finding the binary when using cross.
// https://github.com/assert-rs/assert_cmd/issues/139#issuecomment-1200146157
pub fn get_base_command(path: PathBuf) -> Command {
    let mut cmd;
    if let Some(runner) = find_runner() {
        let mut runner = runner.split_whitespace();
        cmd = Command::new(runner.next().unwrap());
        for arg in runner {
            cmd.arg(arg);
        }
        cmd.arg(path);
    } else {
        cmd = Command::new(path);
    }
    cmd
}

// https://github.com/assert-rs/assert_cmd/issues/139#issuecomment-1200146157
fn find_runner() -> Option<String> {
    for (key, value) in std::env::vars() {
        if key.starts_with("CARGO_TARGET_") && key.ends_with("_RUNNER") && !value.is_empty() {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::get_uptime_from_seconds;

    #[test]
    fn unit_get_uptime_from_seconds() {
        let mut secs: u64 = 1;

        assert_eq!(get_uptime_from_seconds(secs), "1 second");
        secs = 30;
        assert_eq!(get_uptime_from_seconds(secs), "30 seconds");
        secs = 60;
        assert_eq!(get_uptime_from_seconds(secs), "1 minute");
        secs = 1800;
        assert_eq!(get_uptime_from_seconds(secs), "30 minutes");
        secs = 3600;
        assert_eq!(get_uptime_from_seconds(secs), "1 hour");
        secs = 43200;
        assert_eq!(get_uptime_from_seconds(secs), "12 hours");
        secs = 86400;
        assert_eq!(get_uptime_from_seconds(secs), "1 day");
        secs = 604800;
        assert_eq!(get_uptime_from_seconds(secs), "7 days")
    }
}
