use std::{collections::HashMap, fs, path::PathBuf};

use crate::PROFILES_DIR;

use anyhow::{anyhow, Result};

pub struct Profiles {
    profiles: HashMap<String, Profile>,
}

#[derive(Clone)]
pub struct Profile {
    pub exec_path: Option<PathBuf>,
    pub name: Option<String>,
    pub interpreter: Option<String>,
    pub arguments: Option<String>,
    pub stop_command: Option<String>,
}

impl Profiles {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Get the profile with the given name.
    pub fn get_profile(&mut self, name: &str) -> Result<Profile> {
        if self.profiles.is_empty() {
            self.load_profiles()?;
        }

        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| anyhow!("Profile '{name}' not found."))?
            .clone();

        Ok(profile)
    }

    fn load_profiles(&mut self) -> Result<()> {
        let profiles_dir = PathBuf::from(PROFILES_DIR);

        if !profiles_dir.is_dir() {
            fs::create_dir(&profiles_dir)?;
            return Ok(());
        }

        let files = fs::read_dir(&profiles_dir)?;

        for file in files {
            let file_dir = file?;
            let name = file_dir.file_name();
            let path = file_dir.path();

            if !path.is_file() || !path.to_string_lossy().ends_with(".toml") {
                continue;
            }

            let profile_str = fs::read_to_string(path)?;
            let profile = self.parse_profile(&profile_str)?;

            let name = name.to_string_lossy().replace(".toml", "");
            self.profiles.insert(name, profile);
        }

        Ok(())
    }

    fn parse_profile(&self, profile_str: &str) -> Result<Profile> {
        let mut profile = Profile {
            exec_path: None,
            name: None,
            interpreter: None,
            arguments: None,
            stop_command: None,
        };

        for line in profile_str.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').to_string();

                if !value.is_empty() {
                    match key {
                        "exec_path" => profile.exec_path = Some(PathBuf::from(value)),
                        "name" => profile.name = Some(value),
                        "interpreter" => profile.interpreter = Some(value),
                        "arguments" => profile.arguments = Some(value),
                        "stop_command" => profile.stop_command = Some(value),
                        _ => {}
                    }
                }
            }
        }

        Ok(profile)
    }
}
