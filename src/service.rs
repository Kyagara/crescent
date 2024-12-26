use std::{fs::OpenOptions, io::Read, path::PathBuf};

use anyhow::{anyhow, Result};

use crate::{
    init_system::InitSystem, init_systems::systemd::Systemd, logger::Logger,
    loggers::journald::Journald, APPS_DIR,
};

/// Represents a crescent service.
///
/// A crescent service is a background service created by one of the supported init systems using crescent.
///
/// Contains methods to retrieve interfaces to interact with a service ([`Self::init_system()`]),
/// its logging system ([`Self::logger()`]) and to retrieve paths such as the service stdin, command history, etc.
pub struct Service {
    /// The short name of the service, does not include any prefix or suffix.
    pub name: String,
}

impl Service {
    pub fn from(name: Option<&str>) -> Self {
        match name {
            Some(name) => {
                let name = name.to_string();
                Self { name }
            }
            None => {
                let name = String::new();
                Self { name }
            }
        }
    }

    /// Returns an [`InitSystem`] interface for the current init system.
    pub fn init_system(&self) -> impl InitSystem {
        match self.name.is_empty() {
            true => Systemd::new(None),
            false => Systemd::new(Some(&self.name)),
        }
    }

    /// Returns a [`Logger`] interface.
    pub fn logger(&self) -> impl Logger {
        Journald::new(&self.name)
    }

    /// Check if the service folder exists, return an error if it doesn't.
    pub fn exists(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Service name is not set"));
        }

        match PathBuf::from(APPS_DIR.to_owned()).join(&self.name).exists() {
            true => Ok(()),
            false => Err(anyhow!("Service folder '{}' does not exist", self.name)),
        }
    }

    /// Get the path of the service stdin file.
    pub fn stdin_path(&self) -> Result<String> {
        if self.name.is_empty() {
            return Err(anyhow!("Service name is not set"));
        }
        let stdin = format!("{}/{}/stdin", APPS_DIR.clone(), self.name);
        Ok(stdin)
    }

    /// Get the path of the service command history.
    pub fn history_path(&self) -> Result<String> {
        if self.name.is_empty() {
            return Err(anyhow!("Service name is not set"));
        }
        let history = format!("{}/{}/history", APPS_DIR.clone(), self.name);
        Ok(history)
    }

    /// Read all lines from the service command history.
    pub fn read_command_history(&self) -> Result<Vec<String>> {
        let path = self.history_path()?;

        let mut history_file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)
            .expect("Failed to open history file");

        let mut history = String::new();
        history_file.read_to_string(&mut history)?;
        Ok(history.lines().map(|line| line.to_string()).rev().collect())
    }
}
