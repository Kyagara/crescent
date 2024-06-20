use std::process::{Child, Command, Output, Stdio};

use crate::logger::LogSystem;

use anyhow::Result;

/// `journald` implementation.
pub struct Journald {
    service_name: String,
}

impl Journald {
    pub fn new(service_name: String) -> Self {
        let service_name = format!("cres.{}.service", service_name);
        Self { service_name }
    }

    /// Run a journald command as the user.
    fn run_command(&self, args: Vec<&str>) -> Result<Output> {
        Ok(Command::new("journalctl")
            .arg("--user")
            .arg("--unit")
            .arg(&self.service_name)
            .arg("--no-pager")
            .args(args)
            .output()?)
    }
}

impl LogSystem for Journald {
    fn log(&self, n: u64) -> Result<String> {
        let output = self.run_command(vec!["--lines", &format!("{n}")])?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }

    fn follow(&self) -> Result<Child> {
        Ok(Command::new("journalctl")
            .arg("--user")
            .arg("--unit")
            .arg(&self.service_name)
            .arg("--no-pager")
            .arg("--follow")
            .stdout(Stdio::piped())
            .spawn()?)
    }
}
