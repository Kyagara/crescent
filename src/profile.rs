use std::{collections::HashMap, fs, path::PathBuf};

use crate::PROFILES_DIR;

use anyhow::{anyhow, Result};

pub struct Profiles {
    profiles: HashMap<String, Profile>,
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
        let files = fs::read_dir(profiles_dir)?;

        for file in files {
            let file_dir = file?;
            let name = file_dir.file_name();
            let path = file_dir.path();

            if !path.is_file() || !path.to_string_lossy().ends_with(".toml") {
                continue;
            }

            let mut profile = Profile::new();
            let profile_content = fs::read_to_string(path)?;
            profile.parse_profile(&profile_content)?;

            let name = name.to_string_lossy().replace(".toml", "");
            self.profiles.insert(name, profile);
        }

        Ok(())
    }

    /// Write all default profiles to `$HOME/.crescent/profiles/<name>.toml`
    pub fn install_default_profiles(&self) -> Result<()> {
        let profiles_dir = PathBuf::from(PROFILES_DIR);

        for (name, profile) in DEFAULT_PROFILES {
            let profile_path = profiles_dir.join(format!("{name}.toml"));
            eprintln!("Copying profile '{name}'");
            fs::write(profile_path, profile)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Profile {
    pub exec_path: Option<String>,
    pub name: Option<String>,
    pub interpreter: Option<String>,
    pub arguments: Option<String>,
}

impl Profile {
    pub fn new() -> Self {
        Self {
            exec_path: None,
            name: None,
            interpreter: None,
            arguments: None,
        }
    }

    fn parse_profile(&mut self, profile_content: &str) -> Result<()> {
        for line in profile_content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').to_string();

                if !value.is_empty() {
                    match key {
                        "exec_path" => self.exec_path = Some(value),
                        "name" => self.name = Some(value),
                        "interpreter" => self.interpreter = Some(value),
                        "arguments" => self.arguments = Some(value),
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

const DEFAULT_PROFILES: [(&str, &str); 3] = [
    ("example", EXAMPLE_PROFILE),
    ("mc-10g", MC_10G_SERVER_PROFILE),
    ("velocity", VELOCITY_PROXY_PROFILE),
];

const EXAMPLE_PROFILE: &str = r#"# Example profile that executes a long running service with an interpreter
name = "example"
# Provide the interpreter if necessary and optionally pass arguments to it
interpreter = "python3 -O"
# Path to the executable
#exec_path = "./tools/long_running_service.py"
# Arguments passed to the executable
arguments = "--log=info -a b
"#;

const MC_10G_SERVER_PROFILE: &str = r#"# Aikar's flags for Minecraft servers, 10G ram with GC logging, Java 11+. Works well for Fabric and Paper servers. https://docs.papermc.io/paper/aikars-flags#recommended-jvm-startup-flags
name = "minecraft"
interpreter = "java -Xms10G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1 -Dusing.aikars.flags=https://mcflags.emc.gs -Daikars.new.flags=true -jar"
arguments = "--nogui"
"#;

const VELOCITY_PROXY_PROFILE: &str = r#"# Recommended flags for Velocity. https://docs.papermc.io/velocity/tuning
name = "velocity"
interpreter = "java -Xms1024M -Xmx1024M -XX:+UseG1GC -XX:G1HeapRegionSize=4M -XX:+UnlockExperimentalVMOptions -XX:+ParallelRefProcEnabled -XX:+AlwaysPreTouch -XX:MaxInlineLevel=15 -jar"
"#;
