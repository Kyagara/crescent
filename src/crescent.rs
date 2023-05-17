use anyhow::{anyhow, Context, Result};
use std::{env, fs, path::PathBuf};

pub fn crescent_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("Error getting HOME env.")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent");

    if !crescent_dir.exists() {
        fs::create_dir_all(&crescent_dir).context("Error creating crescent directory.")?;
    }

    Ok(crescent_dir)
}

pub fn get_profile_path(profile: String) -> Result<PathBuf> {
    match fs::canonicalize(&profile) {
        Ok(path) => Ok(path),
        Err(_) => {
            let mut crescent_dir = crescent_dir()?;
            crescent_dir.push("profiles");
            crescent_dir.push(profile + ".json");

            if crescent_dir.exists() && crescent_dir.is_file() {
                return Ok(crescent_dir);
            }

            Err(anyhow!("Profile not found."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unit_crescent_dir_created() -> Result<()> {
        let home = env::var("HOME")?;
        let mut home_path = PathBuf::from(home);
        home_path.push(".crescent");

        assert_eq!(crescent_dir()?, home_path);

        Ok(())
    }

    #[test]
    fn unit_get_profile_path() -> Result<()> {
        let mut path = get_profile_path(String::from("example"))?;
        assert!(path.exists());
        assert!(path.is_file());

        path = get_profile_path(String::from("./profiles/example.json"))?;
        assert!(path.exists());
        assert!(path.is_file());

        let err = get_profile_path(String::from("profile/does/not/exist")).unwrap_err();

        assert_eq!(format!("{}", err), "Profile not found.");

        Ok(())
    }
}
