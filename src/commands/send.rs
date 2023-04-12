use crate::process;
use clap::Args;
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Args)]
#[command(about = "Send a command to an application.")]
pub struct SendArgs {
    #[arg(help = "The application name")]
    pub name: String,
    #[arg(help = "The command you want to send")]
    pub command: String,
}

impl SendArgs {
    pub fn run(name: String, command: String) {
        let mut work_dir = process::application_temp_dir_by_name(name.clone());

        work_dir.push(name + ".sock");

        let mut stream = UnixStream::connect(work_dir).unwrap();

        let message = format!("{}\n", command);

        stream.write_all(message.as_bytes()).unwrap();
    }
}
