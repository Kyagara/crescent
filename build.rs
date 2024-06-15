use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Result};

// This build file creates the required `HOME/.crescent` directories and
// copies the default profiles in the project folder to ~/.crescent/profiles.
fn main() -> Result<()> {
    let home_dir = env!("HOME", "Error retrieving HOME directory.");

    fs::create_dir_all(PathBuf::from(home_dir).join(".crescent/apps/"))?;
    let profiles_dir = PathBuf::from(home_dir).join(".crescent/profiles/");

    let default_profiles = match PathBuf::from("./profiles").read_dir() {
        Ok(dir) => dir.flatten(),
        Err(err) => return Err(anyhow!("Error reading default profiles directory: {err}")),
    };

    'base_loop: for default_profile in default_profiles {
        let user_profiles = match profiles_dir.read_dir() {
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
            profiles_dir.join(default_profile.file_name()),
        ) {
            return Err(anyhow!("Error copying profile: {err}"));
        }
    }

    Ok(())
}
