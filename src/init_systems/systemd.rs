use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;

use crate::{
    init_system::{InitSystem, Status, StatusOutput},
    Crescent, APPS_DIR,
};

static USER_DIR: Lazy<String> = Lazy::new(|| {
    let mut path = env::var("HOME").expect("Error retrieving HOME directory.");
    path.push_str("/.config/systemd/user/");
    path
});

const SYSTEM_DIR: &str = concat!("/etc/systemd/system/");

/// `systemd` implementation.
pub struct Systemd {
    /// `<name>`
    name: String,
    /// `cres.<name>.service`
    service_name: String,
    /// `cres.<name>.socket`
    socket_name: String,
    /// Use the user's service directory instead of the system's, requires root if false.
    user_service: bool,
}

impl Systemd {
    pub fn new(name: Option<&str>) -> Self {
        let name = name.unwrap_or_default();

        Self {
            name: name.to_string(),
            service_name: format!("cres.{name}.service"),
            socket_name: format!("cres.{name}.socket"),
            user_service: Crescent::parse().user_service,
        }
    }

    /// Run a `systemctl` command.
    fn run_command(&self, mut args: Vec<&str>) -> Result<Output> {
        if self.user_service {
            args.push("--user");
        }

        Ok(Command::new("systemctl")
            .arg("--no-pager")
            .args(args)
            .output()?)
    }

    fn write_service_unit(&self, path: PathBuf, cmd: &str) -> Result<()> {
        let requires = format!("Requires={}", self.socket_name);
        let after = format!("After=network.target {}", self.socket_name);
        let exec_start = format!("ExecStart={cmd}");
        let wanted_by = match self.user_service {
            true => "WantedBy=default.target",
            false => "WantedBy=multi-user.target",
        };

        let service = [
            "[Unit]",
            &requires,
            &after,
            "",
            "[Service]",
            "Type=exec",
            &exec_start,
            "StandardInput=socket",
            "StandardOutput=journal",
            "StandardError=journal",
            "",
            "[Install]",
            wanted_by,
            "",
        ];

        fs::write(path, service.join("\n"))?;
        Ok(())
    }

    fn write_socket_unit(&self, path: PathBuf) -> Result<()> {
        let listen_fifo = format!("ListenFIFO={}/stdin", APPS_DIR.to_string() + &self.name);

        let socket = [
            "[Socket]",
            &listen_fifo,
            "",
            "[Install]",
            "WantedBy=sockets.target",
            "",
        ];

        fs::write(path, socket.join("\n"))?;
        Ok(())
    }
}

impl InitSystem for Systemd {
    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
        self.service_name = format!("cres.{name}.service");
        self.socket_name = format!("cres.{name}.socket");
    }

    fn get_scripts_paths(&self) -> Vec<String> {
        if self.user_service {
            vec![
                USER_DIR.to_string() + &self.service_name,
                USER_DIR.to_string() + &self.socket_name,
            ]
        } else {
            vec![
                SYSTEM_DIR.to_string() + &self.service_name,
                SYSTEM_DIR.to_string() + &self.socket_name,
            ]
        }
    }

    fn is_running(&self) -> Result<bool> {
        let output = self.run_command(vec!["is-active", &self.service_name])?;
        let stdout = String::from_utf8(output.stdout)?;
        let is_running = stdout.trim().to_string();
        Ok(is_running == "active")
    }

    fn is_enabled(&self) -> Result<bool> {
        let output = self.run_command(vec!["is-enabled", &self.service_name])?;
        let stdout = String::from_utf8(output.stdout)?;
        let is_enabled = stdout.trim().to_string();
        Ok(is_enabled == "enabled")
    }

    fn reload(&self) -> Result<()> {
        self.run_command(vec!["daemon-reload"])?;
        Ok(())
    }

    fn create(&self, cmd: &str) -> Result<()> {
        let path_str = match self.user_service {
            true => USER_DIR.to_string(),
            false => SYSTEM_DIR.to_string(),
        };

        let path = PathBuf::from(path_str);

        eprintln!("Writing '{}' unit", self.service_name);
        let service_path = path.join(self.service_name.clone());
        self.write_service_unit(service_path, cmd)?;

        eprintln!("Writing '{}' unit", self.socket_name);
        let socket_path = path.join(self.socket_name.clone());
        self.write_socket_unit(socket_path)?;
        Ok(())
    }

    fn start(&self) -> Result<()> {
        self.run_command(vec!["start", &self.service_name])?;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        self.run_command(vec!["stop", &self.socket_name])?;
        Ok(())
    }

    fn kill(&self, signal: i32) -> Result<()> {
        self.run_command(vec![
            "kill",
            &self.service_name,
            format!("--signal={signal}").as_str(),
        ])?;
        Ok(())
    }

    fn restart(&self) -> Result<()> {
        self.run_command(vec!["restart", &self.service_name])?;
        Ok(())
    }

    fn enable(&self) -> Result<()> {
        self.run_command(vec!["enable", &self.service_name])?;
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        self.run_command(vec!["disable", &self.service_name])?;
        Ok(())
    }

    fn status(&self, raw: bool) -> Result<StatusOutput> {
        let output = match self.run_command(vec!["status", &self.service_name]) {
            Ok(output) => output,
            Err(err) => return Err(err),
        };

        let stdout = String::from_utf8(output.stdout)?;
        if raw {
            return Ok(StatusOutput::Raw(stdout));
        }

        let result: Vec<&str> = stdout.lines().collect();
        let mut iter = result.iter().map(|line| line.trim());

        let script = iter
            .find(|line| line.contains("Loaded:"))
            .context("Error finding Loaded line.")?
            .split('(')
            .nth(1)
            .and_then(|s| s.split(';').next())
            .unwrap_or("")
            .trim()
            .to_string();

        let status = iter
            .find(|line| line.contains("Active:"))
            .context("Error parsing status.")?
            .split(':')
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();

        if !status.starts_with("active") {
            return Ok(StatusOutput::Raw(stdout));
        }

        let stdin = APPS_DIR.to_string() + &self.name + "/" + &self.name + ".stdin";

        let active = status
            .split("since")
            .nth(0)
            .unwrap_or("No status found.")
            .trim()
            .to_string();

        let pid = iter
            .find(|line| line.contains("Main PID:"))
            .context("Error finding PID.")?
            .split(':')
            .nth(1)
            .and_then(|line| line.split(' ').nth(1))
            .unwrap_or("No PID found.")
            .to_string()
            .parse()?;

        let cgroup = iter
            .clone()
            .skip_while(|line| !line.starts_with("CGroup:"))
            .nth(1)
            .context("Error parsing CGroup.")?;

        let cmd = cgroup.split_once(' ').unwrap().1.to_string();

        Ok(StatusOutput::Pretty(Status {
            script,
            stdin,
            pid,
            active,
            cmd,
        }))
    }

    fn list(&self) -> Result<Vec<String>> {
        let output = self.run_command(vec!["list-unit-files", "cres.*.service"])?;
        let out = String::from_utf8(output.stdout)?;
        let output_lines = out.lines().collect::<Vec<&str>>();
        let names: Vec<String> = output_lines
            .iter()
            .filter(|line| line.starts_with("cres."))
            .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
            .collect();
        Ok(names)
    }
}
