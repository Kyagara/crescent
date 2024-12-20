use std::{fs::OpenOptions, io::Read, path::PathBuf};

use anyhow::{anyhow, Result};

use crate::{
    logger::Logger, loggers::journald::Journald, system::InitSystem, systems::systemd::Systemd,
    APPS_DIR,
};

/// Represents a crescent application.
pub struct Application {
    /// The short name of the application.
    pub name: String,
}

impl Application {
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

    /// Returns an [`InitSystem`] interface for the current `init` system.
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

    /// Check if application exists, return an error if it doesn't.
    pub fn exists(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Service name is not set"));
        }

        match PathBuf::from(APPS_DIR).join(&self.name).exists() {
            true => Ok(()),
            false => Err(anyhow!("Application '{}' does not exist", self.name)),
        }
    }

    /// Get the path to the application's stdin file.
    pub fn stdin_path(&self) -> Result<String> {
        if self.name.is_empty() {
            return Err(anyhow!("Service name is not set"));
        }
        let stdin = format!("{APPS_DIR}/{}/stdin", self.name);
        Ok(stdin)
    }

    /// Get the path to the application's history file.
    pub fn history_path(&self) -> Result<String> {
        if self.name.is_empty() {
            return Err(anyhow!("Service name is not set"));
        }
        let history = format!("{APPS_DIR}/{}/history", self.name);
        Ok(history)
    }

    /// Read all lines inside the command history file for the application.
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
