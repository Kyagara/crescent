use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

use crate::{
    service::{InitSystem, Status},
    APPS_DIR, HOME_DIR,
};

use anyhow::{Context, Result};

/// Systemd implementation.
///
/// Units paths:
///
/// Service: $HOME/.config/systemd/user/cres.example.service
///
/// Socket: $HOME/.config/systemd/user/cres.example.socket
pub struct Systemd {
    /// 'example'
    name: String,
    /// 'cres.example.service'
    service_name: String,
    /// 'cres.example.socket'
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

    /// Run a systemctl command as the user.
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

    fn start(&self, cmd: &str) -> Result<()> {
        let path_str = HOME_DIR.to_string() + "/.config/systemd/user/";
        let path = PathBuf::from(path_str);

        eprintln!("Writing '{}' unit", self.service_name);
        let service_path = path.join(self.service_name.clone());
        self.write_service_unit(service_path, cmd)?;

        eprintln!("Writing '{}' unit", self.socket_name);
        let socket_path = path.join(self.socket_name.clone());
        self.write_socket_unit(socket_path)?;

        eprintln!("Reloading systemd daemon");
        self.run_command(vec!["daemon-reload"])?;

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

    fn status(&self, raw: bool) -> Result<Status> {
        let output = match self.run_command(vec!["status", &self.service_name]) {
            Ok(output) => output,
            Err(err) => return Err(err),
        };

        let out = String::from_utf8(output.stdout)?;

        if raw {
            // Print the output without any modification
            println!("{out}");
            return Ok(Status {
                script: String::new(),
                stdin: String::new(),
                active: String::new(),
                pid: 0,
                cmd: String::new(),
            });
        }

        let result: Vec<&str> = out.lines().collect::<Vec<&str>>();

        let script = result
            .iter()
            .find(|line| line.contains("Loaded:"))
            .context("Error finding Loaded line.")?
            .split('(')
            .nth(1)
            .and_then(|s| s.split(';').next())
            .unwrap_or("")
            .trim()
            .to_string();

        let status = result
            .iter()
            .find(|line| line.contains("Active:"))
            .context("Error parsing status.")?
            .split(':')
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();

        if !status.starts_with("active") {
            return Ok(Status {
                script,
                stdin: String::new(),
                active: status,
                pid: 0,
                cmd: String::new(),
            });
        }

        let stdin = APPS_DIR.to_string() + &self.name + "/" + &self.name + ".stdin";

        let active = status
            .split("since")
            .nth(0)
            .unwrap_or("No status found.")
            .trim()
            .to_string();

        let pid = result
            .iter()
            .find(|line| line.contains("Main PID:"))
            .context("Error finding PID.")?
            .split(':')
            .nth(1)
            .and_then(|line| line.split(' ').nth(1))
            .unwrap_or("No PID found.")
            .to_string()
            .parse()?;

        let cgroup_index = result
            .iter()
            .position(|line| line.contains("CGroup:"))
            .context("Error parsing CGroup.")?;

        let cmd = result
            .get(cgroup_index + 1)
            .context("Error parsing CGroup.")?
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ");

        Ok(Status {
            script,
            stdin,
            pid,
            active,
            cmd,
        })
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

    fn is_running(&self) -> Result<bool> {
        let output = self.run_command(vec!["is-active", &self.service_name])?;
        let out = String::from_utf8(output.stdout)?;
        let is_running = out.trim().to_string();
        Ok(is_running == "active")
    }
}
