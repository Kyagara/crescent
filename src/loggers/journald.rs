use std::process::{Command, Output};

use crate::logger::LogSystem;

use anyhow::Result;

/// `journald` implementation.
pub struct Journald {
    service_name: String,
}

impl Journald {
    pub const fn new() -> Self {
        Self {
            service_name: String::new(),
        }
    }

    /// Run a journald command as the user.
    fn run_command(&self, args: Vec<&str>) -> Result<Output> {
        let cmd = Command::new("journalctl")
            .arg("--user")
            .arg("--unit")
            .arg(&self.service_name)
            .arg("--no-pager")
            .args(args)
            .output()?;
        Ok(cmd)
    }
}

impl LogSystem for Journald {
    fn set_service_name(&mut self, name: &str) {
        self.service_name = format!("cres.{name}.service");
    }

    fn log(&self, n: u64) -> Result<String> {
        let output = self.run_command(vec!["--lines", &format!("{n}")])?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }

    fn follow(&self) -> Result<()> {
        let mut cmd = Command::new("journalctl")
            .arg("--user")
            .arg("--unit")
            .arg(&self.service_name)
            .arg("--no-pager")
            .arg("--follow")
            .spawn()?;

        cmd.wait()?;
        Ok(())
    }
}
