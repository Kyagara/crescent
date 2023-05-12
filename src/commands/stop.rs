use super::signal;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Send a SIGTERM signal to the application subprocess.")]
pub struct StopArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

impl StopArgs {
    pub fn run(self) -> Result<()> {
        signal::generic_send_signal_command(&self.name, &15)
    }
}
