use std::process::{Child, Command, Output, Stdio};

use anyhow::Result;
use clap::Parser;

use crate::{logger::Logger, Crescent};

/// `journald` implementation.
pub struct Journald {
    service_name: String,
    user_service: bool,
}

impl Journald {
    pub fn new(service_name: &str) -> Self {
        let service_name = format!("cres.{}.service", service_name);
        Self {
            service_name,
            user_service: Crescent::parse().user_service,
        }
    }

    /// Run a `journald` command as the user.
    fn run_command(&self, mut args: Vec<&str>) -> Result<Output> {
        if self.user_service {
            args.push("--user");
        }

        Ok(Command::new("journalctl")
            .arg("--unit")
            .arg(&self.service_name)
            .arg("--no-pager")
            .args(args)
            .output()?)
    }
}

impl Logger for Journald {
    fn log(&self, n: u64) -> Result<String> {
        let output = self.run_command(vec!["--lines", &format!("{n}")])?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout)
    }

    fn follow(&self) -> Result<Child> {
        let mut args = vec![
            "--unit",
            &self.service_name,
            "--no-pager",
            "--follow",
            "--lines=200",
        ];

        if self.user_service {
            args.push("--user");
        }

        Ok(Command::new("journalctl")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?)
    }
}
