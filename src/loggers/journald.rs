use std::process::{Command, Output};

use crate::logger::LogSystem;

use anyhow::Result;
use crossterm::style::Stylize;

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

    fn log(&self, n: u64) -> Result<Vec<String>> {
        let output = self.run_command(vec!["--lines", &format!("{n}")])?;

        let out = String::from_utf8(output.stdout)?;
        let output_lines = out.lines().collect::<Vec<&str>>();
        let lines = output_lines.iter().map(ToString::to_string).collect();

        Ok(lines)
    }

    fn follow(&self) -> Result<()> {
        let lines = self.log(10)?;
        for line in lines {
            eprintln!("{line}");
        }

        eprintln!("{}", "Following logs... Press Ctrl+C to stop".white());
        self.run_command(vec!["--follow"])?;
        Ok(())
    }
}
