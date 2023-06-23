use anyhow::{anyhow, Result};
use std::{env, fs, path::PathBuf};

// This build file creates the ~/.crescent/profiles directories and
// copies the default profiles in ./profiles to ~/.crescent/profiles.
fn main() -> Result<()> {
    let home_dir = match env::var("HOME") {
        Ok(dir) => dir,
        Err(err) => return Err(anyhow!("Error retrieving HOME directory: {err}")),
    };

    let mut crescent_dir = PathBuf::from(home_dir);
    crescent_dir.push(".crescent/profiles");

    if !crescent_dir.exists() {
        if let Err(err) = fs::create_dir_all(&crescent_dir) {
            return Err(anyhow!(
                "Error creating crescent and profiles directory: {err}"
            ));
        }
    }

    let default_profiles_dir = PathBuf::from("./profiles");
    let default_profiles = match default_profiles_dir.read_dir() {
        Ok(dir) => dir.flatten(),
        Err(err) => return Err(anyhow!("Error reading default profiles directory: {err}")),
    };

    'base_loop: for default_profile in default_profiles {
        let user_profiles = match crescent_dir.read_dir() {
            Ok(dir) => dir.flatten(),
            Err(err) => return Err(anyhow!("Error reading default user directory: {err}")),
        };

        for user_profile in user_profiles {
            if user_profile.file_name() == default_profile.file_name() {
                continue 'base_loop;
            }
        }

        if let Err(err) = fs::copy(
            default_profile.path(),
            crescent_dir.join(default_profile.file_name()),
        ) {
            return Err(anyhow!("Error copying profile: {err}"));
        }
    }

    Ok(())
}
