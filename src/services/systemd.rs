use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

use crate::{
    service::{InitSystem, Status, StatusOutput},
    APPS_DIR, HOME_DIR,
};

use anyhow::{Context, Result};

const SCRIPTS_DIR: &str = concat!(
    env!("HOME", "Error retrieving HOME directory."),
    "/.config/systemd/user/"
);

/// `systemd` implementation.
pub struct Systemd {
    /// `<name>`
    name: String,
    /// `cres.<name>.service`
    service_name: String,
    /// `cres.<name>.socket`
    socket_name: String,
}

impl Systemd {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            service_name: String::new(),
            socket_name: String::new(),
        }
    }

    /// Run a `systemctl` command as the user.
    fn run_command(&self, args: Vec<&str>) -> Result<Output> {
        let cmd = Command::new("systemctl")
            .arg("--user")
            .arg("--no-pager")
            .args(args)
            .output()?;
        Ok(cmd)
    }

    fn write_service_unit(&self, path: PathBuf, cmd: &str) -> Result<()> {
        let description = format!("Description=Service unit for '{}'", self.name);
        let requires = format!("Requires={}", self.socket_name);
        let after = format!("After=network.target {}", self.socket_name);
        let exec_start = format!("ExecStart={cmd}");

        let service = [
            "[Unit]",
            &description,
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
            "WantedBy=default.target",
            "",
        ];

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, service.join("\n"))?;
        Ok(())
    }

    fn write_socket_unit(&self, path: PathBuf) -> Result<()> {
        let description = format!("Description=Socket unit for '{}'", self.service_name);
        let listen_fifo = format!(
            "ListenFIFO={}/stdin",
            APPS_DIR.to_string() + "/" + &self.name
        );

        let socket = [
            "[Unit]",
            &description,
            "",
            "[Socket]",
            &listen_fifo,
            "",
            "[Install]",
            "WantedBy=sockets.target",
            "",
        ];

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, socket.join("\n"))?;
        Ok(())
    }
}

impl InitSystem for Systemd {
    fn set_service_name(&mut self, name: &str) {
        self.name = name.to_string();
        self.service_name = format!("cres.{name}.service");
        self.socket_name = format!("cres.{name}.socket");
    }

    fn get_scripts_paths(&self) -> Vec<String> {
        vec![
            SCRIPTS_DIR.to_string() + &self.service_name,
            SCRIPTS_DIR.to_string() + &self.socket_name,
        ]
    }

    fn is_running(&self) -> Result<bool> {
        let output = self.run_command(vec!["is-active", &self.service_name])?;
        let stdout = String::from_utf8(output.stdout)?;
        let is_running = stdout.trim().to_string();
        Ok(is_running == "active")
    }

    fn reload(&self) -> Result<()> {
        self.run_command(vec!["daemon-reload"])?;
        Ok(())
    }

    fn create(&self, cmd: &str) -> Result<()> {
        let path_str = HOME_DIR.to_string() + "/.config/systemd/user/";
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
        eprintln!("Reloading systemd daemon");
        self.reload()?;

        eprintln!("Starting '{}'", self.service_name);
        self.run_command(vec!["start", &self.service_name])?;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        self.run_command(vec!["stop", &self.socket_name])?;
        Ok(())
    }

    fn restart(&self) -> Result<()> {
        self.run_command(vec!["restart", &self.service_name])?;
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
        let mut iter = result.iter();

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

        let cgroup_index = iter
            .position(|line| line.contains("CGroup:"))
            .context("Error parsing CGroup.")?;

        let cmd = result
            .get(cgroup_index + 1)
            .context("Error parsing CGroup.")?
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ");

        Ok(StatusOutput::Pretty(Status {
            script,
            stdin,
            pid,
            active,
            cmd,
        }))
    }

    fn list(&self) -> Result<Vec<String>> {
        let output = self.run_command(vec!["list-unit-files"])?;
        let out = String::from_utf8(output.stdout)?;
        let output_lines = out.lines().collect::<Vec<&str>>();
        let names: Vec<String> = output_lines
            .iter()
            .filter(|line| line.contains("cres.") && line.contains(".service"))
            .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
            .collect();
        Ok(names)
    }
}
