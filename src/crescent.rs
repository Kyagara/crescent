use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    // Not used
    pub __comment: Option<String>,
    // Not used
    pub __version: Option<i8>,
    pub file_path: Option<String>,
    pub name: Option<String>,
    pub interpreter: Option<String>,
    pub interpreter_arguments: Option<Vec<String>>,
    pub application_arguments: Option<Vec<String>>,
    pub stop_command: Option<String>,
}

pub fn crescent_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("Error getting HOME env.")?;
    let mut crescent_dir = PathBuf::from(home);
    crescent_dir.push(".crescent");
    if !crescent_dir.exists() {
        fs::create_dir_all(&crescent_dir).context("Error creating crescent directory.")?;
    }
    Ok(crescent_dir)
}

pub fn get_apps_dir() -> Result<PathBuf> {
    let mut apps_dir = crescent_dir()?;
    apps_dir.push("apps");
    if !apps_dir.exists() {
        fs::create_dir_all(&apps_dir).context("Error creating apps directory.")?;
    }
    Ok(apps_dir)
}

pub fn get_profiles_dir() -> Result<PathBuf> {
    let mut profiles_dir = crescent_dir()?;
    profiles_dir.push("profiles");
    if !profiles_dir.exists() {
        fs::create_dir_all(&profiles_dir).context("Error creating profiles directory.")?;
    }
    Ok(profiles_dir)
}

pub fn get_profile(profile: &str) -> Result<Profile> {
    let mut profiles_dir = get_profiles_dir()?;
    profiles_dir.push(profile.to_owned() + ".json");

    if !profiles_dir.exists() || !profiles_dir.is_file() {
        return Err(anyhow!("Profile not found."));
    }

    match File::open(profiles_dir) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let profile: Profile = serde_json::from_reader(reader)?;
            Ok(profile)
        }
        Err(err) => Err(anyhow!("Error opening profile file: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unit_crescent_dir() -> Result<()> {
        let home = env::var("HOME")?;
        let mut home_path = PathBuf::from(home);
        home_path.push(".crescent");

        assert_eq!(crescent_dir()?, home_path);
        Ok(())
    }

    #[test]
    fn unit_get_profiles_dir() -> Result<()> {
        get_profiles_dir()?;
        Ok(())
    }

    #[test]
    fn unit_get_profile() -> Result<()> {
        let profile = get_profile(&String::from("example"))?;
        assert!(profile.__comment.is_some());

        let err = get_profile(&String::from("does-not-exist")).unwrap_err();
        assert_eq!(format!("{}", err), "Profile not found.");
        Ok(())
    }
}
