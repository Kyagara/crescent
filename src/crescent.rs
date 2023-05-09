use anyhow::{Context, Result};
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
}
