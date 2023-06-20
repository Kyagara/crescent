use crate::crescent::{self, Profile};
use anyhow::Result;
use clap::Args;
use std::fs;

#[derive(Args)]
#[command(about = "Verify and print a profile.")]
pub struct ProfileArgs {
    #[arg(help = "Profile name.")]
    pub profile: String,
}

impl ProfileArgs {
    pub fn run(self) -> Result<()> {
        let profile_path = crescent::get_profile_path(self.profile)?;
        let json_str = fs::read_to_string(profile_path)?;
        let profile: Profile = serde_json::from_str(&json_str)?;
        let profile_pretty = serde_json::to_string_pretty(&profile)?;

        println!("{profile_pretty}");

        Ok(())
    }
}
