use std::fs::File;

use crate::{
    application::{self, Application},
    crescent,
    util::print_title_cyan,
};

use anyhow::{Context, Result};
use clap::Args;
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub apps: Vec<Application>,
}

#[derive(Args)]
#[command(about = "Save all currently running application.")]
pub struct SaveArgs;

impl SaveArgs {
    pub fn run(self) -> Result<()> {
        let apps_dir = crescent::get_apps_dir()?;

        let dirs = apps_dir
            .read_dir()
            .context("Error reading apps directory.")?
            .flatten();

        let mut save = SaveFile { apps: vec![] };

        for app_dir in dirs {
            let name = app_dir.file_name().to_str().unwrap().to_string();

            if let Ok(app_info) = application::get_app_info(&name) {
                save.apps.push(app_info);
            }
        }

        if save.apps.is_empty() {
            println!("No application running.");
            return Ok(());
        }

        let mut save_dir = crescent::crescent_dir()?;
        save_dir.push("apps.json");

        let save_file = File::create(save_dir)?;
        serde_json::to_writer_pretty(save_file, &save)?;

        print_title_cyan(&format!("Saved {} apps:", save.apps.len()));

        for app in save.apps {
            println!("{}", app.name.white());
        }

        Ok(())
    }
}
