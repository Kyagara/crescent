use std::{fs::OpenOptions, io::Read, path::PathBuf};

use crate::APPS_DIR;

use anyhow::{anyhow, Result};

pub struct Application {
    pub name: String,
    pub service_name: String,
}

impl Application {
    pub fn from(name: &str) -> Self {
        let service_name = format!("cres.{}.service", name);
        Self {
            name: name.to_string(),
            service_name,
        }
    }

    /// Check if application exists, return an error if it doesn't.
    pub fn exists(&self) -> Result<()> {
        match PathBuf::from(APPS_DIR).join(&self.name).exists() {
            true => Ok(()),
            false => Err(anyhow!("Application '{}' does not exist", self.name)),
        }
    }

    /// Get the path to the application's stdin file.
    pub fn stdin_path(&self) -> Result<String> {
        let stdin = format!("{}/{}/stdin", APPS_DIR, self.name);
        Ok(stdin)
    }

    /// Get the path to the application's history file.
    pub fn history_path(&self) -> Result<String> {
        let history = format!("{}/{}/history", APPS_DIR, self.name);
        Ok(history)
    }

    /// Read all lines inside the command history file for the application.
    pub fn read_command_history(&self) -> Result<Vec<String>> {
        let path = format!("{}/{}/history", APPS_DIR, self.name);

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
