use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

fn main() -> Result<()> {
    let home = env::var("HOME").context("Error getting HOME env.")?;

    let mut crescent_dir = PathBuf::from(home);

    crescent_dir.push(".crescent/profiles");

    if !crescent_dir.exists() {
        fs::create_dir_all(&crescent_dir)
            .context("Error creating crescent and profiles directory.")?;
    }

    let base_profiles_dir = PathBuf::from("./profiles");

    let base_profiles = base_profiles_dir
        .read_dir()
        .context("Error reading base profiles directory.")?
        .flatten();

    'base_loop: for base_profile in base_profiles {
        let user_profiles = crescent_dir
            .read_dir()
            .context("Error reading user profiles directory.")?
            .flatten();

        for user_profile in user_profiles {
            if user_profile.file_name() == base_profile.file_name() {
                continue 'base_loop;
            }
        }

        fs::copy(
            base_profile.path(),
            crescent_dir.join(base_profile.file_name()),
        )?;
    }

    Ok(())
}
